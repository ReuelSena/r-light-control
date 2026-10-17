//! Importador de fixtures `.qxf` do QLC+ (E3): converte a definição para o formato próprio e
//! anota no relatório o que ficou de fora ou mudou. As regras seguem o carregador do QLC+ 5.3.0
//! (`engine/src/qlc*.cpp`), que é mais tolerante que o `fixture.xsd`.

use rlc_core::{Attribute, Capability, Channel, FixtureDef, FixtureKind, Mode, Physical};
use roxmltree::{Document, Node, ParsingOptions};

/// Converte o XML de um `.qxf`. Devolve a definição e os avisos, ou o motivo da recusa.
pub fn import(xml: &str) -> Result<(FixtureDef, Vec<String>), String> {
    let options = ParsingOptions {
        allow_dtd: true,
        ..Default::default()
    };
    let doc = Document::parse_with_options(xml.trim_start_matches('\u{feff}'), options)
        .map_err(|e| format!("O arquivo não é um XML válido ({e})."))?;
    let root = doc.root_element();
    if root.tag_name().name() != "FixtureDefinition" {
        return Err("O arquivo não é uma fixture do QLC+.".into());
    }
    let (manufacturer, model) = (text(root, "Manufacturer"), text(root, "Model"));
    let id = slug(&format!("{manufacturer} {model}"));
    if id.is_empty() {
        return Err("A fixture não tem fabricante nem modelo.".into());
    }
    let kind = match text(root, "Type").as_str() {
        "Color Changer" => FixtureKind::ColorChanger,
        "Dimmer" => FixtureKind::Dimmer,
        "Moving Head" => FixtureKind::MovingHead,
        "Scanner" => FixtureKind::Scanner,
        "LED Bar (Beams)" | "LED Bar (Pixels)" => FixtureKind::LedBar,
        "Strobe" => FixtureKind::Strobe,
        "Hazer" => FixtureKind::Hazer,
        "Smoke" => FixtureKind::Smoke,
        "Laser" => FixtureKind::Laser,
        "Effect" | "Flower" => FixtureKind::Effect,
        _ => FixtureKind::Other,
    };
    let mut warnings = Vec::new();
    let (mut aliases, mut gobo_images) = (false, false);

    // Canais: o atributo vem do preset ou, sem ele, do grupo. Os marcados como finos são conferidos
    // depois, nos modos.
    let mut channels = Vec::new();
    let mut marked_fine = Vec::new();
    for node in children(root, "Channel") {
        let name = node.attribute("Name").unwrap_or_default().to_string();
        if name.is_empty() {
            warnings.push("Canal sem nome ignorado.".into());
            continue;
        }
        let preset = node.attribute("Preset");
        let group = children(node, "Group").next();
        let (attribute, fine) = match preset {
            Some(preset) => preset_attribute(preset).ok_or(preset.to_string()),
            None => {
                let name = text(node, "Group");
                let fine = group.and_then(|g| g.attribute("Byte")) == Some("1");
                group_attribute(&name, &text(node, "Colour"))
                    .map(|a| (a, fine))
                    .ok_or(name)
            }
        }
        .unwrap_or_else(|unknown| {
            warnings.push(format!(
                "Canal \"{name}\": \"{unknown}\" sem equivalente, marcado como \"outro\"."
            ));
            (Attribute::Other, false)
        });

        // Com preset, o QLC+ cria a faixa sozinho (e ignora as do arquivo)
        let mut capabilities: Vec<Capability> = Vec::new();
        for cap in children(node, "Capability").filter(|_| preset.is_none()) {
            let limit = |attr| {
                cap.attribute(attr)
                    .map(|v| v.trim().parse::<i64>().unwrap_or(0).clamp(0, 255) as u8)
            };
            let label = cap
                .text()
                .unwrap_or_default()
                .split_whitespace()
                .collect::<Vec<_>>()
                .join(" ");
            let (Some(min), Some(max)) = (limit("Min"), limit("Max")) else {
                warnings.push(format!(
                    "Canal \"{name}\": faixa \"{label}\" sem limites, ficou de fora."
                ));
                continue;
            };
            if min > max || capabilities.iter().any(|c| min <= c.max && max >= c.min) {
                warnings.push(format!(
                    "Canal \"{name}\": faixa \"{label}\" ({min}–{max}) invertida ou sobreposta a outra, ficou de fora."
                ));
                continue;
            }
            let cap_preset = cap.attribute("Preset").unwrap_or_default();
            aliases |= cap_preset == "Alias" || children(cap, "Alias").next().is_some();
            gobo_images |= cap.attribute("Res").is_some()
                || (matches!(
                    cap_preset,
                    "GoboMacro" | "GoboShakeMacro" | "GenericPicture"
                ) && !cap.attribute("Res1").unwrap_or_default().is_empty());
            let file_color = match cap_preset {
                "ColorMacro" | "ColorDoubleMacro" => cap.attribute("Res1").filter(|c| {
                    c.len() == 7
                        && c.starts_with('#')
                        && c[1..].chars().all(|h| h.is_ascii_hexdigit())
                }),
                _ => None,
            };
            let color = match file_color {
                Some(color) => Some(color.to_ascii_lowercase()),
                None if matches!(attribute, Attribute::ColorWheel | Attribute::ColorMacro) => {
                    color_by_name(&label)
                }
                None => None,
            };
            capabilities.push(Capability {
                min,
                max,
                name: label,
                color,
            });
        }
        if capabilities.is_empty() {
            capabilities.push(Capability {
                min: 0,
                max: 255,
                name: name.clone(),
                color: None,
            });
        }
        marked_fine.push(fine);
        channels.push(Channel {
            default: node
                .attribute("Default")
                .and_then(|v| v.trim().parse().ok())
                .unwrap_or(0),
            name,
            attribute,
            fine: false,
            capabilities,
        });
    }
    if channels.is_empty() {
        return Err("A fixture não tem canais.".into());
    }

    // Modos: canais pelo nome e cabeças pela posição no modo
    let mut physical = children(root, "Physical").next().map(physical_of);
    let mut mode_physical = false;
    let mut modes = Vec::new();
    for node in children(root, "Mode") {
        let name = node.attribute("Name").unwrap_or_default().to_string();
        if name.is_empty() {
            warnings.push("Modo sem nome ignorado.".into());
            continue;
        }
        let mut indices: Vec<u16> = Vec::new();
        for wanted in children(node, "Channel").map(|c| c.text().unwrap_or_default()) {
            // Dois canais com o mesmo nome: cada referência pega o próximo ainda fora do modo
            let found = (0..channels.len() as u16)
                .find(|&i| channels[usize::from(i)].name == wanted && !indices.contains(&i));
            match found {
                Some(i) => indices.push(i),
                None => warnings.push(format!(
                    "Modo \"{name}\": canal \"{wanted}\" {}, ficou de fora.",
                    match channels.iter().any(|c| c.name == wanted) {
                        true => "repetido",
                        false => "não existe",
                    }
                )),
            }
        }
        if indices.is_empty() {
            warnings.push(format!("Modo \"{name}\" sem canais ignorado."));
            continue;
        }
        let mut heads: Vec<Vec<u16>> = Vec::new();
        for head in children(node, "Head") {
            let mut positions: Vec<u16> = Vec::new();
            for position in children(head, "Channel").filter_map(|c| c.text()?.trim().parse().ok())
            {
                if usize::from(position) < indices.len() && !positions.contains(&position) {
                    positions.push(position);
                }
            }
            if !positions.is_empty() {
                heads.push(positions);
            }
        }
        if let Some(own) = children(node, "Physical").next().map(physical_of) {
            match &physical {
                None => physical = Some(own),
                Some(global) => mode_physical |= *global != own,
            }
        }
        modes.push(Mode {
            name,
            channels: indices,
            heads,
        });
    }
    if modes.is_empty() {
        return Err("A fixture não tem modos.".into());
    }

    // Fino só depois de um canal grosso do mesmo tipo, em algum modo, como o QLC+ junta os 16 bits
    // (o fino pode vir logo depois do grosso ou mais à frente: "Pan, Tilt, Pan fino, Tilt fino")
    for (i, _) in marked_fine
        .iter()
        .enumerate()
        .filter(|(_, marked)| **marked)
    {
        let after_coarse = modes.iter().any(|m| {
            let position = m.channels.iter().position(|&c| usize::from(c) == i);
            m.channels[..position.unwrap_or(0)].iter().any(|&c| {
                let coarse = usize::from(c);
                !marked_fine[coarse] && channels[coarse].attribute == channels[i].attribute
            })
        });
        let used = modes.iter().any(|m| m.channels.contains(&(i as u16)));
        if after_coarse {
            channels[i].fine = true;
        } else if used {
            warnings.push(format!(
                "Canal \"{}\" marcado como fino, sem o canal grosso antes dele: fica como canal normal.",
                channels[i].name
            ));
        }
    }
    if aliases {
        warnings.push("A troca de canais conforme o valor (Alias) não é suportada: os modos ficam com os canais padrão.".into());
    }
    if gobo_images {
        warnings.push("As imagens dos gobos ficaram de fora: as faixas guardam só o nome.".into());
    }
    if mode_physical {
        warnings.push("Dados físicos diferentes por modo: vale o geral.".into());
    }
    let def = FixtureDef {
        id,
        manufacturer,
        model,
        kind,
        channels,
        modes,
        physical: physical.unwrap_or_default(),
    };
    Ok((def, warnings))
}

/// Filhos com o nome dado (o .qxf tem namespace; vale só o nome local).
fn children<'a, 'i>(node: Node<'a, 'i>, tag: &'static str) -> impl Iterator<Item = Node<'a, 'i>> {
    node.children().filter(move |n| n.tag_name().name() == tag)
}

/// Texto do primeiro filho com o nome dado, sem os espaços das pontas.
fn text(node: Node, tag: &'static str) -> String {
    children(node, tag)
        .next()
        .and_then(|n| n.text())
        .unwrap_or_default()
        .trim()
        .to_string()
}

/// Identificador da definição (A4): fabricante e modelo em minúsculas, sem acentos, com hífens.
fn slug(text: &str) -> String {
    let mut slug = String::new();
    for c in text.to_lowercase().chars() {
        let c = match c {
            'á' | 'à' | 'â' | 'ã' | 'ä' => 'a',
            'é' | 'è' | 'ê' | 'ë' => 'e',
            'í' | 'ì' | 'î' | 'ï' => 'i',
            'ó' | 'ò' | 'ô' | 'õ' | 'ö' => 'o',
            'ú' | 'ù' | 'û' | 'ü' => 'u',
            'ç' => 'c',
            'ñ' => 'n',
            c => c,
        };
        if c.is_ascii_alphanumeric() {
            slug.push(c);
        } else if !slug.is_empty() && !slug.ends_with('-') {
            slug.push('-');
        }
    }
    slug.trim_end_matches('-').to_string()
}

/// Atributo e "fino" de um canal com preset do QLC+ (`QLCChannel::Preset`).
fn preset_attribute(preset: &str) -> Option<(Attribute, bool)> {
    use Attribute::*;
    let (base, fine) = preset
        .strip_suffix("Fine")
        .map_or((preset, false), |base| (base, true));
    let attribute = match base {
        "IntensityHue" | "IntensitySaturation" | "IntensityLightness" | "IntensityValue" => {
            return None;
        }
        // IntensityRed → cor "Red"; IntensityDimmer e IntensityMasterDimmer → dimmer
        _ if base.starts_with("Intensity") => group_attribute("Intensity", &base[9..])?,
        "PositionPan" | "PositionXAxis" => Pan,
        "PositionTilt" | "PositionYAxis" => Tilt,
        "ColorMacro" | "ColorRGBMixer" => ColorMacro,
        "ColorWheel" => ColorWheel,
        "ColorCTOMixer" | "ColorCTCMixer" | "ColorCTBMixer" => ColorTemperature,
        "GoboWheel" => Gobo,
        "GoboIndex" => GoboRotation,
        "NoFunction" => Other,
        _ if base.starts_with("Speed") => Speed,
        _ if base.starts_with("ShutterStrobe") => Shutter,
        _ if base.starts_with("ShutterIris") => Iris,
        _ if base.starts_with("BeamFocus") => Focus,
        _ if base.starts_with("BeamZoom") => Zoom,
        _ if base.starts_with("PrismRotation") => Prism,
        _ => return None,
    };
    Some((attribute, fine))
}

/// Atributo de um canal sem preset, pelo grupo do QLC+ (e pela cor, no grupo Intensity).
fn group_attribute(group: &str, colour: &str) -> Option<Attribute> {
    use Attribute::*;
    Some(match group {
        // Sem grupo, o QLC+ trata o canal como intensidade
        "Intensity" | "" => match colour {
            "Red" => Red,
            "Green" => Green,
            "Blue" => Blue,
            "Cyan" => Cyan,
            "Magenta" => Magenta,
            "Yellow" => Yellow,
            "Amber" => Amber,
            "White" => White,
            "UV" => Uv,
            "Lime" => Lime,
            "Indigo" => Indigo,
            _ => Dimmer,
        },
        "Colour" => ColorWheel,
        "Gobo" => Gobo,
        "Speed" => Speed,
        "Pan" => Pan,
        "Tilt" => Tilt,
        "Shutter" => Shutter,
        "Prism" => Prism,
        "Beam" | "Effect" => Effect,
        "Maintenance" => Maintenance,
        "Nothing" => Other,
        _ => return None,
    })
}

/// Cor de uma faixa de roda ou macro de cor pelo nome ("Red", "Vermelho", "Pink 2"). Nomes compostos
/// ("White / Red") ficam sem cor.
fn color_by_name(name: &str) -> Option<String> {
    let key = name
        .trim_end_matches(|c: char| c.is_ascii_digit() || c == ' ')
        .to_lowercase();
    let color = match key.as_str() {
        "white" | "branco" => "#ffffff",
        "red" | "vermelho" => "#ff0000",
        "orange" | "laranja" => "#ff8000",
        "yellow" | "amarelo" => "#ffff00",
        "green" | "verde" => "#00ff00",
        "cyan" | "ciano" => "#00ffff",
        "blue" | "azul" => "#0000ff",
        "magenta" => "#ff00ff",
        "pink" | "rosa" => "#ff66cc",
        "purple" | "violet" | "roxo" | "violeta" | "uv" => "#8000ff",
        "amber" | "ambar" | "âmbar" => "#ffbf00",
        "beige" | "bege" => "#f5deb3",
        "lime" => "#80ff00",
        _ => return None,
    };
    Some(color.into())
}

/// Dados físicos de um `<Physical>`. O que falta fica em zero.
fn physical_of(node: Node) -> Physical {
    let get = |tag, attr| {
        children(node, tag)
            .next()
            .and_then(|n| n.attribute(attr))
            .and_then(|v| v.trim().parse::<f32>().ok())
            .unwrap_or(0.0)
    };
    Physical {
        width_mm: get("Dimensions", "Width") as u32,
        height_mm: get("Dimensions", "Height") as u32,
        depth_mm: get("Dimensions", "Depth") as u32,
        weight_kg: get("Dimensions", "Weight"),
        power_w: get("Technical", "PowerConsumption") as u32,
        lens_min_deg: get("Lens", "DegreesMin"),
        lens_max_deg: get("Lens", "DegreesMax"),
        pan_max_deg: get("Focus", "PanMax") as u32,
        tilt_max_deg: get("Focus", "TiltMax") as u32,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Um .qxf mínimo com os canais e modos dados.
    fn qxf(body: &str) -> String {
        format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE FixtureDefinition>
<FixtureDefinition xmlns="http://www.qlcplus.org/FixtureDefinition">
 <Manufacturer>Teste</Manufacturer>
 <Model>Fixture Ação</Model>
 <Type>Moving Head</Type>
 {body}
</FixtureDefinition>"#
        )
    }

    #[test]
    fn id_is_a_slug_without_accents() {
        let (def, _) = import(&qxf(
            r#"<Channel Name="D" Preset="IntensityDimmer"/><Mode Name="1"><Channel Number="0">D</Channel></Mode>"#,
        ))
        .unwrap();
        assert_eq!(def.id, "teste-fixture-acao");
        assert_eq!(def.kind, FixtureKind::MovingHead);
    }

    #[test]
    fn preset_and_group_give_the_attribute() {
        let (def, warnings) = import(&qxf(r#"
 <Channel Name="R" Preset="IntensityRed"/>
 <Channel Name="Lima"><Group Byte="0">Intensity</Group><Colour>Lime</Colour></Channel>
 <Channel Name="CTO" Preset="ColorCTOMixer"/>
 <Channel Name="Iris" Preset="ShutterIrisMinToMax"/>
 <Channel Name="Hue" Preset="IntensityHue"/>
 <Mode Name="1"><Channel>R</Channel><Channel>Lima</Channel><Channel>CTO</Channel><Channel>Iris</Channel><Channel>Hue</Channel></Mode>"#))
        .unwrap();
        let attributes: Vec<_> = def.channels.iter().map(|c| c.attribute).collect();
        use Attribute::*;
        assert_eq!(attributes, [Red, Lime, ColorTemperature, Iris, Other]);
        assert_eq!(warnings.len(), 1);
        assert!(warnings[0].contains("IntensityHue"));
    }

    #[test]
    fn fine_only_after_a_coarse_of_the_same_kind() {
        let (def, warnings) = import(&qxf(r#"
 <Channel Name="Pan" Preset="PositionPan"/>
 <Channel Name="Tilt" Preset="PositionTilt"/>
 <Channel Name="Pan fino" Preset="PositionPanFine"/>
 <Channel Name="Dimmer" Preset="IntensityDimmer"/>
 <Channel Name="Gobo"><Group Byte="1">Gobo</Group></Channel>
 <Mode Name="1"><Channel>Pan</Channel><Channel>Tilt</Channel><Channel>Pan fino</Channel><Channel>Dimmer</Channel><Channel>Gobo</Channel></Mode>"#))
        .unwrap();
        let fine: Vec<_> = def.channels.iter().map(|c| c.fine).collect();
        assert_eq!(fine, [false, false, true, false, false]);
        assert_eq!(warnings.len(), 1);
        assert!(warnings[0].contains("\"Gobo\""));
    }

    #[test]
    fn capabilities_drop_overlaps_and_keep_gaps() {
        let (def, warnings) = import(&qxf(r##"
 <Channel Name="Roda"><Group Byte="0">Colour</Group>
  <Capability Min="0" Max="9">Vermelho</Capability>
  <Capability Min="5" Max="19">Azul</Capability>
  <Capability Min="20" Max="29" Preset="ColorMacro" Res1="#00FF80">Menta</Capability>
  <Capability Min="40" Max="49">White / Red</Capability>
  <Capability Min="60" Max="10">Invertida</Capability>
 </Channel>
 <Channel Name="Reset"><Group Byte="0">Maintenance</Group></Channel>
 <Mode Name="1"><Channel>Roda</Channel><Channel>Reset</Channel></Mode>"##))
        .unwrap();
        let caps: Vec<_> = def.channels[0]
            .capabilities
            .iter()
            .map(|c| (c.min, c.max, c.name.as_str(), c.color.as_deref()))
            .collect();
        assert_eq!(
            caps,
            [
                (0, 9, "Vermelho", Some("#ff0000")),
                (20, 29, "Menta", Some("#00ff80")),
                (40, 49, "White / Red", None),
            ]
        );
        assert_eq!(warnings.len(), 2);
        // Sem faixas no arquivo: uma de 0 a 255 com o nome do canal
        let reset = &def.channels[1].capabilities;
        assert_eq!(
            (reset[0].min, reset[0].max, reset[0].name.as_str()),
            (0, 255, "Reset")
        );
    }

    #[test]
    fn modes_skip_unknown_channels_and_resolve_repeated_names() {
        let (def, warnings) = import(&qxf(r#"
 <Channel Name="Livre" Preset="NoFunction"/>
 <Channel Name="Livre" Preset="NoFunction"/>
 <Channel Name="Dimmer" Preset="IntensityDimmer" Default="255"/>
 <Mode Name="A"><Channel>Livre</Channel><Channel>Fantasma</Channel><Channel>Livre</Channel><Channel>Dimmer</Channel>
  <Head><Channel>3</Channel><Channel>9</Channel></Head></Mode>
 <Mode Name=""><Channel>Dimmer</Channel></Mode>"#))
        .unwrap();
        assert_eq!(def.modes.len(), 1);
        assert_eq!(def.modes[0].channels, [0, 1, 2]);
        // A cabeça aponta para a posição 3, que saiu junto com o canal que não existe
        assert!(def.modes[0].heads.is_empty());
        assert_eq!(def.channels[2].default, 255);
        assert_eq!(warnings.len(), 2);
    }

    #[test]
    fn unsupported_parts_are_reported_once() {
        let (_, warnings) = import(&qxf(r#"
 <Channel Name="Gobo"><Group Byte="0">Gobo</Group>
  <Capability Min="0" Max="9" Preset="GoboMacro" Res1="Others/gobo00001.svg">Aberto</Capability>
  <Capability Min="10" Max="19" Res="Others/gobo00002.svg">Estrela</Capability>
 </Channel>
 <Channel Name="Modo"><Group Byte="0">Maintenance</Group>
  <Capability Min="0" Max="127" Preset="Alias">Normal<Alias Mode="1" Channel="Gobo" With="Gobo"/></Capability>
 </Channel>
 <Mode Name="1"><Channel>Gobo</Channel><Channel>Modo</Channel></Mode>"#))
        .unwrap();
        assert_eq!(warnings.len(), 2);
    }

    #[test]
    fn refuses_what_is_not_a_fixture() {
        assert!(import("não é xml").is_err());
        assert!(import("<Workspace/>").is_err());
        assert!(import(&qxf(r#"<Channel Name="D" Preset="IntensityDimmer"/>"#)).is_err());
        assert!(import(&qxf(r#"<Mode Name="1"/>"#)).is_err());
    }
}

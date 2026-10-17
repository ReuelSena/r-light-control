//! As fixtures do usuário e a biblioteca inteira do QLC+ passando pelo importador.

use std::path::Path;

use rlc_core::{Attribute, FixtureDef, FixtureKind};

/// Definição convertida à mão para o show de exemplo (F3).
fn example_def(id: &str) -> FixtureDef {
    let example: serde_json::Value =
        serde_json::from_str(include_str!("../../rlc-server/src/exemplo.json")).unwrap();
    let defs = example["project"]["fixtureDefs"].as_array().unwrap();
    let def = defs.iter().find(|d| d["id"] == id).unwrap();
    serde_json::from_value(def.clone()).unwrap()
}

#[test]
fn parled_matches_the_hand_conversion() {
    let (def, warnings) = rlc_qxf::import(include_str!("qxf/RSena-ParLed-18x15w.qxf")).unwrap();
    assert_eq!(def, example_def("rsena-parled-18x15w"));
    assert!(warnings.is_empty(), "{warnings:?}");
}

#[test]
fn beam_matches_the_hand_conversion_with_the_strobe_default() {
    let (def, warnings) = rlc_qxf::import(include_str!("qxf/RSena-Beam-7r.qxf")).unwrap();
    assert_eq!(def, example_def("rsena-beam-7r"));
    let strobe = def.channels.iter().find(|c| c.name == "Strobo").unwrap();
    assert_eq!(strobe.default, 255);
    // No .qxf, o "Gobo Wheel" está com Byte="1" sem um canal grosso antes
    assert_eq!(warnings.len(), 1);
    assert!(warnings[0].contains("Gobo Wheel"));
}

#[test]
fn strobe_1000w_imports_with_its_11_channels() {
    let (def, warnings) = rlc_qxf::import(include_str!("qxf/RSena-Strobo-1000w.qxf")).unwrap();
    assert!(warnings.is_empty(), "{warnings:?}");
    assert_eq!(
        (def.id.as_str(), def.kind),
        ("rsena-strobo-1000w", FixtureKind::Strobe)
    );
    assert_eq!(def.modes[0].name, "11CH");
    let attributes: Vec<_> = def.modes[0]
        .channels
        .iter()
        .map(|&i| def.channels[usize::from(i)].attribute)
        .collect();
    use Attribute::*;
    assert_eq!(
        attributes,
        [
            Dimmer, Shutter, Effect, Speed, Red, Green, Blue, Shutter, Other, Other, White
        ]
    );
}

/// As 1.781 fixtures do QLC+ 5.3.0 importam sem recusa. A referência fica fora do git: sem ela, o
/// teste não confere nada.
#[test]
fn whole_qlc_library_imports() {
    let library =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../reference/qlcplus/resources/fixtures");
    let Ok(manufacturers) = std::fs::read_dir(&library) else {
        eprintln!("sem a referência do QLC+ em {}", library.display());
        return;
    };
    let (mut imported, mut refused) = (0, Vec::new());
    for folder in manufacturers.flatten().filter(|e| e.path().is_dir()) {
        for file in std::fs::read_dir(folder.path()).unwrap().flatten() {
            let path = file.path();
            if path.extension().is_none_or(|e| e != "qxf") {
                continue;
            }
            let xml = String::from_utf8_lossy(&std::fs::read(&path).unwrap()).into_owned();
            match rlc_qxf::import(&xml) {
                Ok(_) => imported += 1,
                Err(e) => refused.push(format!("{}: {e}", path.display())),
            }
        }
    }
    assert!(refused.is_empty(), "{refused:#?}");
    assert!(imported > 1700, "só {imported} fixtures");
}

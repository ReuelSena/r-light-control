//! Modelo de dados do r-light-control (contrato v0): só dados, sem lógica.
//! O motor usa estes tipos e a UI recebe os mesmos em TypeScript, gerados pelo ts-rs
//! em `ui/app/types/generated/` ao rodar `cargo test -p rlc-core`.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// Identificador de entidade salva (UUIDv7 em texto, A4). Definições de fixture usam um slug.
pub type Id = String;

/// Estado completo que o servidor envia ao cliente ao conectar (A3).
#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct Snapshot {
    pub project: Project,
    /// Caminho do arquivo do projeto; `None` enquanto não foi salvo.
    pub project_path: Option<String>,
    /// Há alterações não salvas.
    pub dirty: bool,
    pub recent_projects: Vec<String>,
    /// Biblioteca de fixtures do usuário (fica fora do projeto).
    pub library: Vec<FixtureDef>,
    pub interfaces: Vec<DmxInterface>,
    pub midi_devices: Vec<MidiDevice>,
    pub remote: RemoteAccess,
}

#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Project {
    pub name: String,
    pub universes: Vec<Universe>,
    /// Cópia das definições usadas, para o projeto nunca depender da biblioteca (A4).
    pub fixture_defs: Vec<FixtureDef>,
    pub fixtures: Vec<Fixture>,
    pub fixture_groups: Vec<FixtureGroup>,
    pub channel_groups: Vec<ChannelGroup>,
}

#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Universe {
    pub id: Id,
    pub name: String,
    /// Interface DMX que recebe este universo.
    pub output: Option<Id>,
}

// ---------- Definição de fixture (formato próprio, E3) ----------

#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct FixtureDef {
    pub id: Id,
    pub manufacturer: String,
    pub model: String,
    pub kind: FixtureKind,
    pub channels: Vec<Channel>,
    pub modes: Vec<Mode>,
    pub physical: Physical,
}

#[derive(Serialize, Deserialize, TS, Clone, Copy, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum FixtureKind {
    ColorChanger,
    Dimmer,
    MovingHead,
    Scanner,
    LedBar,
    Strobe,
    Hazer,
    Smoke,
    Laser,
    Other,
}

/// O que o canal controla. Taxonomia própria, revisada no importador .qxf (F4).
#[derive(Serialize, Deserialize, TS, Clone, Copy, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum Attribute {
    Dimmer,
    Red,
    Green,
    Blue,
    White,
    Amber,
    Uv,
    Cyan,
    Magenta,
    Yellow,
    ColorWheel,
    ColorMacro,
    Pan,
    Tilt,
    Speed,
    Shutter,
    Gobo,
    GoboRotation,
    Prism,
    Focus,
    Zoom,
    Iris,
    Effect,
    Maintenance,
    Other,
}

#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Channel {
    pub name: String,
    pub attribute: Attribute,
    /// Byte menos significativo de um valor de 16 bits (ex.: Pan fino).
    pub fine: bool,
    pub capabilities: Vec<Capability>,
}

/// Faixa de valores DMX com significado próprio (ex.: 10–19 = gobo 2).
#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Capability {
    pub min: u8,
    pub max: u8,
    pub name: String,
    /// Cor de referência (#rrggbb) em rodas de cor e macros.
    pub color: Option<String>,
}

#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Mode {
    pub name: String,
    /// Índices em `FixtureDef::channels`, na ordem dos endereços DMX.
    pub channels: Vec<u16>,
    /// Cabeças (ex.: pixels de uma barra de LED): posições em `channels` de cada cabeça.
    pub heads: Vec<Vec<u16>>,
}

#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Physical {
    pub width_mm: u32,
    pub height_mm: u32,
    pub depth_mm: u32,
    pub weight_kg: f32,
    pub power_w: u32,
    pub lens_min_deg: f32,
    pub lens_max_deg: f32,
    pub pan_max_deg: u32,
    pub tilt_max_deg: u32,
}

// ---------- Patch e grupos ----------

/// Fixture do projeto, já endereçada.
#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Fixture {
    pub id: Id,
    pub def_id: Id,
    pub mode: String,
    pub name: String,
    /// Cor da etiqueta (#rrggbb), usada nas strips e listas.
    pub color: String,
    pub universe: Id,
    /// Endereço inicial, de 1 a 512.
    pub address: u16,
}

/// Grade 2D de cabeças, usada pelo RGB Matrix.
#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct FixtureGroup {
    pub id: Id,
    pub name: String,
    pub width: u16,
    pub height: u16,
    pub cells: Vec<GroupCell>,
}

#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GroupCell {
    pub x: u16,
    pub y: u16,
    pub fixture: Id,
    pub head: u16,
}

/// Grupo de canais, como os "Channel Groups" do QLC+: canais de quaisquer fixtures, do mesmo
/// atributo ou misturados, controlados juntos por um fader (console, mesa simples) ou numa cena.
#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ChannelGroup {
    pub id: Id,
    pub name: String,
    pub channels: Vec<FixtureChannel>,
}

/// Um canal de uma fixture do projeto.
#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct FixtureChannel {
    pub fixture: Id,
    /// Posição do canal no modo da fixture, a partir de 0.
    pub channel: u16,
}

// ---------- In/Out ----------

#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DmxInterface {
    pub id: Id,
    pub kind: InterfaceKind,
    pub name: String,
    pub serial: String,
    pub connected: bool,
    /// Quadros DMX enviados por segundo.
    pub fps: f32,
    pub jitter_ms: f32,
    pub errors: u32,
}

#[derive(Serialize, Deserialize, TS, Clone, Copy, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum InterfaceKind {
    EnttecPro,
    OpenDmx,
    Dummy,
}

/// Controlador MIDI conectado. O que cada controle faz é definido no elemento do console (E11).
#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct MidiDevice {
    pub id: Id,
    pub name: String,
    /// Tem porta de saída, detectada pelo sistema: aceita retorno (LEDs, faders motorizados).
    /// Usado na edição do elemento do console, para oferecer o retorno.
    pub output: bool,
    pub connected: bool,
}

// ---------- Acesso remoto ----------

#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RemoteAccess {
    /// Aceita aparelhos da rede local (senão, só este computador).
    pub enabled: bool,
    pub pin: Option<String>,
    pub lan_url: Option<String>,
    pub clients: Vec<RemoteClient>,
}

#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RemoteClient {
    pub id: Id,
    pub name: String,
    pub address: String,
    pub this_device: bool,
}

// ---------- Operações (A2) ----------

/// Alterações que um cliente pede. O servidor valida, aplica e difunde o novo estado.
#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(tag = "op", rename_all = "camelCase", rename_all_fields = "camelCase")]
#[ts(export)]
pub enum Op {
    NewProject,
    OpenProject {
        path: String,
    },
    /// Sem `path`, salva no arquivo atual.
    SaveProject {
        path: Option<String>,
    },
    /// Adiciona `count` fixtures iguais; sem `address`, usa o próximo endereço livre.
    AddFixtures {
        def_id: Id,
        mode: String,
        name: String,
        color: String,
        count: u16,
        universe: Id,
        address: Option<u16>,
    },
    UpdateFixture {
        fixture: Fixture,
    },
    RemoveFixtures {
        ids: Vec<Id>,
    },
    /// Endereça as fixtures em sequência a partir de `start`, sem buracos.
    AutoAddress {
        ids: Vec<Id>,
        universe: Id,
        start: u16,
    },
    SaveFixtureGroup {
        group: FixtureGroup,
    },
    RemoveFixtureGroup {
        id: Id,
    },
    SaveChannelGroup {
        group: ChannelGroup,
    },
    RemoveChannelGroup {
        id: Id,
    },
    /// Adiciona ou substitui uma definição na biblioteca.
    SaveFixtureDef {
        def: FixtureDef,
    },
    /// Converte um `.qxf` do QLC+ para o formato próprio e o põe na biblioteca (F4).
    /// Resposta: [`ImportReport`].
    ImportQxf {
        file_name: String,
        content: String,
    },
    AddUniverse,
    UpdateUniverse {
        universe: Universe,
    },
    RemoveUniverse {
        id: Id,
    },
    SetRemoteAccess {
        enabled: bool,
        pin: Option<String>,
    },
}

/// Resultado da importação de um `.qxf`: o que foi convertido e o que precisa de atenção.
#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct ImportReport {
    pub def_id: Id,
    pub channels: u16,
    pub modes: u16,
    pub warnings: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn op_is_tagged_with_camel_case_fields() {
        let op = Op::AutoAddress {
            ids: vec!["f1".into()],
            universe: "u1".into(),
            start: 1,
        };
        assert_eq!(
            serde_json::to_value(op).unwrap(),
            json!({ "op": "autoAddress", "ids": ["f1"], "universe": "u1", "start": 1 })
        );
    }
}

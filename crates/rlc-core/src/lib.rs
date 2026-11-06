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
    /// O projeto veio da cópia de recuperação (alterações não salvas da última sessão).
    pub recovered: bool,
    /// Projetos da pasta, pelo nome, do mais recente para o mais antigo.
    pub projects: Vec<String>,
    /// Biblioteca de fixtures do usuário (fica fora do projeto).
    pub library: Vec<FixtureDef>,
    pub interfaces: Vec<DmxInterface>,
    pub midi_devices: Vec<MidiDevice>,
    pub remote: RemoteAccess,
    /// Criações disparadas pelo usuário, rodando agora (o que um chaser ou uma coleção dispara não
    /// entra).
    pub running: Vec<Id>,
    /// Passo atual de cada chaser (ou sequência) da lista `running`, vindo do motor.
    pub chaser_steps: Vec<CurrentStep>,
    /// Criações da lista `running` pausadas: os fades, os delays e os passos delas não andam (R1).
    pub paused: Vec<Id>,
    /// Criações que pararam por erro (um script de RGB Matrix que falhou): vale até a criação rodar de
    /// novo.
    pub failures: Vec<RunFailure>,
    /// Passo carregado de cada chaser na cue list: o próximo Go vai para ele (R1).
    pub loaded_steps: Vec<CurrentStep>,
    /// Mesa simples (não é salva no projeto).
    pub desk: Desk,
    pub master: Master,
    /// Valores ao vivo dos elementos do console (não são salvos no projeto).
    pub widget_values: Vec<WidgetValue>,
    pub stats: Stats,
    /// Padrão de teste da saída (página /dev/dmx); por cima da Mesa simples enquanto estiver ligado.
    pub test_pattern: Option<TestPattern>,
    /// Cena em prévia: os valores dela vão para a luz enquanto ela é editada (o programador, E12).
    pub preview: Option<Id>,
    /// O "Localizar": as fixtures acesas e quanto as outras ficam.
    pub highlight: Locate,
}

#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RunFailure {
    pub function: Id,
    pub message: String,
}

#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Locate {
    /// Fixtures acesas; vazio = o Localizar está desligado.
    pub fixtures: Vec<Id>,
    /// Intensidade (dimmer e cores) das outras fixtures, de 0 a 255 como o Grand Master: 255 não
    /// reduz nada e 0 deixa só as localizadas acesas.
    pub others: u8,
}

/// Padrões de teste gerados pelo motor em todos os canais, para conferir a saída DMX.
#[derive(Serialize, Deserialize, TS, Clone, Copy, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum TestPattern {
    /// Todos os canais sobem e descem juntos.
    Ramp,
    /// A mesma rampa, atrasada canal a canal: uma onda que corre pelos endereços.
    Chase,
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
    pub functions: Vec<Function>,
    pub folders: Vec<Folder>,
    pub console: Console,
    pub stage: Stage,
    /// Scripts do QLC+ importados para os RGB Matrix, dentro do projeto (como as fixtures, A4).
    pub scripts: Vec<RgbScript>,
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
    /// Efeitos de festa (no QLC+, Effect e Flower).
    Effect,
    Other,
}

/// O que o canal controla. Taxonomia própria; o importador .qxf converte os grupos e presets do QLC+.
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
    Lime,
    Indigo,
    Cyan,
    Magenta,
    Yellow,
    ColorWheel,
    ColorMacro,
    /// Correção de temperatura de cor (CTO, CTC, CTB).
    ColorTemperature,
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
    /// Valor do canal quando nada o controla (o `Default` do .qxf; ex.: strobo em "luz ativa").
    pub default: u8,
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

#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq, Default)]
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
    /// Com um grupo de fixtures, os canais são os `attributes` (sem os finos) das fixtures que estão
    /// nele: o servidor refaz a lista quando o grupo, as fixtures ou o patch mudam (N1).
    pub fixture_group: Option<Id>,
    pub attributes: Vec<Attribute>,
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

// ---------- Studio (no QLC+, "funções") ----------

#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Function {
    pub id: Id,
    pub name: String,
    /// Pasta do Studio (do mesmo tipo); `None` fica solta no tipo.
    pub folder: Option<Id>,
    pub priority: Priority,
    #[serde(flatten)]
    pub kind: FunctionKind,
}

/// Prioridade da criação na mistura (R6): num canal disputado, a maior ganha, também no dimmer e nas
/// cores (o valor dela vale, mesmo que seja menor). Na mesma prioridade, vale a regra de sempre: HTP
/// no dimmer e nas cores, LTP no resto.
#[derive(Serialize, Deserialize, TS, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "camelCase")]
pub enum Priority {
    Low,
    Normal,
    High,
}

/// Pasta do Studio: agrupa criações (e outras pastas) de um tipo só.
#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Folder {
    pub id: Id,
    pub name: String,
    pub kind: FunctionType,
    /// Pasta que contém esta (do mesmo tipo); `None` fica solta no tipo.
    pub parent: Option<Id>,
}

/// Os tipos de criação, iguais ao campo `type` de [`Function`].
#[derive(Serialize, Deserialize, TS, Clone, Copy, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum FunctionType {
    Scene,
    Chaser,
    Sequence,
    Collection,
    Efx,
    RgbMatrix,
    Phaser,
}

#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum FunctionKind {
    Scene(Scene),
    Chaser(Chaser),
    /// Chaser preso a uma cena: cada passo guarda os próprios valores dos canais dela.
    Sequence(Chaser),
    /// Funções disparadas juntas.
    Collection(Collection),
    Efx(Efx),
    RgbMatrix(RgbMatrix),
    Phaser(Phaser),
}

#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Scene {
    /// Valores próprios das fixtures; numa fixture de um grupo da cena, vencem os do grupo.
    pub values: Vec<SceneValue>,
    /// Grupos de fixtures da cena (N1): os valores de cada um valem para as fixtures que estão nele na
    /// hora de rodar. Com uma fixture em dois, vence o que vem depois.
    pub groups: Vec<SceneGroup>,
    /// Grupos de canais usados no editor e o nível de cada um (os canais também ficam em `values`, ou
    /// no grupo de fixtures que o grupo de canais acompanha).
    pub channel_groups: Vec<GroupLevel>,
    /// Espera antes do fade in e antes do fade out (R2).
    pub delay_in_ms: u32,
    pub fade_in_ms: u32,
    pub delay_out_ms: u32,
    pub fade_out_ms: u32,
    /// Curva dos dois fades.
    pub curve: Curve,
    /// Tipos de atributo com tempos próprios (R3); os que não estão aqui usam os da cena.
    pub attribute_times: Vec<AttributeTiming>,
}

/// Como o valor anda do começo ao fim de um fade (R2).
#[derive(Serialize, Deserialize, TS, Clone, Copy, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum Curve {
    Linear,
    /// Lento no começo, rápido no fim.
    EaseIn,
    /// Rápido no começo, lento no fim.
    EaseOut,
    /// Em S: lento nas duas pontas.
    EaseInOut,
}

/// Tempos de um tipo de atributo da cena, no lugar dos dela (a curva é a da cena).
#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AttributeTiming {
    pub kind: AttributeKind,
    pub delay_in_ms: u32,
    pub fade_in_ms: u32,
    pub delay_out_ms: u32,
    pub fade_out_ms: u32,
}

/// Tipos de atributo com tempos próprios na cena: as abas do editor. Intensidade é o dimmer; Cor, as
/// cores, as rodas e as macros de cor; Posição, o pan e o tilt; Efeitos, o shutter, o gobo, o prisma,
/// o zoom, o foco e a íris. O resto (velocidade, manutenção…) usa os tempos da cena.
#[derive(Serialize, Deserialize, TS, Clone, Copy, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum AttributeKind {
    Intensity,
    Color,
    Position,
    Effects,
}

#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SceneValue {
    pub fixture: Id,
    /// Posição do canal no modo da fixture, a partir de 0.
    pub channel: u16,
    pub value: u8,
}

#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SceneGroup {
    /// Grupo de fixtures.
    pub group: Id,
    pub values: Vec<AttributeValue>,
}

/// Valor de um tipo de canal, para fixtures de qualquer modelo: vale para o `index`-ésimo canal com o
/// atributo, na ordem do modo (os dois prismas do Beam 7R são 0 e 1; o Pan fino depois do Pan, 1).
#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AttributeValue {
    pub attribute: Attribute,
    pub index: u16,
    pub value: u8,
}

#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GroupLevel {
    pub group: Id,
    pub value: u8,
}

#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Chaser {
    /// Só em sequências: a cena que define os canais.
    pub scene: Option<Id>,
    pub steps: Vec<ChaserStep>,
    pub run_order: RunOrder,
    pub direction: Direction,
    /// Tempos em batidas do BPM global (milésimos de batida); senão, em milissegundos.
    pub beats: bool,
    pub fade_in_mode: SpeedMode,
    pub hold_mode: SpeedMode,
    pub fade_out_mode: SpeedMode,
    /// Tempos dos modos "comum". O delay de entrada e a curva vão com o fade in; o de saída, com o
    /// fade out.
    pub common: StepTiming,
    /// No Single Shot, o último passo fica aceso até o chaser ser parado (senão, apaga no fim).
    pub keep_last: bool,
}

#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ChaserStep {
    /// Função do passo; em sequências, a própria cena.
    pub function: Id,
    pub timing: StepTiming,
    pub note: String,
    /// Só em sequências: valores dos canais da cena neste passo.
    pub values: Vec<SceneValue>,
}

#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Timing {
    pub fade_in_ms: u32,
    /// Tempo parado depois do fade in; no EFX e no RGB Matrix, a duração de um ciclo ou passo.
    pub hold_ms: u32,
    pub fade_out_ms: u32,
}

/// Tempos de um passo de chaser (e os comuns): o delay e o fade da entrada, o hold, o delay e o fade
/// da saída e a curva dos fades (R2). O hold em [`INFINITE`] espera o Go.
#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct StepTiming {
    pub delay_in_ms: u32,
    pub fade_in_ms: u32,
    pub hold_ms: u32,
    pub delay_out_ms: u32,
    pub fade_out_ms: u32,
    pub curve: Curve,
}

/// Hold infinito (∞), como no QLC+: o passo só sai com o Go, o anterior, outro passo ou o parar (R1).
pub const INFINITE: u32 = u32::MAX;

#[derive(Serialize, Deserialize, TS, Clone, Copy, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum RunOrder {
    Loop,
    SingleShot,
    PingPong,
    Random,
}

#[derive(Serialize, Deserialize, TS, Clone, Copy, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum Direction {
    Forward,
    Backward,
}

/// De onde vem cada tempo dos passos: da própria função do passo, do tempo comum ou do passo.
#[derive(Serialize, Deserialize, TS, Clone, Copy, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum SpeedMode {
    Default,
    Common,
    PerStep,
}

#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Collection {
    pub functions: Vec<Id>,
}

#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Efx {
    /// O desenho de todos os itens, menos os que têm o próprio (F10d).
    #[serde(flatten)]
    pub shape: EfxShape,
    /// Soma o desenho à posição que as fixtures já têm (de uma cena, por exemplo).
    pub relative: bool,
    pub propagation: Propagation,
    pub fixtures: Vec<EfxFixture>,
    /// Sem a ordem aleatória, como no QLC+.
    pub run_order: RunOrder,
    pub direction: Direction,
    pub timing: Timing,
}

/// O desenho do EFX: o caminho, o tamanho, o centro, a rotação e a velocidade ao longo da volta.
#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct EfxShape {
    pub algorithm: EfxAlgorithm,
    /// Tamanho do desenho para cada lado do centro, de 0 a 127 (127 = o curso inteiro), como no QLC+.
    pub width: u8,
    pub height: u8,
    /// Centro do desenho, de 0 a 255 (a escala do pan e do tilt). No relativo, 127 não desloca.
    pub x_offset: u8,
    pub y_offset: u8,
    /// Graus.
    pub rotation: u16,
    /// Só no Lissajous (fases em graus).
    pub x_frequency: u8,
    pub y_frequency: u8,
    pub x_phase: u16,
    pub y_phase: u16,
    /// Só no desenho à mão: até 64 pontos (x, y) de -127 a 127, num caminho fechado; o tilt cresce
    /// para cima.
    pub path: Vec<(i8, i8)>,
    /// Como a fixture anda ao longo da volta, pela posição no desenho (Linear = constante).
    pub speed: Curve,
    /// Curva desenhada no lugar de `speed`: 16 pontos de 0 a 255 (o quanto do desenho já foi andado),
    /// do começo ao fim da volta, sem descer, de 0 a 255.
    pub speed_points: Option<Vec<u8>>,
}

#[derive(Serialize, Deserialize, TS, Clone, Copy, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum EfxAlgorithm {
    Circle,
    Eight,
    Line,
    Line2,
    Diamond,
    Square,
    SquareChoppy,
    SquareTrue,
    Leaf,
    Lissajous,
    /// Desenhado à mão (`path`).
    Freehand,
}

/// Como as fixtures dividem o desenho: juntas, uma depois da outra ou defasadas.
#[derive(Serialize, Deserialize, TS, Clone, Copy, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum Propagation {
    Parallel,
    Serial,
    Asymmetric,
}

#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct EfxFixture {
    #[serde(flatten)]
    pub target: EfxTarget,
    /// Num grupo, vale para todas as fixtures dele.
    pub reverse: bool,
    /// Graus.
    pub start_offset: u16,
    /// Só no grupo: graus que cada fixture começa depois da anterior, na ordem da grade (0 = juntas).
    /// Ausente nos arquivos de antes, vale 0.
    #[serde(default)]
    pub spread: u16,
    /// Desenho próprio do item, no lugar do geral (num grupo, vale para todas as fixtures dele).
    pub shape: Option<EfxShape>,
    /// Espelham o movimento em volta do centro do desenho, no pan e no tilt; `swap_axes` troca um
    /// eixo pelo outro (a fixture montada de lado). A troca vem antes do espelho.
    pub invert_pan: bool,
    pub invert_tilt: bool,
    pub swap_axes: bool,
}

/// Item da lista do EFX: uma fixture ou um grupo de fixtures (N1), que entra com as fixtures que estão
/// nele na hora de rodar, na ordem da grade.
#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum EfxTarget {
    Fixture(Id),
    Group(Id),
}

#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RgbMatrix {
    /// Grupo de fixtures (grade 2D) onde o efeito é desenhado.
    pub group: Option<Id>,
    pub algorithm: RgbAlgorithm,
    /// Que canais das cabeças recebem as cores.
    pub control: RgbControl,
    /// Cores (#rrggbb): a inicial e a final.
    pub colors: Vec<String>,
    /// Loop, Uma vez e Vai e volta (o RGB Matrix do QLC+ não tem Aleatória).
    pub run_order: RunOrder,
    pub direction: Direction,
    /// `hold_ms` é a duração de um passo.
    pub timing: Timing,
}

/// Efeito em loop sobre os atributos de um grupo de fixtures (P4, o "phaser" da grandMA): cada camada
/// leva um atributo por uma onda, e as fixtures do grupo andam a onda defasadas umas das outras.
#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Phaser {
    /// Grupo de fixtures; as fixtures com o atributo de cada camada andam a onda, na ordem da grade.
    pub group: Option<Id>,
    /// De 1 a 8 camadas, no máximo uma por atributo e tipo (absoluta ou relativa).
    pub layers: Vec<PhaserLayer>,
    /// Graus de atraso de uma fixture para a próxima, de -360 a 360 (0 = juntas; negativo = a onda
    /// anda para o outro lado).
    pub spread: i16,
    /// O ciclo em batidas do BPM master: `hold_ms` vale em milésimos de batida.
    pub beats: bool,
    /// `hold_ms` é a duração de um ciclo.
    pub timing: Timing,
}

/// Uma onda sobre um atributo.
#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PhaserLayer {
    /// Só os contínuos: dimmer, as cores, pan, tilt, zoom, foco, íris e rotação do gobo.
    pub attribute: Attribute,
    pub shape: WaveShape,
    /// Só no pulso: a parte do ciclo em que ele fica no alto, de 1 a 99%.
    pub width: u8,
    /// Piso e teto da onda, de 0 a 255 (a escala DMX). No relativo valem como deslocamentos, com o
    /// zero em 127, como no EFX.
    pub low: u8,
    pub high: u8,
    /// Graus de atraso da camada no ciclo, de 0 a 359.
    pub phase: u16,
    /// Soma a onda ao que está embaixo, em vez de assumir o canal.
    pub relative: bool,
}

/// A forma da onda no ciclo: o seno, o triângulo e o dente de serra para cima partem do piso; o
/// dente de serra para baixo e o pulso, do teto.
#[derive(Serialize, Deserialize, TS, Clone, Copy, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum WaveShape {
    Sine,
    Triangle,
    /// Sobe do piso ao teto e volta de uma vez.
    SawUp,
    /// Desce do teto ao piso e volta de uma vez.
    SawDown,
    /// No teto por `width`% do ciclo e no piso no resto.
    Pulse,
}

/// O que desenha o mapa de cores de cada passo do RGB Matrix.
#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum RgbAlgorithm {
    /// Preenche a grade com a cor inicial: um passo só.
    Plain,
    /// Um texto: duas cores (o texto muda de cor de um passo para o outro) e os estilos do QLC+. A fonte
    /// é de pontos 5 × 7, sem acentos (as letras acentuadas saem sem o acento). `x` e `y` deslocam o
    /// texto em células.
    Text {
        text: String,
        style: ScrollStyle,
        x: i16,
        y: i16,
    },
    /// Uma imagem do projeto (sem cor: o pixel vem dela). `x` e `y` deslocam a imagem em células, e ela
    /// dá a volta (nas rolagens, anda uma célula por passo).
    Image {
        image: Option<RgbImage>,
        style: ScrollStyle,
        x: i16,
        y: i16,
    },
    /// Um script do projeto, com o valor de cada propriedade dele (as que faltam valem o padrão).
    Script {
        script: Id,
        properties: Vec<PropertyValue>,
    },
}

/// Como o texto ou a imagem passa pela grade: parado (no texto, uma letra por passo, no centro), ou
/// rolando para a esquerda ou para cima, uma célula por passo.
#[derive(Serialize, Deserialize, TS, Clone, Copy, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum ScrollStyle {
    Static,
    Horizontal,
    Vertical,
}

/// Imagem de um RGB Matrix, guardada no projeto: os pixels em RGBA (4 bytes cada, linha a linha) em
/// base64. A tela a reduz a no máximo 128 × 128 ao escolher o arquivo; o limite é 256 KB no total.
#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RgbImage {
    pub name: String,
    pub width: u16,
    pub height: u16,
    pub pixels: String,
}

/// Script do QLC+ (JavaScript ES5, API 1 a 3) importado para o projeto. O texto vai junto: o projeto
/// nunca depende de um arquivo de fora. `id` é o `name` do script, e vale a última importação.
#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct RgbScript {
    pub id: Id,
    pub name: String,
    pub author: String,
    /// Versão da API do QLC+ (1, 2 ou 3).
    pub api: u8,
    /// Quantas cores o script usa (de 0 a 5; no API 3, vêm de `rgbMapSetColors`).
    pub accepts: u8,
    pub properties: Vec<ScriptProperty>,
    pub source: String,
}

/// Uma propriedade que o script declara em `algo.properties` (API 2 em diante), como o
/// `RGBScriptProperty` do QLC+: o valor vai ao método `write` como texto, e `read` o devolve.
#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ScriptProperty {
    pub name: String,
    pub display: String,
    pub kind: PropertyKind,
    pub write: String,
    pub read: String,
    /// O que `read` devolvia na importação.
    pub default: String,
}

#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum PropertyKind {
    List { values: Vec<String> },
    Range { min: i32, max: i32 },
    Float,
    Text,
}

#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PropertyValue {
    pub name: String,
    pub value: String,
}

/// Um passo do RGB Matrix desenhado: quantos passos o algoritmo tem e a cor (0xRRGGBB) de cada
/// célula da grade, `map[linha][coluna]`.
#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct MatrixPreview {
    pub steps: u32,
    pub map: Vec<Vec<u32>>,
}

/// Que canais de cada cabeça o RGB Matrix controla. No RGB, o dimmer, o branco, o âmbar e o UV não
/// são tocados (como no QLC+); nos outros, o canal recebe o cinza da cor.
#[derive(Serialize, Deserialize, TS, Clone, Copy, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum RgbControl {
    /// Vermelho, verde e azul; sem eles, ciano, magenta e amarelo.
    Rgb,
    Dimmer,
    White,
    Amber,
    Uv,
}

// ---------- Virtual Console (E11) ----------

#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Console {
    pub pages: Vec<VcPage>,
    pub widgets: Vec<VcWidget>,
}

#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct VcPage {
    pub id: Id,
    pub name: String,
}

/// Elemento do console. É um só para os três formatos de tela (VC-2): função, cor e entradas
/// valem em todos, e cada formato guarda só onde ele fica.
#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct VcWidget {
    pub id: Id,
    pub page: Id,
    /// Moldura onde está; a posição passa a contar a partir do canto da moldura.
    pub frame: Option<Id>,
    pub caption: String,
    /// Cor do destaque (#rrggbb).
    pub color: String,
    pub layouts: VcLayouts,
    /// Controles MIDI e teclas que acionam o elemento (VC-1).
    pub inputs: Vec<WidgetInput>,
    #[serde(flatten)]
    pub kind: VcWidgetKind,
}

/// Posição em cada formato de tela; `None` = o elemento não aparece nesse formato.
#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct VcLayouts {
    pub phone: Option<Rect>,
    pub tablet: Option<Rect>,
    pub desktop: Option<Rect>,
}

/// Retângulo em células da grade do formato.
#[derive(Serialize, Deserialize, TS, Clone, Copy, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Rect {
    pub x: u16,
    pub y: u16,
    pub w: u16,
    pub h: u16,
}

#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct WidgetInput {
    pub control: InputControl,
    pub midi: Option<MidiControl>,
    /// Tecla ou combinação (ex.: "F1", "Shift+A").
    pub key: Option<String>,
}

/// Qual parte do elemento a entrada aciona.
#[derive(Serialize, Deserialize, TS, Clone, Copy, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum InputControl {
    /// O botão, o fader ou o Go da cue list.
    Main,
    Previous,
    Next,
    Stop,
    Pause,
    Tap,
    X,
    Y,
}

#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct MidiControl {
    pub device: Id,
    /// Canal MIDI, de 1 a 16.
    pub channel: u8,
    pub message: MidiMessage,
    /// Nota ou número do controle.
    pub number: u8,
    /// Devolve o estado ao controlador (LED, fader motorizado).
    pub feedback: bool,
}

#[derive(Serialize, Deserialize, TS, Clone, Copy, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum MidiMessage {
    Note,
    ControlChange,
    ProgramChange,
    PitchBend,
}

#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum VcWidgetKind {
    Button(VcButton),
    Slider(VcSlider),
    CueList(VcCueList),
    XyPad(VcXyPad),
    /// Tempos ao vivo (VC-3): ajusta fade in, fade out e duração das criações da lista.
    SpeedDial(VcSpeedDial),
    Frame(VcFrame),
    Clock(VcClock),
    /// Controle de um RGB Matrix: cores, algoritmo e intensidade.
    Animation(VcAnimation),
    /// Texto na cor do elemento.
    Label(VcLabel),
}

#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct VcButton {
    pub action: ButtonAction,
    /// Criação disparada (ação `function`).
    pub function: Option<Id>,
    /// Fixtures e grupos de fixtures apagados (ação `blackoutFixtures`).
    pub fixtures: Vec<Id>,
    pub groups: Vec<Id>,
    pub mode: ButtonMode,
    pub highlight: Highlight,
    /// Ícone da biblioteca (lucide, ex.: "Camera"); `None` = só o texto.
    pub icon: Option<String>,
}

#[derive(Serialize, Deserialize, TS, Clone, Copy, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum ButtonAction {
    Function,
    /// "Só esta" (o Kill da grandMA): liga a criação e para todas as outras que estão rodando.
    Exclusive,
    Blackout,
    BlackoutFixtures,
    StopAll,
    /// O passo anterior do chaser (ou sequência) do botão.
    Previous,
    /// Pausa ou continua a criação do botão.
    Pause,
}

#[derive(Serialize, Deserialize, TS, Clone, Copy, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum ButtonMode {
    Toggle,
    /// Enquanto pressionado: acende na hora, a 100% (sem o submaster), e apaga na hora ao soltar.
    Flash,
    /// Enquanto pressionado, com os fades da criação e o submaster.
    Temp,
}

/// Aceso: o botão inteiro (`face`) ou só o filete de LED.
#[derive(Serialize, Deserialize, TS, Clone, Copy, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum Highlight {
    Face,
    Led,
}

#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct VcSlider {
    pub mode: SliderMode,
    /// Nível: grupos de canais controlados.
    pub channel_groups: Vec<Id>,
    /// Playback: criação que roda com a intensidade do slider.
    pub function: Option<Id>,
    /// Velocidade: criações que têm os tempos acelerados ou desacelerados.
    pub functions: Vec<Id>,
    /// Aparência de knob em vez de fader.
    pub knob: bool,
}

#[derive(Serialize, Deserialize, TS, Clone, Copy, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum SliderMode {
    Level,
    Playback,
    /// Escala a intensidade das criações dos elementos da moldura (fora de molduras, da página).
    Submaster,
    /// O "Rate" da grandMA: de ×0,25 a ×4 nos tempos em segundos das criações da lista.
    Rate,
}

#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct VcCueList {
    /// Chaser ou sequência.
    pub chaser: Option<Id>,
}

#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct VcXyPad {
    pub fixtures: Vec<Id>,
}

#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct VcSpeedDial {
    pub functions: Vec<Id>,
    pub fade_in: bool,
    pub fade_out: bool,
    /// Duração (hold do chaser, ciclo do EFX, passo do RGB Matrix), com tap.
    pub hold: bool,
    /// Segue o BPM master: os tempos ficam em batidas, e o tap é o do BPM. Senão, em segundos.
    pub beats: bool,
}

#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct VcFrame {
    /// Só uma criação da moldura roda por vez.
    pub solo: bool,
}

#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct VcClock {
    pub mode: ClockMode,
    pub countdown_s: u32,
}

#[derive(Serialize, Deserialize, TS, Clone, Copy, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum ClockMode {
    Clock,
    Stopwatch,
    Countdown,
}

#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct VcAnimation {
    pub matrix: Option<Id>,
}

#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct VcLabel {
    /// Cores (#rrggbb) do fundo e da borda; `None` = sem.
    pub background: Option<String>,
    pub border: Option<String>,
}

// ---------- 3D View ----------

/// Palco do preview: tamanho em metros e posição de cada fixture.
#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Stage {
    pub width: f32,
    pub depth: f32,
    pub height: f32,
    pub fixtures: Vec<FixturePosition>,
    /// Elementos de palco (truss, caixas, pessoas…), que só complementam a prévia.
    pub elements: Vec<StageElement>,
}

/// Posição (metros: x para a direita, y para cima, z para a plateia) e rotação (graus).
#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct FixturePosition {
    pub fixture: Id,
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub rotation_x: f32,
    pub rotation_y: f32,
    pub rotation_z: f32,
}

/// Elemento de palco, com posição e rotação como as das fixtures.
#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct StageElement {
    pub id: Id,
    pub kind: StageElementKind,
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub rotation_x: f32,
    pub rotation_y: f32,
    pub rotation_z: f32,
}

/// Tipos de elemento de palco (no truss, o comprimento em centímetros).
#[derive(Serialize, Deserialize, TS, Clone, Copy, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum StageElementKind {
    Truss100,
    Truss150,
    Truss300,
    Riser,
    Stairs,
    LedPanel,
    ProjectionScreen,
    Subwoofer,
    Speaker,
    Monitor,
    MicStand,
    Person,
    Drums,
    Keyboard,
    Pulpit,
    Plant,
}

// ---------- Execução ----------

#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Desk {
    pub levels: Vec<DeskLevel>,
    pub groups: Vec<GroupLevel>,
}

#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DeskLevel {
    pub universe: Id,
    /// Canal DMX, de 1 a 512.
    pub channel: u16,
    pub value: u8,
}

/// Seção master do console (E12) e o BPM global.
#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Master {
    /// Grand Master: reduz os canais de intensidade (dimmer e cores).
    pub grand_master: u8,
    pub blackout: bool,
    pub bpm: u16,
}

/// Valor ao vivo de um elemento do console. Conforme o tipo: slider e animation `[nível]` (0 a 255;
/// na velocidade, ×1 em 128), XY pad `[x, y]`, speed dial `[fade in, fade out, duração]` em ms (ou em
/// milésimos de batida, quando ele segue o BPM), botão de blackout de fixtures `[1]` quando ligado e
/// botão de Flash ou Temp `[1]` enquanto segura a criação que ele mesmo acendeu.
#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct WidgetValue {
    pub widget: Id,
    pub value: Vec<u32>,
}

/// Um passo de um chaser: o atual (vindo do motor) ou o carregado na cue list.
#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CurrentStep {
    pub chaser: Id,
    /// Posição do passo, a partir de 0.
    pub step: u16,
}

/// Pedido da cue list a um chaser: o próximo passo, o anterior ou um escolhido.
#[derive(Serialize, Deserialize, TS, Clone, Copy, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum StepAction {
    Next,
    Previous,
    /// Posição do passo, a partir de 0.
    Go(u16),
}

/// Dados do app e da máquina para o Monitoramento.
#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Stats {
    pub cpu_percent: f32,
    pub memory_mb: f32,
    /// Duração do último tick do motor (o limite é 20 ms, A1).
    pub tick_ms: f32,
    pub uptime_s: u32,
}

// ---------- In/Out ----------

#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DmxInterface {
    /// `dummy`, ou `ftdi-` e o número de série da interface USB.
    pub id: Id,
    /// Protocolo em uso: escolhido à mão ou detectado (nome USB e, na Open DMX, a pergunta da Pro).
    pub kind: InterfaceKind,
    /// O tipo foi escolhido à mão no In/Out (fica guardado no `settings.json`).
    pub kind_forced: bool,
    /// Nome gravado na interface.
    pub name: String,
    pub serial: String,
    /// Ligada ao computador.
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
    /// O mesmo id que o cliente recebe no `hello`: é assim que ele se reconhece na lista.
    pub id: Id,
    pub name: String,
    pub address: String,
}

// ---------- Operações (A2) ----------

/// Alterações que um cliente pede. O servidor valida, aplica e difunde o novo estado.
#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(tag = "op", rename_all = "camelCase", rename_all_fields = "camelCase")]
#[ts(export)]
pub enum Op {
    NewProject,
    /// Abre um projeto da pasta, pelo nome.
    OpenProject {
        name: String,
    },
    /// Sem `name`, salva no arquivo atual; com `name`, é o "Salvar como" (um nome novo na pasta).
    SaveProject {
        name: Option<String>,
    },
    /// Desfaz a última alteração do projeto ou da biblioteca, de qualquer aparelho.
    Undo,
    Redo,
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
    /// Leva o grupo para a posição `to` da lista (a partir de 0). A ordem vale em todas as telas.
    MoveFixtureGroup {
        id: Id,
        to: usize,
    },
    SaveChannelGroup {
        group: ChannelGroup,
    },
    RemoveChannelGroup {
        id: Id,
    },
    /// Leva o grupo de canais para a posição `to` da lista (a partir de 0).
    MoveChannelGroup {
        id: Id,
        to: usize,
    },
    SaveFunction {
        function: Function,
    },
    /// Remove a função e as referências a ela (passos de chaser, coleções).
    RemoveFunction {
        id: Id,
    },
    SaveFolder {
        folder: Folder,
    },
    /// Remove a pasta; as criações e as pastas dela passam para a pasta de cima (ou ficam soltas no tipo).
    RemoveFolder {
        id: Id,
    },
    SetFunctionRunning {
        id: Id,
        running: bool,
    },
    /// Pausa uma criação rodando (os fades, os delays e os passos dela param onde estão) ou a
    /// continua de onde parou.
    SetFunctionPaused {
        id: Id,
        paused: bool,
    },
    /// Cue list: leva um chaser (ou sequência) a outro passo. Parado, ele passa a rodar: "próximo"
    /// começa no primeiro passo (ou no carregado) e "anterior", no último. Pausado, ele continua.
    StepChaser {
        id: Id,
        action: StepAction,
    },
    /// Cue list: o passo do próximo Go, sem disparar; `None` desfaz.
    LoadStep {
        id: Id,
        step: Option<u16>,
    },
    /// Para todas as criações rodando, cada uma com o seu fade out (Studio e Stop All do console).
    StopAllFunctions,
    /// Pânico: para todas as criações na hora, sem fade out. Com `all` (o segundo toque), também zera
    /// a mesa simples inteira.
    Panic {
        all: bool,
    },
    SetDeskLevel {
        universe: Id,
        channel: u16,
        value: u8,
    },
    SetDeskGroupLevel {
        group: Id,
        value: u8,
    },
    /// Zera a mesa simples: os canais de um universo ou, sem `universe`, os grupos de canais.
    ResetDesk {
        universe: Option<Id>,
    },
    SaveConsolePage {
        page: VcPage,
    },
    /// Remove a página e os elementos dela.
    RemoveConsolePage {
        id: Id,
    },
    /// Grava vários elementos de uma vez (mover, alinhar, colar).
    SaveWidgets {
        widgets: Vec<VcWidget>,
    },
    /// Remove os elementos; os de dentro de uma moldura removida vão junto.
    RemoveWidgets {
        ids: Vec<Id>,
    },
    SetWidgetValue {
        widget: Id,
        value: Vec<u32>,
    },
    /// Uma parte de um elemento do console foi apertada (`down`) ou solta: o servidor aplica o que o
    /// elemento faz (A2). Vem do toque, das teclas e, na F12, do MIDI.
    PressWidget {
        widget: Id,
        control: InputControl,
        down: bool,
    },
    SetGrandMaster {
        value: u8,
    },
    SetBlackout {
        on: bool,
    },
    /// BPM do tap: a batida do motor recomeça na hora dele.
    SetBpm {
        bpm: u16,
    },
    /// BPM do fader do master: muda a velocidade sem recomeçar a batida.
    SetTempo {
        bpm: u16,
    },
    SaveStage {
        stage: Stage,
    },
    /// Adiciona ou substitui uma definição na biblioteca.
    SaveFixtureDef {
        def: FixtureDef,
    },
    /// Tira a definição da biblioteca; o projeto continua com a cópia dele (A4).
    RemoveFixtureDef {
        id: Id,
    },
    /// Converte um `.qxf` do QLC+ para o formato próprio e o põe na biblioteca, no lugar de uma
    /// definição com o mesmo fabricante e modelo. Resposta: [`ImportReport`].
    ImportQxf {
        file_name: String,
        content: String,
    },
    /// Importa um script de RGB Matrix do QLC+ (JavaScript) para o projeto, no lugar do que tem o
    /// mesmo nome. Resposta: o [RgbScript]. Um texto que não é script, ou que falha, é recusado.
    ImportScript {
        file_name: String,
        source: String,
    },
    /// Tira um script do projeto, se nenhum RGB Matrix o usa.
    RemoveScript {
        id: Id,
    },
    /// O mapa de cores de um passo do RGB Matrix salvo (a prévia do editor é a saída de verdade).
    /// Resposta: [`MatrixPreview`]. O passo passa do último volta ao primeiro.
    PreviewMatrix {
        id: Id,
        step: u32,
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
    /// Liga ou desliga o padrão de teste da saída.
    SetTestPattern {
        pattern: Option<TestPattern>,
    },
    /// Tipo de uma interface USB escolhido à mão; `None` volta para o automático.
    SetInterfaceKind {
        interface: Id,
        kind: Option<InterfaceKind>,
    },
    /// Liga a prévia de uma cena (no lugar de outra) ou, sem `function`, desliga.
    SetPreview {
        function: Option<Id>,
    },
    /// Fixtures acesas pelo "Localizar", com a intensidade das outras; sem fixtures, apaga.
    SetHighlight {
        fixtures: Vec<Id>,
        others: u8,
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

    #[test]
    fn function_has_type_beside_common_fields() {
        let function = Function {
            id: "c1".into(),
            name: "Tudo".into(),
            folder: None,
            priority: Priority::Normal,
            kind: FunctionKind::Collection(Collection {
                functions: vec!["s1".into()],
            }),
        };
        let value = serde_json::to_value(&function).unwrap();
        assert_eq!(
            value,
            json!({ "id": "c1", "name": "Tudo", "folder": null, "priority": "normal", "type": "collection", "functions": ["s1"] })
        );
        assert_eq!(serde_json::from_value::<Function>(value).unwrap(), function);
    }

    #[test]
    fn widget_has_type_beside_common_fields() {
        let widget = VcWidget {
            id: "w1".into(),
            page: "p1".into(),
            frame: None,
            caption: "Palco".into(),
            color: "#ff8000".into(),
            layouts: VcLayouts {
                phone: None,
                tablet: None,
                desktop: Some(Rect {
                    x: 0,
                    y: 0,
                    w: 4,
                    h: 1,
                }),
            },
            inputs: vec![],
            kind: VcWidgetKind::Label(VcLabel {
                background: Some("#402000".into()),
                border: None,
            }),
        };
        let value = serde_json::to_value(&widget).unwrap();
        assert_eq!(value["type"], "label");
        assert_eq!(value["background"], "#402000");
        assert_eq!(serde_json::from_value::<VcWidget>(value).unwrap(), widget);
    }
}

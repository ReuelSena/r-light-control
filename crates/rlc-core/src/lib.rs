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
    /// Funções rodando agora.
    pub running: Vec<Id>,
    /// Mesa simples (não é salva no projeto).
    pub desk: Desk,
    pub master: Master,
    /// Valores ao vivo dos elementos do console (não são salvos no projeto).
    pub widget_values: Vec<WidgetValue>,
    pub stats: Stats,
    /// Padrão de teste da saída (página /dev/dmx); por cima da Mesa simples enquanto estiver ligado.
    pub test_pattern: Option<TestPattern>,
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
    #[serde(flatten)]
    pub kind: FunctionKind,
}

/// Pasta do Studio: agrupa criações de um tipo só.
#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Folder {
    pub id: Id,
    pub name: String,
    pub kind: FunctionType,
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
}

#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Scene {
    pub values: Vec<SceneValue>,
    /// Grupos de canais usados no editor e o nível de cada um (os canais também ficam em `values`).
    pub channel_groups: Vec<GroupLevel>,
    pub fade_in_ms: u32,
    pub fade_out_ms: u32,
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
    /// Tempos dos modos "comum".
    pub common: Timing,
}

#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ChaserStep {
    /// Função do passo; em sequências, a própria cena.
    pub function: Id,
    pub timing: Timing,
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
    pub algorithm: EfxAlgorithm,
    /// Tamanho e centro do desenho, de 0 a 255 (a escala do pan e do tilt).
    pub width: u8,
    pub height: u8,
    pub x_offset: u8,
    pub y_offset: u8,
    /// Graus.
    pub rotation: u16,
    /// Só no Lissajous (fases em graus).
    pub x_frequency: u8,
    pub y_frequency: u8,
    pub x_phase: u16,
    pub y_phase: u16,
    /// Soma o desenho à posição que as fixtures já têm (de uma cena, por exemplo).
    pub relative: bool,
    pub propagation: Propagation,
    pub fixtures: Vec<EfxFixture>,
    pub run_order: RunOrder,
    pub direction: Direction,
    pub timing: Timing,
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
    pub fixture: Id,
    pub reverse: bool,
    /// Graus.
    pub start_offset: u16,
}

#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RgbMatrix {
    /// Grupo de fixtures (grade 2D) onde o efeito é desenhado.
    pub group: Option<Id>,
    /// Algoritmo nativo ou script do QLC+ (F9).
    pub algorithm: String,
    /// Cores (#rrggbb): a inicial e a final.
    pub colors: Vec<String>,
    pub run_order: RunOrder,
    pub direction: Direction,
    pub timing: Timing,
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
    /// O botão, o fader ou o play da lista.
    Main,
    Previous,
    Next,
    Stop,
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
    Blackout,
    BlackoutFixtures,
    StopAll,
}

#[derive(Serialize, Deserialize, TS, Clone, Copy, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum ButtonMode {
    Toggle,
    Flash,
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
    /// Aparência de knob em vez de fader.
    pub knob: bool,
}

#[derive(Serialize, Deserialize, TS, Clone, Copy, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum SliderMode {
    Level,
    Playback,
    /// Escala as criações dos botões da mesma moldura.
    Submaster,
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

/// Valor ao vivo de um elemento do console. Conforme o tipo: slider e animation `[nível]`,
/// XY pad `[x, y]`, speed dial `[fade in, fade out, duração]` em ms, cue list `[passo]`,
/// botão de blackout de fixtures `[1]` quando ligado.
#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct WidgetValue {
    pub widget: Id,
    pub value: Vec<u32>,
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
    /// Remove a pasta; as criações dela ficam soltas no tipo.
    RemoveFolder {
        id: Id,
    },
    SetFunctionRunning {
        id: Id,
        running: bool,
    },
    /// Para todas as criações rodando (Studio e Stop All do console).
    StopAllFunctions,
    /// Pânico: para todas as criações e zera a mesa simples inteira.
    Panic,
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
    SetGrandMaster {
        value: u8,
    },
    SetBlackout {
        on: bool,
    },
    SetBpm {
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
            kind: FunctionKind::Collection(Collection {
                functions: vec!["s1".into()],
            }),
        };
        let value = serde_json::to_value(&function).unwrap();
        assert_eq!(
            value,
            json!({ "id": "c1", "name": "Tudo", "folder": null, "type": "collection", "functions": ["s1"] })
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

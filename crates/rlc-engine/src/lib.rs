//! Motor (A1): uma thread a 50 Hz com prazo absoluto, dona do estado de execução. Os comandos chegam
//! por canal e são aplicados no começo de cada tick, e cada tick publica o quadro de cada universo.
//! Camadas de cada universo, de baixo para cima: o valor padrão dos canais, as criações rodando, a
//! prévia (a cena em edição), os EFX relativos, a Mesa simples, o "Localizar" (que também deixa a
//! intensidade das outras fixtures mais fraca) e, por fim, o Grand Master e o blackout. O padrão de
//! teste substitui as camadas de baixo.
//! Cada cena acesa por uma criação (sozinha, no passo de um chaser ou numa coleção) ganha um fader, e
//! cada fixture de um EFX também. Os faders se misturam como no QLC+: nos canais de intensidade
//! (dimmer e cores) vence o maior valor (HTP); nos outros, o fader iniciado por último (LTP).

mod chaser;
mod efx;

use std::sync::Arc;
use std::sync::mpsc::{self, Sender, TryRecvError};
use std::thread;
use std::time::{Duration, Instant};

use arc_swap::ArcSwap;
use rlc_core::{DeskLevel, Direction, Efx, Id, RunOrder, StepAction, TestPattern};

use chaser::Runner;
use efx::EfxRunner;

/// Período do tick: 50 Hz, como no QLC+.
pub const TICK: Duration = Duration::from_millis(20);

/// Quadro DMX de um universo.
pub type Frame = [u8; 512];

/// O que o motor publica a cada tick.
#[derive(Default)]
pub struct Frames {
    /// Universos na ordem do projeto.
    pub universes: Vec<(Id, Frame)>,
    /// Quanto o último tick levou para calcular (o limite é o período).
    pub tick_time: Duration,
    /// Passo atual de cada chaser disparado pelo usuário.
    pub chasers: Vec<(Id, u16)>,
    /// Criações que terminaram sozinhas (o Single Shot que apagou no fim).
    pub done: Vec<Id>,
}

impl Frames {
    pub fn get(&self, universe: &str) -> Option<&Frame> {
        self.universes
            .iter()
            .find(|(id, _)| id == universe)
            .map(|(_, f)| f)
    }
}

/// Um universo do projeto, do jeito que o motor precisa dele.
pub struct UniversePatch {
    pub id: Id,
    /// Canais de intensidade (dimmer e cores): as cenas misturam pelo maior valor, e o Grand Master e
    /// o blackout só agem neles.
    pub intensity: Box<[bool; 512]>,
    /// Canais que fazem fade. Os outros (rodas, gobo, strobo…) mudam de uma vez.
    pub fades: Box<[bool; 512]>,
    /// Valor de cada canal quando nada o controla: a base do quadro.
    pub defaults: Box<Frame>,
}

/// Uma criação disparada pelo usuário, já resolvida pelo servidor (o motor não conhece o projeto).
pub struct Run {
    pub id: Id,
    pub program: Program,
    /// Multiplica a intensidade (dimmer e cores) de tudo o que ela acende: o slider de playback e os
    /// submasters (255 = cheia).
    pub intensity: u8,
}

/// O que uma criação acende.
#[derive(Clone)]
pub enum Program {
    Scene(SceneProgram),
    Chaser(ChaserProgram),
    /// Criações disparadas juntas (a coleção).
    Collection(Vec<Program>),
    Efx(EfxProgram),
}

/// Uma cena nos endereços DMX (o motor não conhece fixtures), com os fades que valem. Um fade em
/// batidas vira tempo pelo BPM de quando ele começa.
#[derive(Clone)]
pub struct SceneProgram {
    pub levels: Vec<SceneLevel>,
    pub fade_in: Span,
    pub fade_out: Span,
}

#[derive(Clone)]
pub struct ChaserProgram {
    pub steps: Vec<Step>,
    pub run_order: RunOrder,
    pub direction: Direction,
    /// No Single Shot, o último passo fica aceso até o chaser ser parado.
    pub keep_last: bool,
}

/// Um EFX com os endereços do pan e do tilt de cada fixture e os tempos que valem agora.
#[derive(Clone)]
pub struct EfxProgram {
    /// O desenho, a propagação, a ordem e o sentido, como no projeto. As fixtures são só as que têm
    /// pan ou tilt, e os tempos que valem são os de baixo.
    pub efx: Efx,
    /// Pan e tilt de cada fixture de `efx.fixtures`, na mesma ordem, nos endereços DMX (o valor não
    /// conta). Sem um dos dois, a fixture só anda no outro.
    pub heads: Vec<[Option<SceneLevel>; 2]>,
    /// Duração de uma volta do desenho.
    pub cycle: Span,
    pub fade_in: Span,
    pub fade_out: Span,
}

/// Um passo: o que ele dispara e os tempos dele. Sem fade, vale o da criação do passo; com fade, ele
/// vale para tudo o que o passo dispara (o "override" do QLC+). O passo dura o fade in mais o hold;
/// com o hold em batidas, ele troca nas batidas.
#[derive(Clone)]
pub struct Step {
    pub program: Program,
    pub fade_in: Option<Span>,
    pub hold: Span,
    pub fade_out: Option<Span>,
}

/// Tempo em milissegundos ou, nos chasers em batidas, em milésimos de batida.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Span {
    Ms(u32),
    Beats(u32),
}

impl Span {
    /// Em tempo, pelo intervalo entre as batidas de agora.
    fn duration(self, beat: Duration) -> Duration {
        match self {
            Span::Ms(ms) => Duration::from_millis(ms.into()),
            Span::Beats(beats) => beat * beats / 1000,
        }
    }
}

/// Valor de um canal de uma cena. Com o canal fino, o valor tem 16 bits (o grosso no byte de cima).
#[derive(Clone, PartialEq, Debug)]
pub struct SceneLevel {
    pub universe: Id,
    /// Canais DMX, de 1 a 512.
    pub channel: u16,
    pub fine: Option<u16>,
    pub value: u16,
}

/// O "Localizar": os valores das fixtures localizadas e quanto as outras ficam acesas.
#[derive(Default)]
pub struct Highlight {
    /// Valores das fixtures localizadas, já nos endereços DMX.
    pub levels: Vec<DeskLevel>,
    /// Todos os canais (universo e endereço) das fixtures localizadas: a intensidade deles não é
    /// reduzida. Vazio = nada localizado.
    pub located: Vec<(Id, u16)>,
    /// Intensidade das outras fixtures, como o Grand Master (255 não reduz nada).
    pub others: u8,
}

pub enum Command {
    /// Universos do projeto, na ordem.
    Patch(Vec<UniversePatch>),
    /// Criações disparadas pelo usuário, na ordem de início.
    Running(Vec<Run>),
    /// Para tudo na hora, sem fade out (pânico, "Novo" e "Abrir").
    Cut,
    /// Leva um chaser disparado pelo usuário a outro passo (a cue list).
    Step(Id, StepAction),
    /// BPM global. A batida recomeça agora: no tap, ela cai no último toque.
    Bpm(u16),
    /// Muda o BPM sem recomeçar a batida (o fader do master): a parte da batida que falta continua a
    /// mesma, e os chasers e EFX em batidas seguem sem salto.
    Tempo(u16),
    /// Níveis da Mesa simples de todos os universos.
    Desk(Vec<DeskLevel>),
    /// Valores da cena em prévia, já nos endereços DMX.
    Preview(Vec<DeskLevel>),
    /// O "Localizar", acima da Mesa simples.
    Highlight(Highlight),
    /// Canais (universo e endereço) das fixtures apagadas pelos botões de blackout de fixtures: como o
    /// blackout, só a intensidade deles vai a zero.
    FixtureBlackout(Vec<(Id, u16)>),
    GrandMaster(u8),
    Blackout(bool),
    Test(Option<TestPattern>),
}

/// Último quadro publicado, trocado sem trava: as saídas e o servidor leem no próprio ritmo e o
/// motor nunca espera por eles. Se o motor parar, o último quadro continua valendo (A1).
pub type Published = Arc<ArcSwap<Frames>>;

pub struct Engine {
    commands: Sender<Command>,
    pub frames: Published,
}

impl Engine {
    pub fn spawn() -> Self {
        let (commands, inbox) = mpsc::channel();
        let frames: Published = Default::default();
        let publish = frames.clone();
        thread::Builder::new()
            .name("motor".into())
            .spawn(move || {
                let mut state = State::default();
                let mut deadline = Instant::now();
                loop {
                    let now = Instant::now();
                    loop {
                        match inbox.try_recv() {
                            Ok(command) => state.apply(command, now),
                            Err(TryRecvError::Empty) => break,
                            // O servidor foi embora: o motor para junto.
                            Err(TryRecvError::Disconnected) => return,
                        }
                    }
                    let universes = state.render(now);
                    let frames = Arc::new(Frames {
                        universes,
                        tick_time: now.elapsed(),
                        chasers: state.chasers(),
                        done: state.done(),
                    });
                    publish.store(frames);
                    deadline = next_deadline(deadline, Instant::now());
                    thread::sleep(deadline.saturating_duration_since(Instant::now()));
                }
            })
            .expect("não foi possível iniciar a thread do motor");
        Self { commands, frames }
    }

    pub fn send(&self, command: Command) {
        if self.commands.send(command).is_err() {
            tracing::error!("o motor parou; comando ignorado");
        }
    }
}

/// Próximo prazo, sempre na grade do tick: um tick atrasado é compensado no seguinte, sem deriva.
/// Atrasado mais de um período inteiro (PC suspenso, depurador), pula os ticks perdidos em vez de
/// rodá-los em rajada.
fn next_deadline(previous: Instant, now: Instant) -> Instant {
    let mut next = previous + TICK;
    while next + TICK <= now {
        next += TICK;
    }
    next
}

/// Uma cena (ou uma fixture de um EFX) acesa no motor: rodando ou, depois de parada, no fade out.
struct Fader {
    /// Número do fader, pelo qual a criação que o acendeu o encontra.
    serial: u64,
    /// A criação disparada pelo usuário de onde ele veio.
    owner: Id,
    fade_out: Span,
    /// Quando a cena parou e o fade out dela, já em tempo: ela sai do motor no fim dele.
    stopped: Option<(Instant, Duration)>,
    channels: Vec<Fade>,
    /// EFX relativo: os valores são somados ao que está embaixo, com o zero em 127 (no grosso).
    relative: bool,
    /// Intensidade da criação dona: multiplica os canais de intensidade (255 = cheia).
    intensity: u8,
}

/// Um canal de uma cena: vai de `start` ao valor da cena em `time`, contado de `since`. Com a cena
/// parada, vai a 0 (intensidade) ou ao que está embaixo.
#[derive(Clone)]
struct Fade {
    level: SceneLevel,
    /// Posição do universo no patch, que muda quando um universo é criado ou removido.
    universe: Option<usize>,
    start: u16,
    since: Instant,
    time: Duration,
}

impl Fader {
    /// O fade out parte do valor do último quadro.
    fn stop(&mut self, universes: &[UniversePatch], now: Instant, beat: Duration) {
        if self.stopped.is_some() {
            return;
        }
        let fade_out = self.fade_out.duration(beat);
        for f in &mut self.channels {
            let i = usize::from(f.level.channel) - 1;
            if f.universe.is_none_or(|u| universes[u].fades[i]) {
                let elapsed = now.saturating_duration_since(f.since);
                f.start = fade(f.start, f.level.value, elapsed, f.time);
            } else {
                f.start = f.level.value;
            }
            f.since = now;
            f.time = fade_out;
        }
        self.stopped = Some((now, fade_out));
    }
}

/// Fade in e fade out impostos pelo passo de um chaser a tudo o que ele dispara; `None` = o da própria
/// criação.
type Fades = (Option<Span>, Option<Span>);

/// O que uma criação rodando disparou no motor.
enum Node {
    /// Uma cena: o número do fader dela.
    Scene(u64),
    Chaser(Box<Runner>),
    Collection(Vec<Node>),
    Efx(Box<EfxRunner>),
}

impl Node {
    /// Terminou sozinha: o Single Shot que apagou no fim, ou a coleção com tudo terminado.
    fn done(&self) -> bool {
        match self {
            Node::Scene(_) => false,
            Node::Chaser(runner) => runner.done,
            Node::Collection(nodes) => !nodes.is_empty() && nodes.iter().all(Node::done),
            Node::Efx(runner) => runner.done,
        }
    }
}

/// O que começar, parar, atualizar e andar uma criação precisa do motor.
struct Ctx<'a> {
    faders: &'a mut Vec<Fader>,
    serial: &'a mut u64,
    universes: &'a [UniversePatch],
    /// Saída das criações no último quadro: de onde parte o fade de uma cena que começa.
    mixed: &'a [(Id, Frame)],
    owner: &'a Id,
    /// Intensidade da criação dona, para os faders novos.
    intensity: u8,
    now: Instant,
    /// Intervalo entre as batidas e a posição de agora na batida, em milésimos.
    beat: Duration,
    beat_pos: u64,
}

impl Ctx<'_> {
    /// Um fader novo, por último na mistura, partindo do valor que está saindo: um dimmer já aceso não
    /// pisca, e a posição desliza da atual para a nova. No EFX relativo, parte do zero (sem
    /// deslocamento).
    fn push(
        &mut self,
        levels: impl Iterator<Item = SceneLevel>,
        fade_in: Span,
        fade_out: Span,
        relative: bool,
    ) -> u64 {
        *self.serial += 1;
        let channels = levels
            .map(|level| {
                let universe = position(self.universes, &level);
                let mut start = if relative {
                    zero(&level)
                } else {
                    self.mixed
                        .iter()
                        .find(|(id, _)| *id == level.universe)
                        .map_or(0, |(_, frame)| read(frame, &level))
                };
                // Com a intensidade reduzida, o fade parte do valor cheio que dá o que está saindo:
                // num passo de chaser a 50%, a luz não afunda na troca
                let htp = universe
                    .is_some_and(|u| self.universes[u].intensity[usize::from(level.channel) - 1]);
                if htp && (1..255).contains(&self.intensity) {
                    let max = if level.fine.is_some() { 0xFFFF } else { 0xFF };
                    start = (u32::from(start) * 255 / u32::from(self.intensity)).min(max) as u16;
                }
                Fade {
                    universe,
                    start,
                    level,
                    since: self.now,
                    time: fade_in.duration(self.beat),
                }
            })
            .collect();
        self.faders.push(Fader {
            serial: *self.serial,
            owner: self.owner.clone(),
            fade_out,
            stopped: None,
            channels,
            relative,
            intensity: self.intensity,
        });
        *self.serial
    }

    /// Cada cena ganha um fader; o EFX, um para cada fixture, quando chega a vez dela.
    fn start(&mut self, program: Program, fades: Fades) -> Node {
        match program {
            Program::Scene(scene) => {
                let fade_in = fades.0.unwrap_or(scene.fade_in);
                let fade_out = fades.1.unwrap_or(scene.fade_out);
                Node::Scene(self.push(valid(scene.levels), fade_in, fade_out, false))
            }
            Program::Chaser(chaser) => Node::Chaser(Box::new(Runner::new(chaser, fades))),
            Program::Collection(programs) => {
                Node::Collection(programs.into_iter().map(|p| self.start(p, fades)).collect())
            }
            Program::Efx(efx) => Node::Efx(Box::new(EfxRunner::new(efx, fades, self))),
        }
    }

    fn stop_fader(&mut self, serial: u64) {
        if let Some(fader) = self.faders.iter_mut().find(|f| f.serial == serial) {
            fader.stop(self.universes, self.now, self.beat);
        }
    }

    /// Cada cena acesa começa o fade out dela.
    fn stop(&mut self, node: &Node) {
        match node {
            Node::Scene(serial) => self.stop_fader(*serial),
            Node::Chaser(runner) => {
                if let Some(active) = &runner.active {
                    self.stop(&active.node);
                }
            }
            Node::Collection(nodes) => {
                for node in nodes {
                    self.stop(node);
                }
            }
            Node::Efx(runner) => runner.stop(self),
        }
    }

    /// Edição com a criação rodando: vale na hora, sem recomeçar. Na cena, o valor mudado vai direto e
    /// o resto continua o fade; o que mudou de tipo recomeça.
    fn update(&mut self, node: &mut Node, program: Program, fades: Fades) {
        match (node, program) {
            (Node::Scene(serial), Program::Scene(scene)) => {
                let Some(fader) = self.faders.iter_mut().find(|f| f.serial == *serial) else {
                    return;
                };
                let old = std::mem::take(&mut fader.channels);
                fader.channels = valid(scene.levels)
                    .map(|level| match old.iter().find(|f| f.level == level) {
                        Some(f) => f.clone(),
                        None => Fade {
                            universe: position(self.universes, &level),
                            start: level.value,
                            level,
                            since: self.now,
                            time: Duration::ZERO,
                        },
                    })
                    .collect();
                fader.fade_out = fades.1.unwrap_or(scene.fade_out);
            }
            (Node::Chaser(runner), Program::Chaser(chaser)) => runner.update(chaser, fades, self),
            (Node::Efx(runner), Program::Efx(efx)) => runner.update(efx, fades, self),
            (Node::Collection(nodes), Program::Collection(programs)) => {
                for gone in nodes.split_off(programs.len().min(nodes.len())) {
                    self.stop(&gone);
                }
                for (i, program) in programs.into_iter().enumerate() {
                    match nodes.get_mut(i) {
                        Some(node) => self.update(node, program, fades),
                        None => {
                            let node = self.start(program, fades);
                            nodes.push(node);
                        }
                    }
                }
            }
            (node, program) => {
                self.stop(node);
                *node = self.start(program, fades);
            }
        }
    }

    /// Anda os chasers e os EFX, também os de dentro de um passo ou de uma coleção.
    fn tick(&mut self, node: &mut Node) {
        match node {
            Node::Scene(_) => {}
            Node::Chaser(runner) => runner.tick(self),
            Node::Efx(runner) => runner.tick(self),
            Node::Collection(nodes) => {
                for node in nodes {
                    self.tick(node);
                }
            }
        }
    }
}

/// Um canal fora do universo seria erro do servidor: o motor não pode cair por ele.
fn valid(levels: Vec<SceneLevel>) -> impl Iterator<Item = SceneLevel> {
    let valid = |channel: u16| (1..=512).contains(&channel);
    levels
        .into_iter()
        .filter(move |l| valid(l.channel) && l.fine.is_none_or(valid))
}

fn position(universes: &[UniversePatch], level: &SceneLevel) -> Option<usize> {
    universes.iter().position(|u| u.id == level.universe)
}

struct State {
    universes: Vec<UniversePatch>,
    /// Criações disparadas pelo usuário, na ordem de início.
    runs: Vec<(Id, Node)>,
    /// Intensidade de cada criação disparada.
    intensities: Vec<(Id, u8)>,
    /// Cenas acesas, na ordem de início: a mistura LTP segue essa ordem.
    faders: Vec<Fader>,
    serial: u64,
    /// Saída das criações no último quadro: de onde parte o fade de uma cena que começa.
    mixed: Vec<(Id, Frame)>,
    desk: Vec<DeskLevel>,
    preview: Vec<DeskLevel>,
    highlight: Highlight,
    grand_master: u8,
    blackout: bool,
    dark: Vec<(Id, u16)>,
    test: Option<(TestPattern, Instant)>,
    /// Intervalo entre as batidas (o BPM), a hora da próxima e quantas já passaram.
    beat: Duration,
    next_beat: Instant,
    beats: u64,
}

impl Default for State {
    fn default() -> Self {
        let beat = Duration::from_millis(500);
        Self {
            universes: Vec::new(),
            runs: Vec::new(),
            intensities: Vec::new(),
            faders: Vec::new(),
            serial: 0,
            mixed: Vec::new(),
            desk: Vec::new(),
            preview: Vec::new(),
            highlight: Highlight::default(),
            grand_master: 255,
            blackout: false,
            dark: Vec::new(),
            test: None,
            beat,
            next_beat: Instant::now() + beat,
            beats: 0,
        }
    }
}

impl State {
    fn apply(&mut self, command: Command, now: Instant) {
        match command {
            Command::Patch(universes) => {
                self.universes = universes;
                for f in self.faders.iter_mut().flat_map(|s| &mut s.channels) {
                    f.universe = self.universes.iter().position(|u| u.id == f.level.universe);
                }
            }
            Command::Running(runs) => self.set_running(runs, now),
            Command::Cut => {
                self.runs.clear();
                self.faders.clear();
            }
            Command::Step(id, action) => {
                if let Some((_, Node::Chaser(runner))) =
                    self.runs.iter_mut().find(|(r, _)| *r == id)
                {
                    runner.pending = Some(action);
                }
            }
            // O toque é uma batida: a mais perto dele, a que acabou de passar ou a que ia chegar (os
            // passos marcados para ela trocam agora)
            Command::Bpm(bpm) => {
                if self.beat_pos(now) % 1000 >= 500 {
                    self.beats += 1;
                }
                self.beat = Duration::from_secs(60) / u32::from(bpm.max(1));
                self.next_beat = now + self.beat;
            }
            Command::Tempo(bpm) => {
                let beat = Duration::from_secs(60) / u32::from(bpm.max(1));
                let left = self.next_beat.saturating_duration_since(now);
                self.next_beat = now + left.mul_f64(beat.as_secs_f64() / self.beat.as_secs_f64());
                self.beat = beat;
            }
            Command::Desk(levels) => self.desk = levels,
            Command::Preview(levels) => self.preview = levels,
            Command::Highlight(highlight) => self.highlight = highlight,
            Command::FixtureBlackout(channels) => self.dark = channels,
            Command::GrandMaster(value) => self.grand_master = value,
            Command::Blackout(on) => self.blackout = on,
            // Trocar de padrão recomeça a rampa do zero
            Command::Test(pattern) => self.test = pattern.map(|p| (p, now)),
        }
    }

    fn ctx<'a>(&'a mut self, owner: &'a Id, now: Instant) -> Ctx<'a> {
        Ctx {
            beat_pos: self.beat_pos(now),
            faders: &mut self.faders,
            serial: &mut self.serial,
            universes: &self.universes,
            mixed: &self.mixed,
            owner,
            intensity: self
                .intensities
                .iter()
                .find(|(id, _)| id == owner)
                .map_or(255, |(_, i)| *i),
            now,
            beat: self.beat,
        }
    }

    /// Acompanha a lista das criações disparadas. As que saíram dela começam o fade out; as que
    /// continuam recebem as edições na hora; as novas entram por último.
    fn set_running(&mut self, runs: Vec<Run>, now: Instant) {
        self.intensities = runs.iter().map(|r| (r.id.clone(), r.intensity)).collect();
        // A intensidade nova vale na hora para o que já está aceso (o que está apagando fica com a sua)
        for f in self.faders.iter_mut().filter(|f| f.stopped.is_none()) {
            if let Some((_, i)) = self.intensities.iter().find(|(id, _)| *id == f.owner) {
                f.intensity = *i;
            }
        }
        let mut old = std::mem::take(&mut self.runs);
        for (id, node) in &old {
            if !runs.iter().any(|r| r.id == *id) {
                self.ctx(id, now).stop(node);
            }
        }
        for run in runs {
            let node = match old.iter().position(|(id, _)| *id == run.id) {
                Some(i) => {
                    let (_, mut node) = old.swap_remove(i);
                    self.ctx(&run.id, now)
                        .update(&mut node, run.program, (None, None));
                    node
                }
                // Disparada de novo no meio do fade out: o que estava apagando sai, e a nova parte do
                // valor que está saindo, sem piscar
                None => {
                    self.faders
                        .retain(|f| f.owner != run.id || f.stopped.is_none());
                    self.ctx(&run.id, now).start(run.program, (None, None))
                }
            };
            self.runs.push((run.id, node));
        }
    }

    /// Este quadro cai numa batida? Cada batida conta a partir da anterior, e não do quadro, para não
    /// derivar, e cai no quadro mais perto da hora certa.
    fn on_beat(&mut self, now: Instant) -> bool {
        if now + TICK / 2 < self.next_beat {
            return false;
        }
        self.next_beat += self.beat;
        self.beats += 1;
        // Parado muito tempo (PC suspenso, depurador): recomeça daqui
        if self.next_beat + TICK / 2 <= now {
            self.next_beat = now + self.beat;
        }
        true
    }

    /// Posição na batida, em milésimos: as batidas que já passaram e a fração da atual. Como a
    /// batida, a fração conta do quadro mais perto da hora certa.
    fn beat_pos(&self, now: Instant) -> u64 {
        let left = self.next_beat.saturating_duration_since(now + TICK / 2);
        let left = (left.as_micros() * 1000 / self.beat.as_micros().max(1)).min(1000) as u64;
        self.beats * 1000 + 1000 - left
    }

    fn chasers(&self) -> Vec<(Id, u16)> {
        self.runs
            .iter()
            .filter_map(|(id, node)| match node {
                Node::Chaser(runner) if !runner.done => runner.step.map(|s| (id.clone(), s as u16)),
                _ => None,
            })
            .collect()
    }

    fn done(&self) -> Vec<Id> {
        self.runs
            .iter()
            .filter(|(_, node)| node.done())
            .map(|(id, _)| id.clone())
            .collect()
    }

    fn render(&mut self, now: Instant) -> Vec<(Id, Frame)> {
        self.on_beat(now);
        let mut runs = std::mem::take(&mut self.runs);
        for (id, node) in &mut runs {
            self.ctx(id, now).tick(node);
        }
        self.runs = runs;

        let mut mixed: Vec<Frame> = self.universes.iter().map(|u| *u.defaults).collect();
        self.mix(&mut mixed, now);
        let frames = self
            .universes
            .iter()
            .enumerate()
            .zip(&mixed)
            .map(|((index, u), mixed)| {
                let mut frame = *mixed;
                match self.test {
                    Some((pattern, since)) => {
                        let ms = now.duration_since(since).as_millis() as u64;
                        for (i, v) in frame.iter_mut().enumerate() {
                            // A perseguição atrasa cada canal 125 ms: a onda anda 8 canais por segundo
                            let delay = if pattern == TestPattern::Chase {
                                125 * i as u64
                            } else {
                                0
                            };
                            *v = triangle(ms + RAMP_MS - delay % RAMP_MS);
                        }
                    }
                    None => {
                        let set = |frame: &mut Frame, levels: &[DeskLevel]| {
                            for l in levels.iter().filter(|l| l.universe == u.id) {
                                if let Some(v) =
                                    frame.get_mut(usize::from(l.channel).wrapping_sub(1))
                                {
                                    *v = l.value;
                                }
                            }
                        };
                        set(&mut frame, &self.preview);
                        self.relative(index, &mut frame, now);
                        set(&mut frame, &self.desk);
                        set(&mut frame, &self.highlight.levels);
                        // As outras fixtures ficam mais fracas enquanto algo está localizado
                        if self.highlight.others < 255 && !self.highlight.located.is_empty() {
                            let located = mask(&self.highlight.located, &u.id);
                            for (i, v) in frame.iter_mut().enumerate() {
                                if u.intensity[i] && !located[i] {
                                    *v = reduce((*v).into(), self.highlight.others) as u8;
                                }
                            }
                        }
                    }
                }
                let dark = mask(&self.dark, &u.id);
                for (i, v) in frame
                    .iter_mut()
                    .enumerate()
                    .filter(|(i, _)| u.intensity[*i])
                {
                    *v = if self.blackout || dark[i] {
                        0
                    } else {
                        reduce((*v).into(), self.grand_master) as u8
                    };
                }
                (u.id.clone(), frame)
            })
            .collect();
        self.mixed = self
            .universes
            .iter()
            .map(|u| u.id.clone())
            .zip(mixed)
            .collect();
        self.faders.retain(|f| {
            f.stopped
                .is_none_or(|(at, time)| now.saturating_duration_since(at) + TICK < time)
        });
        frames
    }

    /// Mistura dos faders, na ordem de início: na intensidade vence o maior valor (HTP); no resto, o
    /// último (LTP).
    fn mix(&self, frames: &mut [Frame], now: Instant) {
        let mut taken = vec![[false; 512]; frames.len()];
        for fader in self.faders.iter().filter(|f| !f.relative) {
            let stopped = fader.stopped.is_some();
            for f in &fader.channels {
                let Some(u) = f.universe else { continue };
                let i = usize::from(f.level.channel) - 1;
                let htp = self.universes[u].intensity[i];
                let current = read(&frames[u], &f.level);
                // Parada, a intensidade vai a 0, e o resto, ao que está embaixo: outra cena ou o
                // padrão. No QLC+ o resto fica onde está.
                let target = match (stopped, htp) {
                    (false, _) => f.level.value,
                    (true, true) => 0,
                    (true, false) => current,
                };
                // Como no QLC+, o primeiro quadro já anda um passo
                let elapsed = now.saturating_duration_since(f.since) + TICK;
                // Sem fade, o canal muda no começo do fade in e no fim do fade out
                let value = if self.universes[u].fades[i] {
                    fade(f.start, target, elapsed, f.time)
                } else if stopped && elapsed < f.time {
                    f.start
                } else {
                    target
                };
                // A intensidade da criação só escala dimmer e cores, como no QLC+
                let value = if htp && fader.intensity < 255 {
                    reduce(value, fader.intensity)
                } else {
                    value
                };
                if !htp || !taken[u][i] || value > current {
                    write(&mut frames[u], &f.level, value);
                }
                taken[u][i] |= htp;
            }
        }
    }

    /// EFX relativos, por cima da prévia: o desenho somado ao que está embaixo, em qualquer ordem de
    /// início. Parados, voltam ao zero (sem deslocamento) no fade out.
    fn relative(&self, universe: usize, frame: &mut Frame, now: Instant) {
        for fader in self.faders.iter().filter(|f| f.relative) {
            for f in fader
                .channels
                .iter()
                .filter(|f| f.universe == Some(universe))
            {
                let zero = zero(&f.level);
                let target = if fader.stopped.is_some() {
                    zero
                } else {
                    f.level.value
                };
                let elapsed = now.saturating_duration_since(f.since) + TICK;
                let offset = i32::from(fade(f.start, target, elapsed, f.time)) - i32::from(zero);
                let max = if f.level.fine.is_some() { 0xFFFF } else { 0xFF };
                let value = (i32::from(read(frame, &f.level)) + offset).clamp(0, max);
                write(frame, &f.level, value as u16);
            }
        }
    }
}

/// Zero do EFX relativo: 127 no canal grosso, como no QLC+.
fn zero(level: &SceneLevel) -> u16 {
    if level.fine.is_some() { 127 << 8 } else { 127 }
}

/// Valor de um fade linear depois de `elapsed`, em inteiros como no QLC+: a conta trunca para o lado
/// do valor de partida, e o fade chega exatamente no destino.
fn fade(start: u16, target: u16, elapsed: Duration, time: Duration) -> u16 {
    if elapsed >= time {
        return target;
    }
    let step = u128::from(start.abs_diff(target)) * elapsed.as_micros() / time.as_micros().max(1);
    if target > start {
        start + step as u16
    } else {
        start - step as u16
    }
}

/// Valor de um canal no quadro; com o canal fino, em 16 bits.
fn read(frame: &Frame, level: &SceneLevel) -> u16 {
    let coarse = u16::from(frame[usize::from(level.channel) - 1]);
    match level.fine {
        Some(fine) => coarse << 8 | u16::from(frame[usize::from(fine) - 1]),
        None => coarse,
    }
}

fn write(frame: &mut Frame, level: &SceneLevel, value: u16) {
    match level.fine {
        Some(fine) => {
            frame[usize::from(level.channel) - 1] = (value >> 8) as u8;
            frame[usize::from(fine) - 1] = value as u8;
        }
        None => frame[usize::from(level.channel) - 1] = value as u8,
    }
}

/// Período da rampa de teste: sobe em 2 s e desce em 2 s.
const RAMP_MS: u64 = 4000;

fn triangle(ms: u64) -> u8 {
    let phase = ms % RAMP_MS;
    let up = if phase < RAMP_MS / 2 {
        phase
    } else {
        RAMP_MS - phase
    };
    (up * 255 / (RAMP_MS / 2)) as u8
}

/// Grand Master no modo "reduzir" do QLC+: `round(valor × gm / 255)`, em inteiros. Também a
/// intensidade de uma criação (em 16 bits, com o canal fino) e as outras fixtures do Localizar.
fn reduce(value: u16, by: u8) -> u16 {
    ((u32::from(value) * u32::from(by) * 2 + 255) / 510) as u16
}

/// Quais canais de um universo estão na lista (universo e endereço).
fn mask(channels: &[(Id, u16)], universe: &str) -> [bool; 512] {
    let mut mask = [false; 512];
    for (_, channel) in channels.iter().filter(|(id, _)| id == universe) {
        if let Some(m) = mask.get_mut(usize::from(*channel).wrapping_sub(1)) {
            *m = true;
        }
    }
    mask
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Universo de teste: 1, 2 e 5 de intensidade (dimmer, vermelho, azul); 3 é pan; 4 é um strobo
    /// com padrão 255, sem fade; 6 e 7 são o tilt grosso e o fino de um moving head.
    pub(crate) fn patched() -> State {
        let mut intensity = Box::new([true; 512]);
        for i in [2, 3, 5, 6] {
            intensity[i] = false;
        }
        let mut fades = Box::new([true; 512]);
        fades[3] = false;
        let mut defaults = Box::new([0; 512]);
        defaults[3] = 255;
        let mut s = State::default();
        let patch = UniversePatch {
            id: "u1".into(),
            intensity,
            fades,
            defaults,
        };
        s.apply(Command::Patch(vec![patch]), Instant::now());
        s
    }

    /// Com a Mesa simples nos canais 1, 2 e 3.
    fn state() -> State {
        let mut s = patched();
        let desk = vec![level(1, 255), level(2, 100), level(3, 200)];
        s.apply(Command::Desk(desk), Instant::now());
        s
    }

    fn level(channel: u16, value: u8) -> DeskLevel {
        DeskLevel {
            universe: "u1".into(),
            channel,
            value,
        }
    }

    /// Cena no universo de teste, com o mesmo tempo no fade in e no fade out.
    pub(crate) fn scene(levels: &[(u16, u16)], fade_ms: u32) -> Program {
        Program::Scene(SceneProgram {
            levels: levels
                .iter()
                .map(|&(channel, value)| SceneLevel {
                    universe: "u1".into(),
                    channel,
                    fine: None,
                    value,
                })
                .collect(),
            fade_in: Span::Ms(fade_ms),
            fade_out: Span::Ms(fade_ms),
        })
    }

    pub(crate) fn run(id: &str, program: Program) -> Run {
        Run {
            id: id.into(),
            program,
            intensity: 255,
        }
    }

    fn frame(s: &mut State) -> Frame {
        s.render(Instant::now())[0].1
    }

    #[test]
    fn desk_goes_over_the_defaults() {
        let mut s = state();
        assert_eq!(frame(&mut s)[..5], [255, 100, 200, 255, 0]);
        let mut levels = s.desk.clone();
        levels.push(level(4, 10));
        s.apply(Command::Desk(levels), Instant::now());
        assert_eq!(frame(&mut s)[3], 10);
        // Solto (o "Zerar" da Mesa simples), volta ao padrão
        s.apply(Command::Desk(vec![]), Instant::now());
        assert_eq!(frame(&mut s)[..4], [0, 0, 0, 255]);
    }

    #[test]
    fn layers_go_scenes_preview_desk_and_highlight() {
        let mut s = state();
        let now = Instant::now();
        // A mesa tem 1, 2 e 3: a cena só aparece no 5
        let a = run("a", scene(&[(1, 10), (5, 77)], 0));
        s.apply(Command::Running(vec![a]), now);
        assert_eq!(frame(&mut s)[..5], [255, 100, 200, 255, 77]);
        // A prévia fica por cima da cena e embaixo da mesa
        s.apply(Command::Preview(vec![level(1, 50), level(5, 20)]), now);
        assert_eq!(frame(&mut s)[..5], [255, 100, 200, 255, 20]);
        let highlight = Highlight {
            levels: vec![level(2, 255)],
            ..Highlight::default()
        };
        s.apply(Command::Highlight(highlight), now);
        assert_eq!(frame(&mut s)[1], 255);
        // O blackout vale também para o "Localizar"
        s.apply(Command::Blackout(true), now);
        assert_eq!(frame(&mut s)[..2], [0, 0]);
    }

    #[test]
    fn intensity_scales_only_dimmer_and_colors_of_all_it_lights_and_changes_live() {
        let mut s = patched();
        let now = Instant::now();
        // Uma coleção com uma cena no 1 (dimmer) e no 3 (pan) e um chaser com uma cena no 2 (cor)
        let chaser = Program::Chaser(ChaserProgram {
            steps: vec![Step {
                program: scene(&[(2, 100)], 0),
                fade_in: None,
                hold: Span::Ms(1000),
                fade_out: None,
            }],
            run_order: RunOrder::Loop,
            direction: Direction::Forward,
            keep_last: false,
        });
        let collection = Program::Collection(vec![scene(&[(1, 200), (3, 200)], 0), chaser]);
        let half = Run {
            intensity: 128,
            ..run("a", collection.clone())
        };
        s.apply(Command::Running(vec![half]), now);
        s.render(now);
        // A 50%, só o dimmer e a cor caem; o pan fica
        assert_eq!(frame(&mut s)[..3], [100, 50, 200]);
        // Ao vivo, sem recomeçar
        s.apply(Command::Running(vec![run("a", collection)]), now);
        assert_eq!(frame(&mut s)[..3], [200, 100, 200]);
    }

    #[test]
    fn a_reduced_intensity_does_not_dip_when_a_scene_takes_over() {
        let mut s = patched();
        let now = Instant::now();
        let half = |id, fade_ms| Run {
            intensity: 128,
            ..run(id, scene(&[(1, 200)], fade_ms))
        };
        s.apply(Command::Running(vec![half("a", 0)]), now);
        assert_eq!(frame(&mut s)[0], 100);
        // Outra criação a 50% entra no lugar, no mesmo valor e com fade: parte de 200 (o cheio que dá
        // os 100 que estão saindo), e a luz não afunda
        s.apply(Command::Running(vec![half("b", 1000)]), now);
        assert_eq!(frame(&mut s)[0], 100);
    }

    #[test]
    fn fixture_blackout_zeroes_only_their_intensity() {
        let mut s = state();
        let now = Instant::now();
        // A mesa tem 1 (dimmer), 2 (cor) e 3 (pan): as fixtures apagadas têm o 1 e o 3
        s.apply(
            Command::FixtureBlackout(vec![("u1".into(), 1), ("u1".into(), 3)]),
            now,
        );
        assert_eq!(frame(&mut s)[..3], [0, 100, 200]);
        s.apply(Command::FixtureBlackout(vec![]), now);
        assert_eq!(frame(&mut s)[..3], [255, 100, 200]);
    }

    #[test]
    fn highlight_dims_the_other_fixtures_and_keeps_the_located_one() {
        let mut s = state();
        let now = Instant::now();
        let desk = vec![level(1, 255), level(2, 100), level(3, 200), level(8, 200)];
        s.apply(Command::Desk(desk), now);
        s.apply(Command::Running(vec![run("a", scene(&[(5, 77)], 0))]), now);
        // Localizada: o canal 1 vai a 100%; o 2 também é dela, mas o Localizar não mexe nele
        let highlight = |others| {
            Command::Highlight(Highlight {
                levels: vec![level(1, 255)],
                located: vec![("u1".into(), 1), ("u1".into(), 2)],
                others,
            })
        };
        s.apply(highlight(255), now);
        assert_eq!(frame(&mut s)[..8], [255, 100, 200, 255, 77, 0, 0, 200]);
        // A 20%, só a intensidade das outras cai (o 5, da cena, e o 8, da mesa); o pan (3) não
        s.apply(highlight(51), now);
        assert_eq!(frame(&mut s)[..8], [255, 100, 200, 255, 15, 0, 0, 40]);
        // O Grand Master vale por cima de tudo
        s.apply(Command::GrandMaster(128), now);
        assert_eq!(frame(&mut s)[..8], [128, 50, 200, 255, 8, 0, 0, 20]);
        // A 0%, só a localizada fica acesa
        s.apply(Command::GrandMaster(255), now);
        s.apply(highlight(0), now);
        assert_eq!(frame(&mut s)[..8], [255, 100, 200, 255, 0, 0, 0, 0]);
        // Desligado, tudo volta
        s.apply(Command::Highlight(Highlight::default()), now);
        assert_eq!(frame(&mut s)[..8], [255, 100, 200, 255, 77, 0, 0, 200]);
    }

    #[test]
    fn fade_is_linear_and_truncates_like_qlc() {
        let ms = Duration::from_millis;
        // 0→250 em 1 s, a 50 quadros por segundo: 5, 10, 15…
        for k in 1..=50 {
            assert_eq!(fade(0, 250, ms(20 * k), ms(1000)), 5 * k as u16);
        }
        // Até um tick, chega na hora; 2 ticks dão a metade e o fim
        for time in [0, 4, 20] {
            assert_eq!(fade(0, 250, ms(20), ms(time)), 250);
        }
        assert_eq!([20, 40].map(|t| fade(0, 250, ms(t), ms(40))), [125, 250]);
        // Os números do `fadechannel_test` do QLC+
        let up: Vec<u16> = (0..=13).map(|t| fade(13, 147, ms(t), ms(13))).collect();
        assert_eq!(
            up,
            [13, 23, 33, 43, 54, 64, 74, 85, 95, 105, 116, 126, 136, 147]
        );
        let down: Vec<u16> = (0..12).map(|t| fade(245, 101, ms(t), ms(200))).collect();
        assert_eq!(
            down,
            [245, 245, 244, 243, 243, 242, 241, 240, 240, 239, 238, 238]
        );
        let end = [100, 101, 102, 198, 199, 200].map(|t| fade(245, 101, ms(t), ms(200)));
        assert_eq!(end, [173, 173, 172, 103, 102, 101]);
    }

    #[test]
    fn scene_fades_in_takes_live_edits_and_fades_out() {
        // Como o `writeHTPTwoTicks` do QLC+: dimmer e pan com fades de 2 ticks
        let mut s = patched();
        let t0 = Instant::now();
        let at = |s: &mut State, tick: u32| {
            let f = s.render(t0 + TICK * tick)[0].1;
            (f[0], f[2])
        };
        let set = |s: &mut State, tick: u32, dimmer: u16| {
            let runs = vec![run("a", scene(&[(1, dimmer), (3, 100)], 40))];
            s.apply(Command::Running(runs), t0 + TICK * tick);
        };
        set(&mut s, 0, 250);
        assert_eq!(
            [0, 1, 2].map(|t| at(&mut s, t)),
            [(125, 50), (250, 100), (250, 100)]
        );
        // Editada rodando: vale no quadro seguinte, para cima e para baixo
        set(&mut s, 3, 254);
        assert_eq!(at(&mut s, 3), (254, 100));
        set(&mut s, 4, 180);
        assert_eq!(at(&mut s, 4), (180, 100));
        // Parada: a intensidade vai a 0 e o pan volta ao padrão, no fade out
        s.apply(Command::Running(vec![]), t0 + TICK * 5);
        assert_eq!([5, 6].map(|t| at(&mut s, t)), [(90, 50), (0, 0)]);
        assert!(s.faders.is_empty());
    }

    #[test]
    fn htp_keeps_the_highest_and_ltp_follows_the_last_started() {
        let mut s = patched();
        let t0 = Instant::now();
        let at = |s: &mut State, tick: u32| {
            let f = s.render(t0 + TICK * tick)[0].1;
            [f[0], f[1], f[4], f[2]]
        };
        let azul = || run("azul", scene(&[(1, 200), (5, 255), (3, 50)], 0));
        let vermelha = || run("vermelha", scene(&[(1, 100), (2, 255), (3, 200)], 40));
        let set = |s: &mut State, tick: u32, runs: Vec<Run>| {
            s.apply(Command::Running(runs), t0 + TICK * tick);
        };
        set(&mut s, 0, vec![azul()]);
        set(&mut s, 0, vec![azul(), vermelha()]);
        // Dimmer, vermelho e azul pelo maior (magenta); o pan da última iniciada
        assert_eq!(at(&mut s, 1), [200, 255, 255, 200]);
        // Parar a vermelha: o vermelho apaga, e o pan desliza até o da azul
        set(&mut s, 2, vec![azul()]);
        assert_eq!(at(&mut s, 2), [200, 128, 255, 125]);
        assert_eq!(at(&mut s, 3), [200, 0, 255, 50]);
        // Rodar de novo no meio do fade out continua do valor que está saindo
        set(&mut s, 4, vec![azul(), vermelha()]);
        assert_eq!(at(&mut s, 5)[3], 200);
        set(&mut s, 6, vec![azul()]);
        assert_eq!(at(&mut s, 6)[3], 125);
        set(&mut s, 7, vec![azul(), vermelha()]);
        assert_eq!([7, 8].map(|t| at(&mut s, t)[3]), [162, 200]);
    }

    #[test]
    fn channels_without_fade_change_at_the_start_and_at_the_end() {
        let mut s = patched();
        let t0 = Instant::now();
        let at = |s: &mut State, tick: u32| s.render(t0 + TICK * tick)[0].1[3];
        s.apply(Command::Running(vec![run("a", scene(&[(4, 10)], 40))]), t0);
        assert_eq!(at(&mut s, 0), 10);
        s.apply(Command::Running(vec![]), t0 + TICK);
        // O strobo volta ao padrão (255) só no fim do fade out
        assert_eq!([1, 2].map(|t| at(&mut s, t)), [10, 255]);
    }

    #[test]
    fn fine_channel_makes_a_16_bit_fade() {
        let mut s = patched();
        let t0 = Instant::now();
        let Program::Scene(mut tilt) = scene(&[], 40) else {
            unreachable!()
        };
        tilt.levels.push(SceneLevel {
            universe: "u1".into(),
            channel: 6,
            fine: Some(7),
            value: 0x0100,
        });
        s.apply(Command::Running(vec![run("a", Program::Scene(tilt))]), t0);
        // No meio do fade, o grosso ainda em 0 e o fino na metade
        let f = s.render(t0)[0].1;
        assert_eq!((f[5], f[6]), (0, 128));
        let f = s.render(t0 + TICK)[0].1;
        assert_eq!((f[5], f[6]), (1, 0));
    }

    #[test]
    fn cut_stops_everything_without_fade_out() {
        let mut s = patched();
        let now = Instant::now();
        let Program::Scene(mut a) = scene(&[(1, 255)], 0) else {
            unreachable!()
        };
        a.fade_out = Span::Ms(10_000);
        s.apply(Command::Running(vec![run("a", Program::Scene(a))]), now);
        assert_eq!(frame(&mut s)[0], 255);
        s.apply(Command::Cut, now);
        assert_eq!(frame(&mut s)[0], 0);
        assert!(s.runs.is_empty());
    }

    #[test]
    fn collection_starts_and_stops_everything_together() {
        let mut s = patched();
        let t0 = Instant::now();
        let at = |s: &mut State, tick: u32| {
            let f = s.render(t0 + TICK * tick)[0].1;
            [f[0], f[2]]
        };
        // Cada cena com o seu fade: o dimmer em 2 ticks e o pan na hora
        let both = Program::Collection(vec![scene(&[(1, 200)], 40), scene(&[(3, 90)], 0)]);
        s.apply(Command::Running(vec![run("k", both)]), t0);
        assert_eq!([0, 1].map(|t| at(&mut s, t)), [[100, 90], [200, 90]]);
        // Com uma cena dentro, ela nunca termina sozinha
        assert!(s.done().is_empty());
        s.apply(Command::Running(vec![]), t0 + TICK * 2);
        assert_eq!([2, 3].map(|t| at(&mut s, t)), [[100, 0], [0, 0]]);
    }

    #[test]
    fn grand_master_reduces_only_intensity_rounding_like_qlc() {
        let mut s = state();
        s.apply(Command::GrandMaster(128), Instant::now());
        let f = frame(&mut s);
        // 255 × 128/255 = 128; 100 × 128/255 = 50,2 → 50; o pan fica
        assert_eq!(f[..3], [128, 50, 200]);
        assert_eq!(reduce(1, 128), 1);
        assert_eq!(reduce(255, 255), 255);
        assert_eq!(reduce(200, 0), 0);
        // Os números do `universe_test` do QLC+, com o Grand Master a 63
        assert_eq!([10, 20, 50].map(|v| reduce(v, 63)), [2, 5, 12]);
    }

    #[test]
    fn blackout_zeroes_intensity_and_keeps_position() {
        let mut s = state();
        s.apply(Command::Blackout(true), Instant::now());
        assert_eq!(frame(&mut s)[..4], [0, 0, 200, 255]);
        s.apply(Command::Blackout(false), Instant::now());
        assert_eq!(frame(&mut s)[..3], [255, 100, 200]);
    }

    #[test]
    fn test_pattern_replaces_the_desk() {
        let mut s = state();
        s.apply(Command::Test(Some(TestPattern::Ramp)), Instant::now());
        let since = s.test.unwrap().1;
        let mut at = |ms| s.render(since + Duration::from_millis(ms))[0].1;
        assert_eq!(at(0)[..3], [0, 0, 0]);
        assert_eq!(at(2000)[..3], [255, 255, 255]);
        assert_eq!(at(1000)[0], 127);
        s.apply(Command::Test(Some(TestPattern::Chase)), Instant::now());
        let since = s.test.unwrap().1;
        // O canal 9 está 1 s atrasado: no tempo 1 s ele começa e o canal 1 está no meio da subida
        let f = s.render(since + Duration::from_millis(1000))[0].1;
        assert_eq!((f[0], f[8]), (127, 0));
    }

    #[test]
    fn beats_follow_the_bpm_without_drift_and_restart_on_tap() {
        let mut s = State::default();
        let t0 = Instant::now();
        // 128 BPM: uma batida a cada 468,75 ms, que não cabe na grade de 20 ms
        s.apply(Command::Bpm(128), t0);
        let beats: Vec<u32> = (1..=3000).filter(|&k| s.on_beat(t0 + TICK * k)).collect();
        assert_eq!(beats.len(), 128);
        // A última batida cai em 60 s exatos, e nenhuma fica mais de meio tick fora da hora certa
        assert_eq!(beats.last(), Some(&3000));
        for (n, k) in beats.iter().enumerate() {
            let right = 468.75 * (n + 1) as f64;
            assert!((f64::from(*k * 20) - right).abs() <= 10.0, "batida {n}");
        }
        // O tap recomeça a batida: a próxima vem um intervalo depois do toque
        let tap = t0 + TICK * 3010;
        s.apply(Command::Bpm(120), tap);
        let next = (3011..4000).find(|&k| s.on_beat(t0 + TICK * k));
        assert_eq!(next, Some(3035));
    }

    #[test]
    fn tempo_changes_the_speed_without_restarting_the_beat() {
        let mut s = State::default();
        let t0 = Instant::now();
        s.apply(Command::Bpm(120), t0);
        // Na metade da batida de 500 ms, o fader vai a 60 BPM (1 s): falta a metade da batida nova
        let now = t0 + Duration::from_millis(250);
        s.apply(Command::Tempo(60), now);
        assert_eq!(s.beat, Duration::from_secs(1));
        assert_eq!(s.next_beat, now + Duration::from_millis(500));
        // O tap, ao contrário, recomeça: a próxima batida só viria um intervalo inteiro depois
        s.apply(Command::Bpm(60), now);
        assert_eq!(s.next_beat, now + Duration::from_secs(1));
    }

    #[test]
    fn four_full_universes_render_well_within_the_tick() {
        let mut s = State::default();
        let ids: Vec<Id> = (1..=4).map(|u| format!("u{u}")).collect();
        let patch = ids
            .iter()
            .map(|id| UniversePatch {
                id: id.clone(),
                intensity: Box::new([true; 512]),
                fades: Box::new([true; 512]),
                defaults: Box::new([0; 512]),
            })
            .collect();
        let t0 = Instant::now();
        s.apply(Command::Patch(patch), t0);
        let full = |n: u16| {
            Program::Scene(SceneProgram {
                levels: ids
                    .iter()
                    .flat_map(|u| {
                        (1..=512).map(move |channel| SceneLevel {
                            universe: u.clone(),
                            channel,
                            fine: None,
                            value: (channel + n) % 256,
                        })
                    })
                    .collect(),
                fade_in: Span::Ms(10_000),
                fade_out: Span::Ms(10_000),
            })
        };
        // Oito cenas em fade, cada uma com os 2048 canais, e quatro chasers delas trocando de passo a
        // cada 3 quadros, com crossfade de 2 quadros
        let mut runs: Vec<Run> = (0..8).map(|n| run(&format!("s{n}"), full(n))).collect();
        for n in 0..4 {
            let step = |k: u16| Step {
                program: full(n * 2 + k),
                fade_in: Some(Span::Ms(40)),
                hold: Span::Ms(20),
                fade_out: Some(Span::Ms(40)),
            };
            let chaser = ChaserProgram {
                steps: vec![step(0), step(1)],
                run_order: RunOrder::Loop,
                direction: Direction::Forward,
                keep_last: false,
            };
            runs.push(run(&format!("c{n}"), Program::Chaser(chaser)));
        }
        // E um EFX em 48 moving heads, 12 por universo, com pan e tilt em 16 bits
        let heads: Vec<(Id, u16)> = ids
            .iter()
            .flat_map(|u| (0..12).map(move |k| (u.clone(), 1 + 4 * k)))
            .collect();
        let at = |(u, c): &(Id, u16), offset| SceneLevel {
            universe: u.clone(),
            channel: c + offset,
            fine: Some(c + offset + 1),
            value: 0,
        };
        let efx = Efx {
            fixtures: (0..heads.len())
                .map(|k| rlc_core::EfxFixture {
                    target: rlc_core::EfxTarget::Fixture(format!("mh{k}")),
                    reverse: false,
                    start_offset: 0,
                    spread: 0,
                })
                .collect(),
            ..efx::tests::qlc(rlc_core::EfxAlgorithm::Lissajous)
        };
        let efx = EfxProgram {
            efx,
            heads: heads
                .iter()
                .map(|h| [Some(at(h, 0)), Some(at(h, 2))])
                .collect(),
            cycle: Span::Ms(2000),
            fade_in: Span::Ms(1000),
            fade_out: Span::Ms(1000),
        };
        runs.push(run("e", Program::Efx(efx)));
        s.apply(Command::Running(runs), t0);
        let started = Instant::now();
        for tick in 0..50 {
            s.render(t0 + TICK * tick);
        }
        let average = started.elapsed() / 50;
        // A meta é 2 ms no app compilado; nos testes sem otimização, basta caber no tick
        let limit = if cfg!(debug_assertions) {
            TICK
        } else {
            Duration::from_millis(2)
        };
        assert!(average < limit, "tick de {average:?}");
    }

    #[test]
    fn late_tick_is_compensated_and_long_stalls_skip() {
        let base = Instant::now();
        let ms = Duration::from_millis;
        // No prazo: o próximo é 20 ms depois do anterior
        assert_eq!(next_deadline(base, base + ms(5)), base + ms(20));
        // 5 ms atrasado: o próximo prazo já passou, então roda na hora e volta à grade
        assert_eq!(next_deadline(base, base + ms(25)), base + ms(20));
        // Parado 70 ms: pula para o último prazo da grade, sem rajada
        assert_eq!(next_deadline(base, base + ms(70)), base + ms(60));
    }
}

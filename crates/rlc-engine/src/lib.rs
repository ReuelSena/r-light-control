//! Motor (A1): uma thread a 50 Hz com prazo absoluto, dona do estado de execução. Os comandos chegam
//! por canal e são aplicados no começo de cada tick, e cada tick publica o quadro de cada universo.
//! Camadas de cada universo, de baixo para cima: o valor padrão dos canais, as cenas rodando, a prévia
//! (a cena em edição), a Mesa simples, o "Localizar" e, por fim, o Grand Master e o blackout. O padrão
//! de teste substitui as camadas de baixo.
//! As cenas se misturam como no QLC+: nos canais de intensidade (dimmer e cores) vence o maior valor
//! (HTP); nos outros, a cena iniciada por último (LTP).

use std::sync::Arc;
use std::sync::mpsc::{self, Sender, TryRecvError};
use std::thread;
use std::time::{Duration, Instant};

use arc_swap::ArcSwap;
use rlc_core::{DeskLevel, Id, TestPattern};

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

/// Uma cena rodando, já nos endereços DMX (o motor não conhece fixtures).
pub struct RunningScene {
    pub id: Id,
    pub levels: Vec<SceneLevel>,
    pub fade_in: Duration,
    pub fade_out: Duration,
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

pub enum Command {
    /// Universos do projeto, na ordem.
    Patch(Vec<UniversePatch>),
    /// Cenas rodando, na ordem de início.
    Scenes(Vec<RunningScene>),
    /// Para as cenas na hora, sem fade out (pânico, "Novo" e "Abrir").
    Cut,
    /// Níveis da Mesa simples de todos os universos.
    Desk(Vec<DeskLevel>),
    /// Valores da cena em prévia, já nos endereços DMX.
    Preview(Vec<DeskLevel>),
    /// Valores do "Localizar", acima da Mesa simples.
    Highlight(Vec<DeskLevel>),
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

/// Uma cena no motor: rodando ou, depois de parada, no fade out.
struct Fader {
    id: Id,
    fade_out: Duration,
    /// Quando a cena parou: ela sai do motor no fim do fade out.
    stopped: Option<Instant>,
    channels: Vec<Fade>,
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

struct State {
    universes: Vec<UniversePatch>,
    scenes: Vec<Fader>,
    /// Saída das cenas no último quadro: de onde parte o fade de uma cena que começa.
    mixed: Vec<(Id, Frame)>,
    desk: Vec<DeskLevel>,
    preview: Vec<DeskLevel>,
    highlight: Vec<DeskLevel>,
    grand_master: u8,
    blackout: bool,
    test: Option<(TestPattern, Instant)>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            universes: Vec::new(),
            scenes: Vec::new(),
            mixed: Vec::new(),
            desk: Vec::new(),
            preview: Vec::new(),
            highlight: Vec::new(),
            grand_master: 255,
            blackout: false,
            test: None,
        }
    }
}

impl State {
    fn apply(&mut self, command: Command, now: Instant) {
        match command {
            Command::Patch(universes) => {
                self.universes = universes;
                for f in self.scenes.iter_mut().flat_map(|s| &mut s.channels) {
                    f.universe = self.universes.iter().position(|u| u.id == f.level.universe);
                }
            }
            Command::Scenes(scenes) => self.set_scenes(scenes, now),
            Command::Cut => self.scenes.clear(),
            Command::Desk(levels) => self.desk = levels,
            Command::Preview(levels) => self.preview = levels,
            Command::Highlight(levels) => self.highlight = levels,
            Command::GrandMaster(value) => self.grand_master = value,
            Command::Blackout(on) => self.blackout = on,
            // Trocar de padrão recomeça a rampa do zero
            Command::Test(pattern) => self.test = pattern.map(|p| (p, now)),
        }
    }

    /// Acompanha a lista das cenas rodando. As que saíram dela começam o fade out; as que continuam
    /// recebem na hora os valores editados; as novas entram por último, com o fade in a partir do que
    /// está saindo.
    fn set_scenes(&mut self, scenes: Vec<RunningScene>, now: Instant) {
        let universes = &self.universes;
        let stopped = self
            .scenes
            .iter_mut()
            .filter(|f| f.stopped.is_none() && !scenes.iter().any(|s| s.id == f.id));
        for fader in stopped {
            for f in &mut fader.channels {
                // O fade out parte do valor do último quadro
                let i = usize::from(f.level.channel) - 1;
                if f.universe.is_none_or(|u| universes[u].fades[i]) {
                    let elapsed = now.saturating_duration_since(f.since);
                    f.start = fade(f.start, f.level.value, elapsed, f.time);
                } else {
                    f.start = f.level.value;
                }
                f.since = now;
                f.time = fader.fade_out;
            }
            fader.stopped = Some(now);
        }

        let universe = |level: &SceneLevel| universes.iter().position(|u| u.id == level.universe);
        // Um canal fora do universo seria erro do servidor: o motor não pode cair por ele
        let valid = |channel: u16| (1..=512).contains(&channel);
        for scene in scenes {
            let levels = scene
                .levels
                .into_iter()
                .filter(|l| valid(l.channel) && l.fine.is_none_or(valid));
            match self
                .scenes
                .iter_mut()
                .find(|f| f.id == scene.id && f.stopped.is_none())
            {
                // Edição com a cena rodando: o valor mudado vale na hora, e o resto continua o fade
                Some(fader) => {
                    let old = std::mem::take(&mut fader.channels);
                    fader.channels = levels
                        .map(|level| match old.iter().find(|f| f.level == level) {
                            Some(f) => f.clone(),
                            None => Fade {
                                universe: universe(&level),
                                start: level.value,
                                level,
                                since: now,
                                time: Duration::ZERO,
                            },
                        })
                        .collect();
                    fader.fade_out = scene.fade_out;
                }
                // Rodar de novo uma cena no fade out: a parada sai, e a nova parte do valor que está
                // saindo, sem piscar
                None => {
                    self.scenes.retain(|f| f.id != scene.id);
                    let channels = levels
                        .map(|level| Fade {
                            universe: universe(&level),
                            start: self
                                .mixed
                                .iter()
                                .find(|(id, _)| *id == level.universe)
                                .map_or(0, |(_, frame)| read(frame, &level)),
                            level,
                            since: now,
                            time: scene.fade_in,
                        })
                        .collect();
                    self.scenes.push(Fader {
                        id: scene.id,
                        fade_out: scene.fade_out,
                        stopped: None,
                        channels,
                    });
                }
            }
        }
    }

    fn render(&mut self, now: Instant) -> Vec<(Id, Frame)> {
        let mut mixed: Vec<Frame> = self.universes.iter().map(|u| *u.defaults).collect();
        self.mix(&mut mixed, now);
        let frames = self
            .universes
            .iter()
            .zip(&mixed)
            .map(|(u, mixed)| {
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
                        let layers = self.preview.iter().chain(&self.desk).chain(&self.highlight);
                        for l in layers.filter(|l| l.universe == u.id) {
                            if let Some(v) = frame.get_mut(usize::from(l.channel).wrapping_sub(1)) {
                                *v = l.value;
                            }
                        }
                    }
                }
                for (v, _) in frame
                    .iter_mut()
                    .zip(u.intensity.iter())
                    .filter(|(_, i)| **i)
                {
                    *v = if self.blackout {
                        0
                    } else {
                        reduce(*v, self.grand_master)
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
        self.scenes.retain(|f| {
            f.stopped
                .is_none_or(|at| now.saturating_duration_since(at) + TICK < f.fade_out)
        });
        frames
    }

    /// Mistura das cenas, na ordem de início: na intensidade vence o maior valor (HTP); no resto, a
    /// última cena (LTP).
    fn mix(&self, frames: &mut [Frame], now: Instant) {
        let mut taken = vec![[false; 512]; frames.len()];
        for fader in &self.scenes {
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
                if !htp || !taken[u][i] || value > current {
                    write(&mut frames[u], &f.level, value);
                }
                taken[u][i] |= htp;
            }
        }
    }
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

/// Grand Master no modo "reduzir" do QLC+: `round(valor × gm / 255)`, em inteiros.
fn reduce(value: u8, grand_master: u8) -> u8 {
    ((u32::from(value) * u32::from(grand_master) * 2 + 255) / 510) as u8
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Universo de teste: 1, 2 e 5 de intensidade (dimmer, vermelho, azul); 3 é pan; 4 é um strobo
    /// com padrão 255, sem fade; 6 e 7 são o tilt grosso e o fino de um moving head.
    fn patched() -> State {
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

    fn scene(id: &str, levels: &[(u16, u16)], fade_ms: u64) -> RunningScene {
        RunningScene {
            id: id.into(),
            levels: levels
                .iter()
                .map(|&(channel, value)| SceneLevel {
                    universe: "u1".into(),
                    channel,
                    fine: None,
                    value,
                })
                .collect(),
            fade_in: Duration::from_millis(fade_ms),
            fade_out: Duration::from_millis(fade_ms),
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
        s.apply(
            Command::Scenes(vec![scene("a", &[(1, 10), (5, 77)], 0)]),
            now,
        );
        assert_eq!(frame(&mut s)[..5], [255, 100, 200, 255, 77]);
        // A prévia fica por cima da cena e embaixo da mesa
        s.apply(Command::Preview(vec![level(1, 50), level(5, 20)]), now);
        assert_eq!(frame(&mut s)[..5], [255, 100, 200, 255, 20]);
        s.apply(Command::Highlight(vec![level(2, 255)]), now);
        assert_eq!(frame(&mut s)[1], 255);
        // O blackout vale também para o "Localizar"
        s.apply(Command::Blackout(true), now);
        assert_eq!(frame(&mut s)[..2], [0, 0]);
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
        let run = |s: &mut State, tick: u32, dimmer: u16| {
            let scenes = vec![scene("a", &[(1, dimmer), (3, 100)], 40)];
            s.apply(Command::Scenes(scenes), t0 + TICK * tick);
        };
        run(&mut s, 0, 250);
        assert_eq!(
            [0, 1, 2].map(|t| at(&mut s, t)),
            [(125, 50), (250, 100), (250, 100)]
        );
        // Editada rodando: vale no quadro seguinte, para cima e para baixo
        run(&mut s, 3, 254);
        assert_eq!(at(&mut s, 3), (254, 100));
        run(&mut s, 4, 180);
        assert_eq!(at(&mut s, 4), (180, 100));
        // Parada: a intensidade vai a 0 e o pan volta ao padrão, no fade out
        s.apply(Command::Scenes(vec![]), t0 + TICK * 5);
        assert_eq!([5, 6].map(|t| at(&mut s, t)), [(90, 50), (0, 0)]);
        assert!(s.scenes.is_empty());
    }

    #[test]
    fn htp_keeps_the_highest_and_ltp_follows_the_last_started() {
        let mut s = patched();
        let t0 = Instant::now();
        let at = |s: &mut State, tick: u32| {
            let f = s.render(t0 + TICK * tick)[0].1;
            [f[0], f[1], f[4], f[2]]
        };
        let azul = || scene("azul", &[(1, 200), (5, 255), (3, 50)], 0);
        let vermelha = || scene("vermelha", &[(1, 100), (2, 255), (3, 200)], 40);
        let run = |s: &mut State, tick: u32, scenes: Vec<RunningScene>| {
            s.apply(Command::Scenes(scenes), t0 + TICK * tick);
        };
        run(&mut s, 0, vec![azul()]);
        run(&mut s, 0, vec![azul(), vermelha()]);
        // Dimmer, vermelho e azul pelo maior (magenta); o pan da última iniciada
        assert_eq!(at(&mut s, 1), [200, 255, 255, 200]);
        // Parar a vermelha: o vermelho apaga, e o pan desliza até o da azul
        run(&mut s, 2, vec![azul()]);
        assert_eq!(at(&mut s, 2), [200, 128, 255, 125]);
        assert_eq!(at(&mut s, 3), [200, 0, 255, 50]);
        // Rodar de novo no meio do fade out continua do valor que está saindo
        run(&mut s, 4, vec![azul(), vermelha()]);
        assert_eq!(at(&mut s, 5)[3], 200);
        run(&mut s, 6, vec![azul()]);
        assert_eq!(at(&mut s, 6)[3], 125);
        run(&mut s, 7, vec![azul(), vermelha()]);
        assert_eq!([7, 8].map(|t| at(&mut s, t)[3]), [162, 200]);
    }

    #[test]
    fn channels_without_fade_change_at_the_start_and_at_the_end() {
        let mut s = patched();
        let t0 = Instant::now();
        let at = |s: &mut State, tick: u32| s.render(t0 + TICK * tick)[0].1[3];
        s.apply(Command::Scenes(vec![scene("a", &[(4, 10)], 40)]), t0);
        assert_eq!(at(&mut s, 0), 10);
        s.apply(Command::Scenes(vec![]), t0 + TICK);
        // O strobo volta ao padrão (255) só no fim do fade out
        assert_eq!([1, 2].map(|t| at(&mut s, t)), [10, 255]);
    }

    #[test]
    fn fine_channel_makes_a_16_bit_fade() {
        let mut s = patched();
        let t0 = Instant::now();
        let mut tilt = scene("a", &[], 40);
        tilt.levels.push(SceneLevel {
            universe: "u1".into(),
            channel: 6,
            fine: Some(7),
            value: 0x0100,
        });
        s.apply(Command::Scenes(vec![tilt]), t0);
        // No meio do fade, o grosso ainda em 0 e o fino na metade
        let f = s.render(t0)[0].1;
        assert_eq!((f[5], f[6]), (0, 128));
        let f = s.render(t0 + TICK)[0].1;
        assert_eq!((f[5], f[6]), (1, 0));
    }

    #[test]
    fn cut_stops_the_scenes_without_fade_out() {
        let mut s = patched();
        let now = Instant::now();
        let mut a = scene("a", &[(1, 255)], 0);
        a.fade_out = Duration::from_secs(10);
        s.apply(Command::Scenes(vec![a]), now);
        assert_eq!(frame(&mut s)[0], 255);
        s.apply(Command::Cut, now);
        assert_eq!(frame(&mut s)[0], 0);
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
        // Oito cenas em fade, cada uma com os 2048 canais
        let scenes = (0..8)
            .map(|n| RunningScene {
                id: format!("s{n}"),
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
                fade_in: Duration::from_secs(10),
                fade_out: Duration::from_secs(10),
            })
            .collect();
        s.apply(Command::Scenes(scenes), t0);
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

//! Motor (A1): uma thread a 50 Hz com prazo absoluto, dona do estado de execução. Os comandos chegam
//! por canal e são aplicados no começo de cada tick, e cada tick publica o quadro de cada universo.
//! Cada universo parte do valor padrão dos canais; por cima vem a Mesa simples (ou o padrão de
//! teste) e, por fim, o Grand Master e o blackout.
//! As criações do Studio entram na F6.

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
    /// Canais de intensidade: o Grand Master e o blackout só agem neles.
    pub intensity: Box<[bool; 512]>,
    /// Valor de cada canal quando nada o controla: a base do quadro.
    pub defaults: Box<Frame>,
}

pub enum Command {
    /// Universos do projeto, na ordem.
    Patch(Vec<UniversePatch>),
    /// Níveis da Mesa simples de todos os universos.
    Desk(Vec<DeskLevel>),
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
                    loop {
                        match inbox.try_recv() {
                            Ok(command) => state.apply(command),
                            Err(TryRecvError::Empty) => break,
                            // O servidor foi embora: o motor para junto.
                            Err(TryRecvError::Disconnected) => return,
                        }
                    }
                    let started = Instant::now();
                    let universes = state.render(started);
                    let frames = Arc::new(Frames {
                        universes,
                        tick_time: started.elapsed(),
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

struct State {
    universes: Vec<UniversePatch>,
    desk: Vec<DeskLevel>,
    grand_master: u8,
    blackout: bool,
    test: Option<(TestPattern, Instant)>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            universes: Vec::new(),
            desk: Vec::new(),
            grand_master: 255,
            blackout: false,
            test: None,
        }
    }
}

impl State {
    fn apply(&mut self, command: Command) {
        match command {
            Command::Patch(universes) => self.universes = universes,
            Command::Desk(levels) => self.desk = levels,
            Command::GrandMaster(value) => self.grand_master = value,
            Command::Blackout(on) => self.blackout = on,
            // Trocar de padrão recomeça a rampa do zero
            Command::Test(pattern) => self.test = pattern.map(|p| (p, Instant::now())),
        }
    }

    fn render(&self, now: Instant) -> Vec<(Id, Frame)> {
        self.universes
            .iter()
            .map(|u| {
                let mut frame = *u.defaults;
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
                        for l in self.desk.iter().filter(|l| l.universe == u.id) {
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
            .collect()
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

    fn state() -> State {
        // Canais 1 e 2 de intensidade (dimmer, vermelho); o 3 é pan e o 4, um strobo com padrão 255
        let mut intensity = Box::new([true; 512]);
        intensity[2] = false;
        intensity[3] = false;
        let mut defaults = Box::new([0; 512]);
        defaults[3] = 255;
        let mut s = State::default();
        s.apply(Command::Patch(vec![UniversePatch {
            id: "u1".into(),
            intensity,
            defaults,
        }]));
        let level = |channel, value| DeskLevel {
            universe: "u1".into(),
            channel,
            value,
        };
        s.apply(Command::Desk(vec![
            level(1, 255),
            level(2, 100),
            level(3, 200),
        ]));
        s
    }

    fn frame(s: &State) -> Frame {
        s.render(Instant::now())[0].1
    }

    #[test]
    fn desk_goes_over_the_defaults() {
        let mut s = state();
        assert_eq!(frame(&s)[..5], [255, 100, 200, 255, 0]);
        let mut levels = s.desk.clone();
        levels.push(DeskLevel {
            universe: "u1".into(),
            channel: 4,
            value: 10,
        });
        s.apply(Command::Desk(levels));
        assert_eq!(frame(&s)[3], 10);
        // Solto (o "Zerar" da Mesa simples), volta ao padrão
        s.apply(Command::Desk(vec![]));
        assert_eq!(frame(&s)[..4], [0, 0, 0, 255]);
    }

    #[test]
    fn grand_master_reduces_only_intensity_rounding_like_qlc() {
        let mut s = state();
        s.apply(Command::GrandMaster(128));
        let f = frame(&s);
        // 255 × 128/255 = 128; 100 × 128/255 = 50,2 → 50; o pan fica
        assert_eq!(f[..3], [128, 50, 200]);
        assert_eq!(reduce(1, 128), 1);
        assert_eq!(reduce(255, 255), 255);
        assert_eq!(reduce(200, 0), 0);
    }

    #[test]
    fn blackout_zeroes_intensity_and_keeps_position() {
        let mut s = state();
        s.apply(Command::Blackout(true));
        assert_eq!(frame(&s)[..4], [0, 0, 200, 255]);
        s.apply(Command::Blackout(false));
        assert_eq!(frame(&s)[..3], [255, 100, 200]);
    }

    #[test]
    fn test_pattern_replaces_the_desk() {
        let mut s = state();
        s.apply(Command::Test(Some(TestPattern::Ramp)));
        let since = s.test.unwrap().1;
        let at = |ms| s.render(since + Duration::from_millis(ms))[0].1;
        assert_eq!(at(0)[..3], [0, 0, 0]);
        assert_eq!(at(2000)[..3], [255, 255, 255]);
        assert_eq!(at(1000)[0], 127);
        s.apply(Command::Test(Some(TestPattern::Chase)));
        let since = s.test.unwrap().1;
        // O canal 9 está 1 s atrasado: no tempo 1 s ele começa e o canal 1 está no meio da subida
        let f = s.render(since + Duration::from_millis(1000))[0].1;
        assert_eq!((f[0], f[8]), (127, 0));
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

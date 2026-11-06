//! Phaser no motor (P4): cada camada leva um atributo das fixtures do grupo por uma onda, e cada
//! fixture anda a onda atrasada em relação à anterior. As camadas absolutas ficam num fader só e as
//! relativas em outro, como as cenas e o EFX: a criação entra a partir do que está embaixo e, ao parar,
//! volta no fade out até lá. As relativas somam a onda ao que está embaixo, com o zero em 127.

use std::f64::consts::TAU;

use rlc_core::{PhaserLayer, WaveShape};

use crate::efx::dmx;
use crate::{Ctx, Imposed, SceneLevel, Span, TICK, Times, is_valid};

/// Um Phaser com os canais já nos endereços DMX (o motor não conhece fixtures).
#[derive(Clone)]
pub struct PhaserProgram {
    pub tracks: Vec<PhaserTrack>,
    /// Graus de atraso de uma fixture para a próxima.
    pub spread: i16,
    /// Duração de um ciclo.
    pub cycle: Span,
    pub fade_in: Span,
    pub fade_out: Span,
}

/// Uma camada e o canal do atributo dela em cada fixture do grupo, na ordem da grade (o valor não
/// conta). A posição na lista é a da fixture na onda.
#[derive(Clone, PartialEq, Debug)]
pub struct PhaserTrack {
    pub layer: PhaserLayer,
    pub targets: Vec<SceneLevel>,
}

/// O fader de um tipo de camada e de que camada e fixture é cada canal dele, na ordem do fader.
struct Part {
    serial: u64,
    slots: Vec<(usize, usize)>,
}

pub(crate) struct PhaserRunner {
    phaser: PhaserProgram,
    /// Tempos impostos pelo passo de um chaser (o Phaser não tem delay nem curva próprios).
    imposed: Imposed,
    /// Ciclos desde o começo: a fração é a posição na onda.
    turns: f64,
    /// As camadas absolutas e as relativas; `None` sem camadas do tipo.
    parts: [Option<Part>; 2],
}

impl PhaserRunner {
    pub(crate) fn new(phaser: PhaserProgram, imposed: Imposed, ctx: &mut Ctx) -> Self {
        let mut runner = Self {
            phaser,
            imposed,
            turns: 0.0,
            parts: [None, None],
        };
        runner.start(ctx);
        runner.place(ctx);
        runner
    }

    /// Um fader para as camadas absolutas e outro para as relativas.
    fn start(&mut self, ctx: &mut Ctx) {
        let times = self.times();
        let tracks = &self.phaser.tracks;
        self.parts = [false, true].map(|relative| {
            let slots: Vec<(usize, usize)> = tracks
                .iter()
                .enumerate()
                .filter(|(_, t)| t.layer.relative == relative)
                .flat_map(|(k, t)| {
                    let valid = t.targets.iter().enumerate().filter(|(_, l)| is_valid(l));
                    valid.map(move |(i, _)| (k, i))
                })
                .collect();
            if slots.is_empty() {
                return None;
            }
            let levels = slots.iter().map(|&(k, i)| tracks[k].targets[i].clone());
            Some(Part {
                serial: ctx.push(levels, times, relative),
                slots,
            })
        });
    }

    /// Um quadro: põe cada canal no ponto da onda e anda o ciclo.
    pub(crate) fn tick(&mut self, ctx: &mut Ctx) {
        self.place(ctx);
        // O ciclo conta pela duração de agora: mudar a duração (ou o BPM, no ciclo em batidas)
        // continua do ponto onde está, sem salto
        let cycle = self.phaser.cycle.duration(ctx.beat);
        if !cycle.is_zero() {
            self.turns += TICK.as_secs_f64() / cycle.as_secs_f64();
        }
    }

    fn place(&mut self, ctx: &mut Ctx) {
        let PhaserProgram { tracks, spread, .. } = &self.phaser;
        for part in self.parts.iter().flatten() {
            let Some(fader) = ctx.faders.iter_mut().find(|f| f.serial == part.serial) else {
                continue;
            };
            for (f, &(k, i)) in fader.channels.iter_mut().zip(&part.slots) {
                let layer = &tracks[k].layer;
                // A fixture i anda a onda atrasada em i × defasagem, e a camada, na fase dela
                let delay = (f64::from(layer.phase) + i as f64 * f64::from(*spread)) / 360.0;
                let value = wave(layer, (self.turns - delay).rem_euclid(1.0));
                f.level.value = dmx(value, f.level.fine.is_some());
            }
        }
    }

    pub(crate) fn stop(&self, ctx: &mut Ctx) {
        for part in self.parts.iter().flatten() {
            ctx.stop_fader(part.serial);
        }
    }

    /// Os fades do Phaser, com os impostos pelo passo de um chaser por cima.
    fn times(&self) -> Times {
        let own = Times::fades(self.phaser.fade_in, self.phaser.fade_out);
        self.imposed.on(own)
    }

    /// Edição com o Phaser rodando: vale na hora, sem recomeçar o ciclo. Outras fixtures ou canais, ou
    /// uma camada que virou absoluta ou relativa, recomeçam os faders (o que estava aceso sai no fade
    /// out).
    pub(crate) fn update(&mut self, phaser: PhaserProgram, imposed: Imposed, ctx: &mut Ctx) {
        let (old, new) = (&self.phaser.tracks, &phaser.tracks);
        let same = old.len() == new.len()
            && old
                .iter()
                .zip(new)
                .all(|(a, b)| a.layer.relative == b.layer.relative && a.targets == b.targets);
        if !same {
            self.stop(ctx);
        }
        self.phaser = phaser;
        self.imposed = imposed;
        if !same {
            self.start(ctx);
            return;
        }
        let times = self.times();
        for part in self.parts.iter().flatten() {
            if let Some(fader) = ctx.faders.iter_mut().find(|f| f.serial == part.serial) {
                fader.times = times;
            }
        }
    }
}

/// O valor da camada (na escala DMX, do piso ao teto) na posição `u` do ciclo, de 0 a 1.
fn wave(layer: &PhaserLayer, u: f64) -> f64 {
    let level = match layer.shape {
        WaveShape::Sine => 0.5 - 0.5 * (u * TAU).cos(),
        WaveShape::Triangle => 1.0 - (2.0 * u - 1.0).abs(),
        WaveShape::SawUp => u,
        WaveShape::SawDown => 1.0 - u,
        WaveShape::Pulse if u < f64::from(layer.width) / 100.0 => 1.0,
        WaveShape::Pulse => 0.0,
    };
    let (low, high) = (f64::from(layer.low), f64::from(layer.high));
    low + (high - low) * level
}

#[cfg(test)]
mod tests {
    use std::time::Instant;

    use rlc_core::{Attribute, Direction, RunOrder};

    use super::*;
    use crate::tests::{run, scene};
    use crate::{ChaserProgram, Command, Program, State, Step, UniversePatch};

    /// Universo de teste: os canais 1 a 4 são de intensidade (HTP), o resto não; todos fazem fade.
    fn patched() -> State {
        let mut intensity = Box::new([false; 512]);
        intensity[..4].fill(true);
        let patch = UniversePatch {
            id: "u1".into(),
            intensity,
            fades: Box::new([true; 512]),
            defaults: Box::new([0; 512]),
        };
        let mut s = State::default();
        s.apply(Command::Patch(vec![patch]), Instant::now());
        s
    }

    fn level(channel: u16, fine: Option<u16>) -> SceneLevel {
        SceneLevel {
            universe: "u1".into(),
            channel,
            fine,
            value: 0,
        }
    }

    /// Uma camada de 0 a 200 que começa no piso.
    fn layer(shape: WaveShape) -> PhaserLayer {
        PhaserLayer {
            attribute: Attribute::Dimmer,
            shape,
            width: 25,
            low: 0,
            high: 200,
            phase: 0,
            relative: false,
        }
    }

    /// Uma camada nos canais `1..=n` (uma fixture cada), com o ciclo em 200 quadros (4 s) e sem fades.
    fn phaser(shape: WaveShape, n: u16) -> PhaserProgram {
        PhaserProgram {
            tracks: vec![PhaserTrack {
                layer: layer(shape),
                targets: (1..=n).map(|c| level(c, None)).collect(),
            }],
            spread: 0,
            cycle: Span::Ms(4000),
            fade_in: Span::Ms(0),
            fade_out: Span::Ms(0),
        }
    }

    fn started(programs: Vec<(&str, Program)>) -> (State, Instant) {
        let mut s = patched();
        let t0 = Instant::now();
        let runs = programs.into_iter().map(|(id, p)| run(id, p)).collect();
        s.apply(Command::Running(runs), t0);
        (s, t0)
    }

    /// Os `n` primeiros canais, quadro a quadro. O ciclo só anda a cada quadro renderizado.
    fn frames(s: &mut State, t0: Instant, ticks: std::ops::Range<u32>, n: usize) -> Vec<Vec<i32>> {
        ticks
            .map(|k| {
                let f = s.render(t0 + TICK * k)[0].1;
                f[..n].iter().map(|&v| i32::from(v)).collect()
            })
            .collect()
    }

    /// Os `n` primeiros canais no quadro `tick`, depois de renderizar todos os de antes.
    fn at(p: &PhaserProgram, tick: u32, n: usize) -> Vec<i32> {
        let (mut s, t0) = started(vec![("p", Program::Phaser(p.clone()))]);
        frames(&mut s, t0, 0..tick + 1, n).pop().unwrap()
    }

    /// Os valores batem, a menos de 1 (a conta trunca).
    fn near(got: &[i32], want: &[i32]) {
        assert!(
            got.len() == want.len() && got.iter().zip(want).all(|(a, b)| (a - b).abs() <= 1),
            "veio {got:?}, esperava {want:?}"
        );
    }

    #[test]
    fn each_shape_goes_from_the_floor_to_the_ceiling() {
        // Nos quadros 0, 25, 60, 100 e 150 de um ciclo de 200
        let shape_at = |shape| {
            let (mut s, t0) = started(vec![("p", Program::Phaser(phaser(shape, 1)))]);
            let p = frames(&mut s, t0, 0..151, 1);
            [0, 25, 60, 100, 150].map(|t| p[t][0])
        };
        near(&shape_at(WaveShape::Sine), &[0, 29, 131, 200, 100]);
        near(&shape_at(WaveShape::Triangle), &[0, 50, 120, 200, 100]);
        near(&shape_at(WaveShape::SawUp), &[0, 25, 60, 100, 150]);
        near(&shape_at(WaveShape::SawDown), &[200, 175, 140, 100, 50]);
        // O pulso de 25% fica no alto nos primeiros 50 quadros
        near(&shape_at(WaveShape::Pulse), &[200, 200, 0, 0, 0]);
    }

    #[test]
    fn the_fixtures_follow_one_another_by_the_spread() {
        // 4 fixtures: com 90° cada uma anda 50 quadros atrás da anterior
        let mut p = phaser(WaveShape::SawUp, 4);
        p.spread = 90;
        near(&at(&p, 100, 4), &[100, 50, 0, 150]);
        // Sem defasagem, juntas
        p.spread = 0;
        near(&at(&p, 100, 4), &[100; 4]);
        // Negativa, a onda anda para o outro lado
        p.spread = -90;
        near(&at(&p, 100, 4), &[100, 150, 0, 50]);
    }

    #[test]
    fn the_layer_phase_delays_the_whole_layer() {
        let mut p = phaser(WaveShape::SawUp, 2);
        p.tracks[0].layer.phase = 180;
        p.spread = 90;
        near(&at(&p, 0, 2), &[100, 50]);
    }

    #[test]
    fn a_layer_only_moves_the_fixtures_that_have_its_attribute() {
        // O dimmer em 2 fixtures (canais 1 e 2) e o pan só numa (canal 10, fino 11)
        let mut p = phaser(WaveShape::SawUp, 2);
        p.tracks.push(PhaserTrack {
            layer: PhaserLayer {
                high: 255,
                ..layer(WaveShape::SawUp)
            },
            targets: vec![level(10, Some(11))],
        });
        let f = at(&p, 100, 11);
        // O pan em 16 bits: a meio caminho de 0 a 255, 127,5 em 256 passos
        assert_eq!((f[9], f[10]), (127, 128));
        assert_eq!(f[2], 0);
    }

    #[test]
    fn the_cycle_follows_the_beats_and_changes_without_jumping() {
        // O ciclo em 4 batidas: 2 s a 120 BPM e 4 s a 60, contando do ponto onde está
        let mut p = phaser(WaveShape::SawUp, 1);
        p.cycle = Span::Beats(4000);
        let (mut s, t0) = started(vec![]);
        s.apply(Command::Bpm(120), t0);
        s.apply(Command::Running(vec![run("p", Program::Phaser(p))]), t0);
        near(&frames(&mut s, t0, 0..26, 1).pop().unwrap(), &[50]);
        s.apply(Command::Bpm(60), t0 + TICK * 26);
        // Depois de 26 quadros de 1/100, mais 50 de 1/200 do ciclo
        near(&frames(&mut s, t0, 26..77, 1).pop().unwrap(), &[102]);

        // Na duração, a metade do ciclo anda o dobro por quadro
        let p = phaser(WaveShape::SawUp, 1);
        let (mut s, t0) = started(vec![("p", Program::Phaser(p.clone()))]);
        frames(&mut s, t0, 0..30, 1);
        let half = PhaserProgram {
            cycle: Span::Ms(2000),
            ..p
        };
        s.apply(
            Command::Running(vec![run("p", Program::Phaser(half))]),
            t0 + TICK * 30,
        );
        near(&frames(&mut s, t0, 30..32, 1).concat(), &[30, 32]);
    }

    #[test]
    fn enters_from_what_is_below_and_leaves_back_to_it() {
        // Uma cena põe o canal 10 em 50 (LTP); o Phaser entra em 20 quadros e sai em 20
        let below = || scene(&[(10, 50)], 0);
        let mut p = phaser(WaveShape::SawUp, 0);
        p.tracks[0].targets = vec![level(10, None)];
        p.fade_in = Span::Ms(400);
        p.fade_out = Span::Ms(400);
        let (mut s, t0) = started(vec![("s", below())]);
        s.render(t0);
        let both = vec![run("s", below()), run("p", Program::Phaser(p))];
        s.apply(Command::Running(both), t0 + TICK);
        let p = frames(&mut s, t0, 1..21, 10);
        // No primeiro quadro, um vigésimo do caminho entre o 50 e o ponto da onda (0)
        assert_eq!(p[0][9], 48);
        near(&[p[19][9]], &[19]);
        // Parado, sai do ponto onde está e volta ao 50 da cena
        s.apply(Command::Running(vec![run("s", below())]), t0 + TICK * 21);
        let p = frames(&mut s, t0, 21..42, 10);
        assert!(p[0][9] > 19 && p[0][9] < 50);
        assert_eq!(p[20][9], 50);
        assert!(s.faders.iter().all(|f| f.owner == "s"));
    }

    #[test]
    fn the_relative_layer_adds_to_what_is_below() {
        // Uma cena põe o canal 10 em 50; a camada relativa balança de 0 a +20 em volta dele
        let mut p = phaser(WaveShape::SawUp, 0);
        p.tracks[0].targets = vec![level(10, None)];
        p.tracks[0].layer = PhaserLayer {
            relative: true,
            low: 127,
            high: 147,
            ..layer(WaveShape::SawUp)
        };
        let (mut s, t0) = started(vec![
            ("p", Program::Phaser(p)),
            ("s", scene(&[(10, 50)], 0)),
        ]);
        // A cena iniciada depois não esconde o relativo: ele fica por cima de todas as criações
        let p = frames(&mut s, t0, 0..101, 10);
        assert_eq!(p[0][9], 50);
        near(&[p[100][9]], &[60]);
        // Parados, tudo volta ao padrão
        s.apply(Command::Running(vec![]), t0 + TICK * 101);
        assert_eq!(frames(&mut s, t0, 101..103, 10)[1][9], 0);
    }

    #[test]
    fn the_pause_holds_the_cycle_and_going_on_follows_from_there() {
        let p = || Program::Phaser(phaser(WaveShape::SawUp, 1));
        let (mut s, t0) = started(vec![("p", p())]);
        frames(&mut s, t0, 0..50, 1);
        let mut paused = run("p", p());
        paused.paused = true;
        s.apply(Command::Running(vec![paused]), t0 + TICK * 50);
        let held = frames(&mut s, t0, 50..60, 1).concat();
        assert!(held.iter().all(|&v| v == held[0]), "{held:?}");
        s.apply(Command::Running(vec![run("p", p())]), t0 + TICK * 60);
        near(&frames(&mut s, t0, 60..61, 1)[0], &[held[0] + 1]);
    }

    #[test]
    fn live_edits_keep_the_cycle_going_and_other_channels_restart() {
        let p = phaser(WaveShape::SawUp, 1);
        let (mut s, t0) = started(vec![("p", Program::Phaser(p.clone()))]);
        frames(&mut s, t0, 0..20, 1);
        // O teto muda no quadro seguinte, no mesmo ponto do ciclo (20/200 de 100)
        let mut edited = p;
        edited.tracks[0].layer.high = 100;
        s.apply(
            Command::Running(vec![run("p", Program::Phaser(edited.clone()))]),
            t0 + TICK * 20,
        );
        near(&frames(&mut s, t0, 20..21, 1)[0], &[10]);
        // Outro canal: o antigo apaga (sem fade out) e o novo anda o ciclo do ponto onde está
        let mut moved = edited;
        moved.tracks[0].targets = vec![level(2, None)];
        s.apply(
            Command::Running(vec![run("p", Program::Phaser(moved))]),
            t0 + TICK * 21,
        );
        near(&frames(&mut s, t0, 21..22, 2)[0], &[0, 10]);
    }

    #[test]
    fn it_lights_in_a_chaser_step_and_in_a_collection() {
        let lit = |program| {
            let (mut s, t0) = started(vec![("c", program)]);
            let p = frames(&mut s, t0, 0..40, 1).concat();
            p.windows(2).filter(|w| w[0] != w[1]).count() > 5
        };
        let step = Step {
            program: Program::Phaser(phaser(WaveShape::SawUp, 1)),
            times: Imposed::default(),
            hold: Span::Ms(60_000),
        };
        let chaser = ChaserProgram {
            steps: vec![step],
            run_order: RunOrder::Loop,
            direction: Direction::Forward,
            keep_last: false,
        };
        assert!(lit(Program::Chaser(chaser)));
        let p = Program::Phaser(phaser(WaveShape::SawUp, 1));
        assert!(lit(Program::Collection(vec![p])));
    }
}

//! EFX no motor, como o `EFXFixture` do QLC+: cada fixture anda pelo desenho, e o ponto vira pan e
//! tilt. Cada fixture tem um fader da mistura: no fade in, ela sai da posição em que está e entra no
//! desenho; parada, volta no fade out ao que está embaixo, como as cenas. No relativo, o desenho é
//! somado à posição de baixo (a camada logo acima da prévia).

use std::f64::consts::{FRAC_PI_2, PI, TAU};

use rlc_core::{Direction, Efx, EfxAlgorithm, Propagation, RunOrder};

use crate::{Ctx, EfxProgram, Imposed, TICK, Times, valid};

pub(crate) struct EfxRunner {
    efx: EfxProgram,
    /// Tempos impostos pelo passo de um chaser (o EFX não tem delay nem curva próprios).
    imposed: Imposed,
    /// Voltas desde o começo: a fração é a posição no desenho.
    turns: f64,
    /// Fader de cada fixture, na ordem do EFX; `None` até a vez dela ("Em sequência").
    faders: Vec<Option<u64>>,
    pub(crate) done: bool,
}

impl EfxRunner {
    /// As fixtures que já começam ganham o fader agora, para a mistura seguir a ordem de início.
    pub(crate) fn new(efx: EfxProgram, imposed: Imposed, ctx: &mut Ctx) -> Self {
        let mut runner = Self {
            faders: vec![None; efx.heads.len()],
            efx,
            imposed,
            turns: 0.0,
            done: false,
        };
        runner.place(ctx);
        runner
    }

    /// Um quadro: põe cada fixture no ponto dela e anda o desenho. No Single Shot, quando todas
    /// deram a volta, o EFX para com o fade out.
    pub(crate) fn tick(&mut self, ctx: &mut Ctx) {
        if self.done {
            return;
        }
        if self.place(ctx) {
            self.stop(ctx);
            self.done = true;
            return;
        }
        // A volta conta pela duração de agora: mudar a duração (ou o BPM, na volta em batidas)
        // continua do ponto onde está, sem salto
        let cycle = self.efx.cycle.duration(ctx.beat);
        if !cycle.is_zero() {
            self.turns += TICK.as_secs_f64() / cycle.as_secs_f64();
        }
    }

    /// Põe cada fixture no ponto dela; a que chegou a vez ganha o fader. Devolve se todas já deram a
    /// volta do Single Shot.
    fn place(&mut self, ctx: &mut Ctx) -> bool {
        let EfxProgram { efx, heads, .. } = &self.efx;
        let n = heads.len() as f64;
        let single = efx.run_order == RunOrder::SingleShot;
        let mut finished = single;
        for (i, (fixture, head)) in efx.fixtures.iter().zip(heads).enumerate() {
            let share = i as f64 / n;
            // Quanto do desenho a fixture já andou. "Em sequência", cada uma começa 1/n da volta
            // depois da anterior e, até a vez dela, fica onde está
            let serial = efx.propagation == Propagation::Serial;
            let progress = self.turns - if serial { share } else { 0.0 };
            finished &= progress >= 1.0;
            if progress < 0.0 {
                continue;
            }
            // Depois da volta do Single Shot, fica no ponto em que começou, como no QLC+
            let progress = if single { progress.min(1.0) } else { progress };
            // "Defasadas", cada uma 1/n da volta à frente da anterior
            let asymmetric = efx.propagation == Propagation::Asymmetric;
            let at = progress + if asymmetric { share } else { 0.0 };
            let lap = at.floor();
            let mut angle = (at - lap) * TAU;
            // No vai e volta, as voltas ímpares andam para trás
            if efx.run_order == RunOrder::PingPong && lap % 2.0 != 0.0 {
                angle = TAU - angle;
            }
            if (efx.direction == Direction::Backward) != fixture.reverse {
                angle = reverse(efx.algorithm, angle);
            }
            angle += f64::from(fixture.start_offset % 360).to_radians();
            if angle >= TAU {
                angle -= TAU;
            }
            let (x, y) = point(efx, angle);

            let serial = match self.faders[i] {
                Some(serial) => serial,
                None => {
                    let levels = valid(head.iter().flatten().cloned().collect());
                    let times = self.times();
                    let serial = ctx.push(levels, times, efx.relative);
                    self.faders[i] = Some(serial);
                    serial
                }
            };
            let Some(fader) = ctx.faders.iter_mut().find(|f| f.serial == serial) else {
                continue;
            };
            for (level, v) in head.iter().zip([x, y]) {
                let Some(level) = level else { continue };
                let channel = fader.channels.iter_mut().find(|f| {
                    f.level.channel == level.channel && f.level.universe == level.universe
                });
                if let Some(f) = channel {
                    f.level.value = dmx(v, level.fine.is_some());
                }
            }
        }
        finished
    }

    pub(crate) fn stop(&self, ctx: &mut Ctx) {
        for serial in self.faders.iter().flatten() {
            ctx.stop_fader(*serial);
        }
    }

    /// Os fades do EFX, com os impostos pelo passo de um chaser por cima.
    fn times(&self) -> Times {
        let own = Times::fades(self.efx.fade_in, self.efx.fade_out);
        self.imposed.on(own)
    }

    /// Edição com o EFX rodando: vale na hora, sem recomeçar o desenho. A fixture incluída ganha o
    /// fader na vez dela; a retirada (ou com outro endereço) faz o fade out. Trocar o relativo
    /// recomeça os faders.
    pub(crate) fn update(&mut self, efx: EfxProgram, imposed: Imposed, ctx: &mut Ctx) {
        let restart = efx.efx.relative != self.efx.efx.relative;
        let old = std::mem::take(&mut self.faders);
        self.faders = efx
            .efx
            .fixtures
            .iter()
            .zip(&efx.heads)
            .map(|(fixture, head)| {
                let fixtures = &self.efx.efx.fixtures;
                let i = fixtures.iter().position(|o| o.target == fixture.target)?;
                if restart || self.efx.heads[i] != *head {
                    return None;
                }
                old[i]
            })
            .collect();
        for serial in old.iter().flatten() {
            if !self.faders.contains(&Some(*serial)) {
                ctx.stop_fader(*serial);
            }
        }
        self.efx = efx;
        self.imposed = imposed;
        let times = self.times();
        for fader in ctx.faders.iter_mut() {
            if self.faders.contains(&Some(fader.serial)) {
                fader.times = times;
            }
        }
    }
}

/// Ponto do desenho (0 a 255) em DMX: em 16 bits com o fino. O que passa do curso fica na ponta.
fn dmx(v: f64, fine: bool) -> u16 {
    let v = v.clamp(0.0, 255.0);
    if fine { (v * 256.0) as u16 } else { v as u16 }
}

/// O desenho ao contrário, como o `calculateDirection` do QLC+. A linha passa pelos mesmos pontos na
/// ida e na volta: ao contrário, ela anda meia volta à frente.
fn reverse(algorithm: EfxAlgorithm, angle: f64) -> f64 {
    match algorithm {
        EfxAlgorithm::Line if angle > PI => angle - PI,
        EfxAlgorithm::Line => angle + PI,
        _ => TAU - angle,
    }
}

/// Ponto do desenho no ângulo (0 a 2π), já girado, escalado e levado ao centro, na escala do pan e do
/// tilt (0 a 255). As contas do `EFX::calculatePoint` do QLC+ (as mesmas da prévia da tela).
fn point(efx: &Efx, angle: f64) -> (f64, f64) {
    use EfxAlgorithm::*;
    let (x, y) = match efx.algorithm {
        Circle => ((angle + FRAC_PI_2).cos(), angle.cos()),
        Eight => ((angle * 2.0 + FRAC_PI_2).cos(), angle.cos()),
        Line => (angle.cos(), angle.cos()),
        Line2 => (angle / PI - 1.0, angle / PI - 1.0),
        Diamond => ((angle - FRAC_PI_2).cos().powi(3), angle.cos().powi(3)),
        // Nos quadrados, cada quarto da volta é um lado (f anda de -1 a 1 dentro do quarto)
        Square | SquareTrue => {
            let quarter = (angle / FRAC_PI_2).floor().clamp(0.0, 3.0);
            let f = (angle / FRAC_PI_2 - quarter) * 2.0 - 1.0;
            let sides = match efx.algorithm {
                Square => [(f, 1.0), (1.0, -f), (-f, -1.0), (-1.0, f)],
                _ => [(1.0, 1.0), (1.0, -1.0), (-1.0, -1.0), (-1.0, 1.0)],
            };
            sides[quarter as usize]
        }
        SquareChoppy => (angle.cos().round(), angle.sin().round()),
        Leaf => ((angle + FRAC_PI_2).cos().powi(5), angle.cos()),
        Lissajous => (
            wave(efx.x_frequency, efx.x_phase, angle),
            wave(efx.y_frequency, efx.y_phase, angle),
        ),
    };
    scale(efx, x, y)
}

/// Um eixo do Lissajous: a onda na frequência e na fase dela. Com a frequência 0, um vai e vem em
/// linha reta.
fn wave(frequency: u8, phase: u16, angle: f64) -> f64 {
    let phase = f64::from(phase).to_radians();
    if frequency > 0 {
        return (f64::from(frequency) * angle - phase).cos();
    }
    let t = ((angle + phase) / PI) % 2.0;
    (if t < 1.0 { t } else { 2.0 - t }) * 2.0 - 1.0
}

/// Gira, escala pela largura e pela altura e leva ao centro, como o `rotateAndScale` do QLC+.
fn scale(efx: &Efx, x: f64, y: f64) -> (f64, f64) {
    let (sin, cos) = f64::from(efx.rotation).to_radians().sin_cos();
    let (w, h) = (f64::from(efx.width), f64::from(efx.height));
    (
        f64::from(efx.x_offset) + x * cos * w + y * sin * h,
        f64::from(efx.y_offset) - x * sin * w + y * cos * h,
    )
}

#[cfg(test)]
pub(crate) mod tests {
    use std::time::Instant;

    use rlc_core::{EfxFixture, EfxTarget, Timing};

    use super::*;
    use crate::tests::{run, scene};
    use crate::{
        ChaserProgram, Command, DeskLevel, Program, SceneLevel, Span, State, Step, UniversePatch,
    };
    use EfxAlgorithm::*;

    /// O EFX padrão do QLC+: o desenho inteiro (largura e altura 127) no centro.
    pub(crate) fn qlc(algorithm: EfxAlgorithm) -> Efx {
        Efx {
            algorithm,
            width: 127,
            height: 127,
            x_offset: 127,
            y_offset: 127,
            rotation: 0,
            x_frequency: 2,
            y_frequency: 3,
            x_phase: 90,
            y_phase: 0,
            relative: false,
            propagation: Propagation::Parallel,
            fixtures: vec![],
            run_order: RunOrder::Loop,
            direction: Direction::Forward,
            timing: Timing {
                fade_in_ms: 0,
                hold_ms: 4000,
                fade_out_ms: 0,
            },
        }
    }

    /// Os ângulos da prévia do QLC+: 512 pontos somados em `float`, como lá.
    fn preview_angles() -> Vec<f64> {
        let step = (1.0 / (512.0 / TAU)) as f32;
        let mut angle = 0f32;
        (0..512)
            .map(|_| {
                let a = f64::from(angle);
                angle += step;
                a
            })
            .collect()
    }

    #[test]
    fn shapes_match_the_qlc_preview() {
        // Os pontos 0, 64, 128… dos testes `preview…` do `efx_test` do QLC+
        let cases = [
            (
                Circle,
                [
                    (127, 254),
                    (37, 217),
                    (0, 127),
                    (37, 37),
                    (127, 0),
                    (217, 37),
                    (254, 127),
                    (217, 217),
                ],
            ),
            (
                Eight,
                [
                    (127, 254),
                    (0, 217),
                    (127, 127),
                    (254, 37),
                    (127, 0),
                    (0, 37),
                    (127, 127),
                    (254, 217),
                ],
            ),
            (
                Line,
                [
                    (254, 254),
                    (217, 217),
                    (127, 127),
                    (37, 37),
                    (0, 0),
                    (37, 37),
                    (127, 127),
                    (217, 217),
                ],
            ),
            (
                Line2,
                [
                    (0, 0),
                    (32, 32),
                    (64, 64),
                    (95, 95),
                    (127, 127),
                    (159, 159),
                    (191, 191),
                    (222, 222),
                ],
            ),
            (
                Diamond,
                [
                    (127, 254),
                    (172, 172),
                    (254, 127),
                    (172, 82),
                    (127, 0),
                    (82, 82),
                    (0, 127),
                    (82, 172),
                ],
            ),
            (
                Square,
                [
                    (0, 254),
                    (127, 254),
                    (254, 254),
                    (254, 127),
                    (254, 0),
                    (127, 0),
                    (0, 0),
                    (0, 127),
                ],
            ),
            (
                SquareChoppy,
                [
                    (254, 127),
                    (254, 254),
                    (127, 254),
                    (0, 254),
                    (0, 127),
                    (0, 0),
                    (127, 0),
                    (254, 0),
                ],
            ),
            (
                SquareTrue,
                [
                    (254, 254),
                    (254, 254),
                    (254, 0),
                    (254, 0),
                    (0, 0),
                    (0, 0),
                    (0, 254),
                    (0, 254),
                ],
            ),
            (
                Leaf,
                [
                    (127, 254),
                    (105, 217),
                    (0, 127),
                    (105, 37),
                    (127, 0),
                    (149, 37),
                    (254, 127),
                    (149, 217),
                ],
            ),
            (
                Lissajous,
                [
                    (127, 254),
                    (254, 37),
                    (127, 127),
                    (0, 217),
                    (127, 0),
                    (254, 217),
                    (127, 127),
                    (0, 37),
                ],
            ),
        ];
        let angles = preview_angles();
        for (algorithm, points) in cases {
            for (j, expected) in points.into_iter().enumerate() {
                let (x, y) = point(&qlc(algorithm), angles[64 * j]);
                assert_eq!(
                    (x.round() as i32, y.round() as i32),
                    expected,
                    "{algorithm:?} {j}"
                );
            }
        }
    }

    #[test]
    fn backwards_goes_the_other_way_and_the_line_half_a_turn_ahead() {
        let a = |k: usize| k as f64 * TAU / 512.0;
        for algorithm in [Circle, Eight, Diamond, Leaf, Lissajous] {
            for i in 1..512 {
                // Como os `preview…Backwards`: para trás, o ponto i é o 512 - i de para a frente
                let (b, f) = (
                    point(&qlc(algorithm), reverse(algorithm, a(512 - i))),
                    point(&qlc(algorithm), a(i)),
                );
                assert!(
                    (b.0 - f.0).abs() < 1e-6 && (b.1 - f.1).abs() < 1e-6,
                    "{algorithm:?} {i}"
                );
            }
        }
        // A linha para trás anda meia volta à frente
        let line = |angle| point(&qlc(Line), angle);
        for i in 0..256 {
            for (b, f) in [(i + 256, i), (i, i + 256)] {
                let (b, f) = (line(reverse(Line, a(b))), line(a(f)));
                assert!((b.0 - f.0).abs() < 1e-6 && (b.1 - f.1).abs() < 1e-6, "{i}");
            }
        }
        // Lissajous com a frequência 0: um vai e vem em linha reta
        let mut e = qlc(Lissajous);
        (e.x_frequency, e.x_phase) = (0, 0);
        let xs = [0.0, PI / 2.0, PI, 1.5 * PI].map(|a| point(&e, a).0.round());
        assert_eq!(xs, [0.0, 127.0, 254.0, 127.0]);
    }

    #[test]
    fn rotation_scale_and_offset_like_qlc() {
        // O primeiro ponto do círculo, nos números do `rotateAndScale` do QLC+
        let (x, y) = (-0.125333, 0.992115);
        for (rotation, expected) in [
            (0, (111, 253)),
            (90, (253, 143)),
            (180, (143, 1)),
            (270, (1, 111)),
            (45, (205, 227)),
        ] {
            let e = Efx {
                rotation,
                ..qlc(Circle)
            };
            let (sx, sy) = scale(&e, x, y);
            assert_eq!(
                (sx.round() as i32, sy.round() as i32),
                expected,
                "{rotation}°"
            );
        }
        // O `widthHeightOffset`: a largura e a altura são para cada lado do centro
        let bounds = |e: &Efx| {
            let points: Vec<(f64, f64)> =
                preview_angles().into_iter().map(|a| point(e, a)).collect();
            let max =
                |f: fn(&(f64, f64)) -> f64| points.iter().map(f).fold(f64::MIN, f64::max).round();
            let min =
                |f: fn(&(f64, f64)) -> f64| points.iter().map(f).fold(f64::MAX, f64::min).round();
            (
                max(|p| p.0) - min(|p| p.0),
                max(|p| p.1) - min(|p| p.1),
                max(|p| p.0),
                max(|p| p.1),
            )
        };
        let mut e = Efx {
            width: 50,
            ..qlc(Circle)
        };
        assert_eq!(bounds(&e), (100.0, 254.0, 177.0, 254.0));
        e.height = 87;
        e.x_offset = 147;
        e.y_offset = 102;
        assert_eq!(bounds(&e), (100.0, 174.0, 197.0, 189.0));
    }

    /// Universo só de posição (sem intensidade), sem padrão.
    fn patched() -> State {
        let mut s = State::default();
        let patch = UniversePatch {
            id: "u1".into(),
            intensity: Box::new([false; 512]),
            fades: Box::new([true; 512]),
            defaults: Box::new([0; 512]),
        };
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

    /// Círculo em `n` fixtures de 8 bits (pan no canal 2k + 1, tilt no 2k + 2), com a volta em 200
    /// quadros (4 s) e sem fades.
    fn efx(n: u16) -> EfxProgram {
        let mut efx = qlc(Circle);
        efx.fixtures = (0..n)
            .map(|k| EfxFixture {
                target: EfxTarget::Fixture(format!("f{k}")),
                reverse: false,
                start_offset: 0,
                spread: 0,
            })
            .collect();
        EfxProgram {
            efx,
            heads: (0..n)
                .map(|k| [Some(level(2 * k + 1, None)), Some(level(2 * k + 2, None))])
                .collect(),
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

    /// Pan e tilt de cada uma das `n` fixtures, quadro a quadro.
    fn positions(
        s: &mut State,
        t0: Instant,
        ticks: std::ops::Range<u32>,
        n: usize,
    ) -> Vec<Vec<(i32, i32)>> {
        ticks
            .map(|k| {
                let f = s.render(t0 + TICK * k)[0].1;
                (0..n)
                    .map(|i| (i32::from(f[2 * i]), i32::from(f[2 * i + 1])))
                    .collect()
            })
            .collect()
    }

    /// O ponto do círculo depois de `turns` voltas, em DMX (a conta trunca, então ±1 vale).
    fn at(turns: f64) -> (i32, i32) {
        let (x, y) = point(&qlc(Circle), turns.fract() * TAU);
        (i32::from(dmx(x, false)), i32::from(dmx(y, false)))
    }

    fn close(a: (i32, i32), b: (i32, i32)) -> bool {
        (a.0 - b.0).abs() <= 1 && (a.1 - b.1).abs() <= 1
    }

    #[test]
    fn fixtures_spread_by_the_propagation() {
        // Defasadas: com 4 fixtures, cada uma 90° (50 quadros) à frente da anterior
        let mut e = efx(4);
        e.efx.propagation = Propagation::Asymmetric;
        let (mut s, t0) = started(vec![("e", Program::Efx(e.clone()))]);
        let p = positions(&mut s, t0, 0..200, 4);
        for (t, frame) in p.iter().take(50).enumerate() {
            for (k, &position) in frame.iter().enumerate() {
                let turns = (t + 50 * k) as f64 / 200.0;
                assert!(close(position, at(turns)), "quadro {t}, fixture {k}");
            }
        }
        // Juntas: todas no mesmo ponto
        e.efx.propagation = Propagation::Parallel;
        let (mut s, t0) = started(vec![("e", Program::Efx(e.clone()))]);
        for frame in positions(&mut s, t0, 0..100, 4) {
            assert!(frame.iter().all(|&p| p == frame[0]));
        }
        // Em sequência: a segunda espera 50 quadros onde está (no padrão) e começa do início
        e.efx.propagation = Propagation::Serial;
        let (mut s, t0) = started(vec![("e", Program::Efx(e))]);
        let p = positions(&mut s, t0, 0..100, 4);
        assert!(p[..50].iter().all(|frame| frame[1] == (0, 0)));
        for t in 0..50 {
            assert!(close(p[50 + t][1], p[t][0]), "quadro {t}");
        }
    }

    #[test]
    fn reverse_backward_and_ping_pong() {
        // A segunda fixture invertida anda o desenho ao contrário
        let mut e = efx(2);
        e.efx.fixtures[1].reverse = true;
        let (mut s, t0) = started(vec![("e", Program::Efx(e.clone()))]);
        let p = positions(&mut s, t0, 0..200, 2);
        for t in 1..200 {
            assert!(close(p[t][1], p[200 - t][0]), "quadro {t}");
        }
        // "Para trás" com "Inverter" volta a andar para a frente
        e.efx.direction = Direction::Backward;
        let (mut s, t0) = started(vec![("e", Program::Efx(e.clone()))]);
        let b = positions(&mut s, t0, 0..200, 2);
        for t in 0..200 {
            assert!(
                close(b[t][1], p[t][0]) && close(b[t][0], p[t][1]),
                "quadro {t}"
            );
        }
        // Vai e volta: depois de uma volta, o desenho volta pelo mesmo caminho
        let mut e = efx(1);
        e.efx.run_order = RunOrder::PingPong;
        let (mut s, t0) = started(vec![("e", Program::Efx(e))]);
        let p = positions(&mut s, t0, 0..300, 1);
        for t in 1..100 {
            assert!(close(p[200 + t][0], p[200 - t][0]), "quadro {t}");
        }
    }

    #[test]
    fn single_shot_ends_after_one_turn_and_goes_back() {
        let mut e = efx(1);
        e.efx.run_order = RunOrder::SingleShot;
        let (mut s, t0) = started(vec![("e", Program::Efx(e))]);
        let p = positions(&mut s, t0, 0..200, 1);
        assert!(close(p[199][0], at(199.0 / 200.0)));
        assert!(s.done().is_empty());
        // Na volta seguinte, para: sem fade out, a posição volta ao padrão na hora
        assert_eq!(positions(&mut s, t0, 200..201, 1)[0][0], (0, 0));
        assert_eq!(s.done(), ["e"]);
    }

    #[test]
    fn fine_channels_take_16_bits_and_the_drawing_stops_at_the_ends() {
        // Pan em 16 bits (1 e 2) e tilt em 8 (3), com o centro do pan fora do meio
        let mut e = efx(1);
        e.heads[0] = [Some(level(1, Some(2))), Some(level(3, None))];
        e.efx.x_offset = 200;
        let x = |turns: f64| point(&e.efx, turns * TAU).0;
        let (mut s, t0) = started(vec![("e", Program::Efx(e.clone()))]);
        let frame = |s: &mut State, tick: u32| s.render(t0 + TICK * tick)[0].1;
        frame(&mut s, 0);
        let f = frame(&mut s, 1);
        let x1 = x(1.0 / 200.0);
        assert_eq!((f[0], f[1]), (x1 as u8, ((x1 * 256.0) as u16 & 0xFF) as u8));
        assert_ne!(f[1], 0);
        // A 270°, o desenho iria a 327: fica na ponta
        for tick in 2..=150 {
            frame(&mut s, tick);
        }
        assert_eq!(frame(&mut s, 150)[..2], [255, 0]);
    }

    #[test]
    fn fade_in_leaves_the_position_and_fade_out_goes_back() {
        // Uma cena põe o pan da fixture em 50; o EFX entra em 20 quadros e sai em 20
        let mut e = efx(1);
        e.fade_in = Span::Ms(400);
        e.fade_out = Span::Ms(400);
        let pan = || scene(&[(1, 50)], 0);
        let (mut s, t0) = started(vec![("s", pan())]);
        s.render(t0);
        let both = vec![run("s", pan()), run("e", Program::Efx(e))];
        s.apply(Command::Running(both), t0 + TICK);
        let p = positions(&mut s, t0, 1..41, 1);
        // No primeiro quadro, um vigésimo do caminho entre o 50 e o ponto do desenho (127)
        assert_eq!(p[0][0].0, 50 + 77 / 20);
        assert!(close(p[19][0], at(19.0 / 200.0)));
        // Parado, sai do ponto onde está e volta ao 50 da cena
        s.apply(Command::Running(vec![run("s", pan())]), t0 + TICK * 41);
        let p = positions(&mut s, t0, 41..61, 1);
        let x = at(39.0 / 200.0).0;
        assert!(
            (p[0][0].0 - (x + (50 - x) / 20)).abs() <= 1,
            "{:?}",
            p[0][0]
        );
        assert_eq!(p[19][0].0, 50);
        assert!(s.faders.iter().all(|f| f.owner == "s"));
    }

    #[test]
    fn relative_adds_the_drawing_to_what_is_below() {
        // Um círculo pequeno (10 para cada lado) em volta da posição de uma cena: pan 50 e tilt 200
        let mut e = efx(1);
        e.efx.relative = true;
        (e.efx.width, e.efx.height) = (10, 10);
        let below = scene(&[(1, 50), (2, 200)], 0);
        let (mut s, t0) = started(vec![("e", Program::Efx(e.clone())), ("s", below)]);
        let offset = |turns: f64| {
            let (x, y) = point(&e.efx, turns * TAU);
            (
                i32::from(dmx(x, false)) - 127,
                i32::from(dmx(y, false)) - 127,
            )
        };
        // A cena iniciada depois não esconde o relativo: ele fica por cima de todas as criações
        let p = positions(&mut s, t0, 0..50, 1);
        for (t, frame) in p.iter().enumerate() {
            let (dx, dy) = offset(t as f64 / 200.0);
            assert!(close(frame[0], (50 + dx, 200 + dy)), "quadro {t}");
        }
        // Também em volta da prévia (o programador), e a Mesa simples fica por cima
        let desk = |channel, value| DeskLevel {
            universe: "u1".into(),
            channel,
            value,
        };
        s.apply(Command::Preview(vec![desk(1, 100)]), t0);
        s.apply(Command::Desk(vec![desk(2, 30)]), t0);
        let frame = positions(&mut s, t0, 50..51, 1)[0][0];
        assert!(close(frame, (100 + offset(0.25).0, 30)), "{frame:?}");
    }

    #[test]
    fn changing_the_duration_or_the_bpm_does_not_jump() {
        let e = efx(1);
        let (mut s, t0) = started(vec![("e", Program::Efx(e.clone()))]);
        positions(&mut s, t0, 0..30, 1);
        // Na metade da duração, continua de 30/200 da volta, agora 2 por quadro
        let half = EfxProgram {
            cycle: Span::Ms(2000),
            ..e.clone()
        };
        s.apply(
            Command::Running(vec![run("e", Program::Efx(half))]),
            t0 + TICK * 30,
        );
        let p = positions(&mut s, t0, 30..32, 1);
        assert!(close(p[0][0], at(30.0 / 200.0)) && close(p[1][0], at(32.0 / 200.0)));

        // A volta em 4 batidas: 2 s a 120 BPM e 4 s a 60, contando do ponto onde está
        let beats = EfxProgram {
            cycle: Span::Beats(4000),
            ..e
        };
        let (mut s, t0) = started(vec![]);
        s.apply(Command::Bpm(120), t0);
        s.apply(Command::Running(vec![run("e", Program::Efx(beats))]), t0);
        let p = positions(&mut s, t0, 0..26, 1);
        assert!(close(p[25][0], at(0.25)));
        s.apply(Command::Bpm(60), t0 + TICK * 26);
        let p = positions(&mut s, t0, 26..77, 1);
        assert!(close(p[50][0], at(0.26 + 50.0 / 200.0)));
    }

    #[test]
    fn live_edits_keep_the_drawing_going() {
        let e = efx(2);
        let (mut s, t0) = started(vec![("e", Program::Efx(e.clone()))]);
        positions(&mut s, t0, 0..20, 2);
        // A largura muda no quadro seguinte, no mesmo ponto da volta; a segunda fixture sai e apaga
        let mut edited = e.clone();
        edited.efx.width = 20;
        edited.efx.fixtures.pop();
        edited.heads.pop();
        s.apply(
            Command::Running(vec![run("e", Program::Efx(edited.clone()))]),
            t0 + TICK * 20,
        );
        let p = positions(&mut s, t0, 20..21, 2);
        let x = dmx(point(&edited.efx, 0.1 * TAU).0, false);
        assert_eq!((p[0][0].0, p[0][1]), (i32::from(x), (0, 0)));
        // A que entra ganha o fader e anda junto
        s.apply(
            Command::Running(vec![run("e", Program::Efx(e))]),
            t0 + TICK * 21,
        );
        let p = positions(&mut s, t0, 21..22, 2);
        assert!(close(p[0][1], at(21.0 / 200.0)));
    }

    #[test]
    fn efx_lights_in_a_chaser_step_and_in_a_collection() {
        let moving = |p: &Vec<Vec<(i32, i32)>>| p.windows(2).filter(|w| w[0] != w[1]).count() > 5;
        let step = Step {
            program: Program::Efx(efx(1)),
            times: Imposed::default(),
            hold: Span::Ms(60_000),
        };
        let chaser = ChaserProgram {
            steps: vec![step],
            run_order: RunOrder::Loop,
            direction: Direction::Forward,
            keep_last: false,
        };
        let (mut s, t0) = started(vec![("c", Program::Chaser(chaser))]);
        assert!(moving(&positions(&mut s, t0, 0..20, 1)));
        // Na coleção, um EFX "Uma vez" termina, e a coleção só com ele também
        let mut once = efx(1);
        once.efx.run_order = RunOrder::SingleShot;
        once.cycle = Span::Ms(400);
        let (mut s, t0) = started(vec![("k", Program::Collection(vec![Program::Efx(once)]))]);
        assert!(moving(&positions(&mut s, t0, 0..20, 1)));
        positions(&mut s, t0, 20..22, 1);
        assert_eq!(s.done(), ["k"]);
    }
}

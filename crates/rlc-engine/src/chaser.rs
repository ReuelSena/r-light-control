//! Chaser no motor, como o `ChaserRunner` do QLC+: anda pelos passos no tempo ou nas batidas, na ordem
//! escolhida, com crossfade de um passo para o seguinte (o que sai faz o fade out enquanto o que entra
//! faz o fade in). Cada passo acende o que dispara com os faders do motor.

use std::time::{Duration, Instant};

use rlc_core::{Direction, RunOrder, StepAction};

use crate::{ChaserProgram, Ctx, Imposed, Node, Program, Span, TICK};

pub(crate) struct Runner {
    chaser: ChaserProgram,
    /// Tempos impostos pelo passo de um chaser de fora (um chaser dentro do outro).
    imposed: Imposed,
    /// Último passo que começou.
    pub(crate) step: Option<usize>,
    pub(crate) active: Option<Active>,
    /// Sentido de agora: o Ping Pong vira nas pontas.
    forward: bool,
    /// Ordem sorteada da volta, no Random.
    order: Vec<usize>,
    /// Pedido da cue list, feito no quadro seguinte.
    pub(crate) pending: Option<StepAction>,
    pub(crate) done: bool,
}

/// O passo atual e o que ele acendeu.
pub(crate) struct Active {
    pub(crate) node: Node,
    since: Instant,
    /// Posição na batida (milésimos) em que o passo começou: a do fim do passo anterior, quando ele
    /// veio em sequência (os passos não atrasam), ou a de quando foi disparado (o primeiro passo ou um
    /// pedido da cue list).
    start: u64,
    chained: bool,
    /// Delay e fade de entrada do passo, já em tempo: contam na duração dele.
    entry: Duration,
}

impl Runner {
    pub(crate) fn new(chaser: ChaserProgram, imposed: Imposed) -> Self {
        Self {
            forward: chaser.direction == Direction::Forward,
            order: shuffled(chaser.steps.len()),
            chaser,
            imposed,
            step: None,
            active: None,
            pending: None,
            done: false,
        }
    }

    /// Um quadro: faz o pedido da cue list e, quando o passo acaba, troca para o seguinte. O primeiro
    /// passo entra no primeiro quadro.
    pub(crate) fn tick(&mut self, ctx: &mut Ctx) {
        let n = self.chaser.steps.len();
        if self.done || n == 0 {
            return;
        }
        if let Some(active) = &mut self.active {
            ctx.tick(&mut active.node);
        }
        let started = self.step.is_some();
        let (mut since, mut start) = (ctx.now, None);
        let next = match self.pending.take() {
            Some(StepAction::Go(i)) => Some(usize::from(i)),
            // Parado, "próximo" começa no primeiro passo e "anterior", no último
            Some(StepAction::Next) if !started => Some(0),
            Some(StepAction::Previous) if !started => Some(n - 1),
            Some(StepAction::Next) => self.next(false),
            // No começo de um Single Shot, "anterior" não faz nada
            Some(StepAction::Previous) => match self.next(true) {
                Some(i) => Some(i),
                None => return,
            },
            None if !started => self.next(false),
            None => match self.over(ctx) {
                Some(at) => {
                    (since, start) = at;
                    self.next(false)
                }
                None => return,
            },
        };
        match next {
            Some(i) if i < n => self.start_step(i, since, start, ctx),
            Some(_) => {}
            // Fim do Single Shot: o último passo fica aceso ou apaga com o fade out dele
            None if self.chaser.keep_last => {}
            None => {
                if let Some(active) = self.active.take() {
                    ctx.stop(&active.node);
                }
                self.done = true;
            }
        }
    }

    /// O passo que sai faz o fade out enquanto o que entra faz o fade in (o crossfade). Os tempos
    /// impostos de fora valem antes dos do passo; sem nenhum, valem os da criação do passo.
    fn start_step(&mut self, index: usize, since: Instant, start: Option<u64>, ctx: &mut Ctx) {
        if let Some(old) = self.active.take() {
            ctx.stop(&old.node);
        }
        let step = &self.chaser.steps[index];
        let imposed = self.imposed.or(step.times);
        let mut node = ctx.start(step.program.clone(), imposed);
        // Um chaser dentro do passo já acende o primeiro passo dele neste quadro
        ctx.tick(&mut node);
        self.active = Some(Active {
            node,
            since,
            start: start.unwrap_or(ctx.beat_pos),
            chained: start.is_some(),
            entry: entry(imposed, &step.program, ctx.beat),
        });
        self.step = Some(index);
    }

    /// Pausado: o passo não anda, nem o que ele acendeu.
    pub(crate) fn shift(&mut self, gap: Duration, beats: u64) {
        if let Some(active) = &mut self.active {
            active.since += gap;
            active.start += beats;
            active.node.shift(gap, beats);
        }
    }

    /// O passo atual acabou? Devolve de onde conta o seguinte, para os passos não atrasarem: no
    /// tempo, a hora em que este devia acabar (ele troca no quadro mais perto dela); em batidas, a
    /// posição na batida em que ele devia acabar.
    fn over(&self, ctx: &Ctx) -> Option<(Instant, Option<u64>)> {
        let active = self.active.as_ref()?;
        let step = self.chaser.steps.get(self.step?)?;
        match step.hold {
            Span::Ms(hold) => {
                let length = active.entry + Duration::from_millis(hold.into());
                let due = active.since + length.max(TICK);
                if ctx.now + TICK / 2 < due {
                    return None;
                }
                // Uma edição deixou o passo mais curto que o tempo que já passou: conta de agora
                let since = if ctx.now.saturating_duration_since(due) < TICK {
                    due
                } else {
                    ctx.now
                };
                Some((since, None))
            }
            Span::Beats(hold) => {
                // A entrada em tempo (a da cena, no "Padrão") conta pelas batidas de agora
                let fade = active.entry.as_micros() * 1000 / ctx.beat.as_micros().max(1);
                let length = fade as u64 + u64::from(hold);
                // Disparado fora da grade da batida (ou da fração dela, num passo menor que uma
                // batida), o passo acaba no ponto seguinte da grade: o primeiro pode ser mais curto,
                // para cair no tempo
                let grid = length.clamp(1, 1000);
                let start = if active.chained {
                    active.start
                } else {
                    active.start / grid * grid
                };
                let end = start + length;
                (ctx.beat_pos >= end && ctx.now > active.since).then_some((ctx.now, Some(end)))
            }
        }
    }

    /// Próximo passo (ou o anterior), como o `getNextStepIndex` do QLC+; `None` no fim do Single Shot.
    fn next(&mut self, previous: bool) -> Option<usize> {
        let n = self.chaser.steps.len() as isize;
        let run_order = self.chaser.run_order;
        let random = run_order == RunOrder::Random;
        // Posição do último passo na volta (no Random, na ordem sorteada)
        let mut i = match self.step {
            Some(step) if random => {
                self.order.iter().position(|&o| o == step).unwrap_or(step) as isize
            }
            Some(step) => step as isize,
            None if self.chaser.direction == Direction::Backward => n,
            None => -1,
        };
        // "Anterior" numa ponta do Ping Pong volta pelo outro lado
        if run_order == RunOrder::PingPong && previous {
            if i == 0 {
                self.forward = false;
            } else if i == n - 1 {
                self.forward = true;
            }
        }
        i += if self.forward != previous { 1 } else { -1 };
        if (0..n).contains(&i) {
            let i = i as usize;
            return Some(if random { self.order[i] } else { i });
        }
        let i = match run_order {
            RunOrder::SingleShot => return None,
            RunOrder::Loop if i >= n => 0,
            RunOrder::Loop => n - 1,
            // Vira sem repetir a ponta
            RunOrder::PingPong => {
                self.forward = !self.forward;
                (if self.forward { 1 } else { n - 2 }).clamp(0, n - 1)
            }
            // Volta nova: outra ordem sorteada, sem repetir seguido o passo que acabou de rodar
            RunOrder::Random => {
                fastrand::shuffle(&mut self.order);
                let pos = if i >= n { 0 } else { self.order.len() - 1 };
                if self.order.len() > 1 && Some(self.order[pos]) == self.step {
                    self.order.swap(pos, if pos == 0 { 1 } else { pos - 1 });
                }
                return Some(self.order[pos]);
            }
        };
        Some(i as usize)
    }

    /// Edição com o chaser rodando: o passo atual continua, com os valores e os tempos novos; a ordem,
    /// o sentido e os passos novos valem a partir da próxima troca.
    pub(crate) fn update(&mut self, chaser: ChaserProgram, imposed: Imposed, ctx: &mut Ctx) {
        if chaser.direction != self.chaser.direction {
            self.forward = chaser.direction == Direction::Forward;
        }
        if chaser.steps.len() != self.order.len() {
            self.order = shuffled(chaser.steps.len());
        }
        self.chaser = chaser;
        self.imposed = imposed;
        let (Some(i), Some(active)) = (self.step, &mut self.active) else {
            return;
        };
        match self.chaser.steps.get(i) {
            Some(step) => {
                let imposed = self.imposed.or(step.times);
                active.entry = entry(imposed, &step.program, ctx.beat);
                ctx.update(&mut active.node, step.program.clone(), imposed);
            }
            // O passo atual saiu do chaser: vai para o seguinte
            None => self.pending = self.pending.or(Some(StepAction::Next)),
        }
    }
}

/// Delay e fade de entrada do passo, em tempo, que contam na duração dele: os impostos pelo chaser ou,
/// sem eles, os da própria criação do passo (numa coleção, os da mais longa).
fn entry(imposed: Imposed, program: &Program, beat: Duration) -> Duration {
    let own = |delay: Span, fade: Span| {
        imposed.delay_in.unwrap_or(delay).duration(beat)
            + imposed.fade_in.unwrap_or(fade).duration(beat)
    };
    match program {
        Program::Scene(scene) => own(scene.times.delay_in, scene.times.fade_in),
        Program::Efx(efx) => own(Span::Ms(0), efx.fade_in),
        Program::Collection(programs) if !programs.is_empty() => programs
            .iter()
            .map(|p| entry(imposed, p, beat))
            .max()
            .unwrap_or_default(),
        Program::Collection(_) | Program::Chaser(_) => own(Span::Ms(0), Span::Ms(0)),
    }
}

fn shuffled(n: usize) -> Vec<usize> {
    let mut order: Vec<usize> = (0..n).collect();
    fastrand::shuffle(&mut order);
    order
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Step;
    use crate::tests::{patched, run, scene};
    use crate::{Command, Run, State};
    use Direction::*;
    use RunOrder::*;

    /// Chaser cujos passos põem o pan (canal 3, LTP) em 10, 20, 30…, sem fade.
    fn chaser(n: u16, hold_ms: u32, run_order: RunOrder, direction: Direction) -> ChaserProgram {
        let steps = (1..=n)
            .map(|k| Step {
                program: scene(&[(3, 10 * k)], 0),
                times: Imposed::default(),
                hold: Span::Ms(hold_ms),
            })
            .collect();
        ChaserProgram {
            steps,
            run_order,
            direction,
            keep_last: false,
        }
    }

    fn started(chaser: ChaserProgram) -> (State, Instant) {
        let mut s = patched();
        let t0 = Instant::now();
        s.apply(
            Command::Running(vec![run("c", Program::Chaser(chaser))]),
            t0,
        );
        (s, t0)
    }

    /// Chaser de passos em batidas, a 120 BPM (uma batida a cada 25 quadros) desde `t0`.
    fn in_beats(beats: u32) -> (State, Instant) {
        let mut c = chaser(2, 0, Loop, Forward);
        for step in &mut c.steps {
            step.hold = Span::Beats(beats);
        }
        let mut s = patched();
        let t0 = Instant::now();
        s.apply(Command::Bpm(120), t0);
        s.apply(Command::Running(vec![run("c", Program::Chaser(c))]), t0);
        (s, t0)
    }

    /// Passo aceso em cada quadro, pelo pan (1, 2, 3…; 0 = nenhum).
    fn played(s: &mut State, t0: Instant, ticks: std::ops::Range<u32>) -> Vec<u16> {
        ticks
            .map(|k| u16::from(s.render(t0 + TICK * k)[0].1[2]) / 10)
            .collect()
    }

    /// Cada passo por 5 quadros (passos de 100 ms, como nos testes do QLC+).
    fn fives(steps: &[u16]) -> Vec<u16> {
        steps.iter().flat_map(|&k| [k; 5]).collect()
    }

    fn go(s: &mut State, at: Instant, action: StepAction) {
        s.apply(Command::Step("c".into(), action), at);
    }

    #[test]
    fn steps_follow_the_run_order_like_qlc() {
        // Os `write…Five` do `chaserrunner_test`
        let cases = [
            (Loop, Forward, vec![1, 2, 3, 1, 2, 3]),
            (Loop, Backward, vec![3, 2, 1, 3, 2, 1]),
            (PingPong, Forward, vec![1, 2, 3, 2, 1, 2, 3]),
            (PingPong, Backward, vec![3, 2, 1, 2, 3, 2, 1]),
            // O Single Shot para no fim, e o pan volta ao padrão
            (SingleShot, Forward, vec![1, 2, 3, 0, 0]),
            (SingleShot, Backward, vec![3, 2, 1, 0, 0]),
        ];
        for (run_order, direction, expected) in cases {
            let (mut s, t0) = started(chaser(3, 100, run_order, direction));
            let ticks = 0..expected.len() as u32 * 5;
            assert_eq!(
                played(&mut s, t0, ticks),
                fives(&expected),
                "{run_order:?} {direction:?}"
            );
        }
    }

    #[test]
    fn random_plays_every_step_each_round_without_repeating() {
        // Passos de um quadro: 50 voltas de 4
        let (mut s, t0) = started(chaser(4, 0, Random, Forward));
        let steps = played(&mut s, t0, 0..200);
        for round in steps.chunks(4) {
            let mut sorted = round.to_vec();
            sorted.sort();
            assert_eq!(sorted, [1, 2, 3, 4]);
        }
        assert!(steps.windows(2).all(|w| w[0] != w[1]));
    }

    #[test]
    fn steps_last_fade_in_plus_hold_and_crossfade() {
        // Dimmer, vermelho e azul (canais 1, 2 e 5), com fade in e fade out de 2 quadros e hold de 3:
        // cada passo dura 5 quadros
        let red = scene(&[(1, 255), (2, 255), (5, 0)], 0);
        let blue = scene(&[(1, 255), (2, 0), (5, 255)], 0);
        let step = |program| Step {
            program,
            times: Imposed {
                fade_in: Some(Span::Ms(40)),
                fade_out: Some(Span::Ms(40)),
                ..Imposed::default()
            },
            hold: Span::Ms(60),
        };
        let (mut s, t0) = started(ChaserProgram {
            steps: vec![step(red), step(blue)],
            ..chaser(0, 0, Loop, Forward)
        });
        let frames: Vec<[u8; 3]> = (0..7)
            .map(|k| {
                let f = s.render(t0 + TICK * k)[0].1;
                [f[0], f[1], f[4]]
            })
            .collect();
        // O vermelho entra em 2 quadros; no quinto, o azul entra enquanto o vermelho sai, e o dimmer
        // aceso nos dois passos não pisca
        assert_eq!(
            frames,
            [
                [127, 127, 0],
                [255, 255, 0],
                [255, 255, 0],
                [255, 255, 0],
                [255, 255, 0],
                [255, 128, 127],
                [255, 0, 255]
            ]
        );
    }

    #[test]
    fn default_fade_in_is_the_scene_one_and_counts_in_the_step() {
        // Sem fade no passo, vale o da cena (2 quadros), e o passo dura ele mais o hold (3 quadros)
        let mut c = chaser(2, 60, Loop, Forward);
        c.steps[0].program = scene(&[(3, 10)], 40);
        let (mut s, t0) = started(c);
        let steps: Vec<_> = (0..6)
            .map(|k| {
                s.render(t0 + TICK * k);
                s.chasers()[0].1
            })
            .collect();
        assert_eq!(steps, [0, 0, 0, 0, 0, 1]);
    }

    #[test]
    fn cue_list_goes_next_previous_and_to_a_step() {
        // O `nextPrevious` do `chaserrunner_test`, com passos de 1 s
        let (mut s, t0) = started(chaser(3, 1000, Loop, Forward));
        let at = |k: u32| t0 + TICK * k;
        assert_eq!(played(&mut s, t0, 0..1), [1]);
        go(&mut s, at(1), StepAction::Next);
        assert_eq!(played(&mut s, t0, 1..2), [2]);
        go(&mut s, at(2), StepAction::Next);
        assert_eq!(played(&mut s, t0, 2..3), [3]);
        // Loop: depois do último, o primeiro; antes do primeiro, o último
        go(&mut s, at(3), StepAction::Next);
        assert_eq!(played(&mut s, t0, 3..4), [1]);
        go(&mut s, at(4), StepAction::Previous);
        assert_eq!(played(&mut s, t0, 4..5), [3]);
        go(&mut s, at(5), StepAction::Go(1));
        assert_eq!(played(&mut s, t0, 5..6), [2]);
        // Depois de ir para um passo, o chaser continua dali no tempo dele
        assert_eq!(played(&mut s, t0, 55..56), [3]);
        assert_eq!(s.chasers(), [("c".to_string(), 2)]);

        // Parado, "anterior" começa no último passo
        let (mut s, t0) = started(chaser(3, 1000, Loop, Forward));
        go(&mut s, t0, StepAction::Previous);
        assert_eq!(played(&mut s, t0, 0..1), [3]);
    }

    #[test]
    fn single_shot_ends_dark_or_keeps_the_last_step() {
        for keep_last in [false, true] {
            let (mut s, t0) = started(ChaserProgram {
                keep_last,
                ..chaser(2, 100, SingleShot, Forward)
            });
            let last = if keep_last { 2 } else { 0 };
            let expected = [fives(&[1, 2]), vec![last; 5]].concat();
            assert_eq!(played(&mut s, t0, 0..15), expected);
            assert_eq!(s.done().is_empty(), keep_last);
            // "Próximo" no último passo faz o mesmo que o fim
            if keep_last {
                go(&mut s, t0 + TICK * 15, StepAction::Next);
                assert_eq!(played(&mut s, t0, 15..16), [2]);
            }
        }
    }

    #[test]
    fn steps_in_beats_change_on_the_beat() {
        // Passos de 1 batida, disparado no meio de uma: o primeiro entra na hora e acaba na batida
        // seguinte; daí em diante, os passos trocam nas batidas
        let mut c = chaser(2, 0, Loop, Forward);
        for step in &mut c.steps {
            step.hold = Span::Beats(1000);
        }
        let mut s = patched();
        let t0 = Instant::now();
        s.apply(Command::Bpm(120), t0);
        let program = Program::Chaser(c);
        s.apply(Command::Running(vec![run("c", program)]), t0 + TICK * 10);
        let expected = [vec![1; 15], vec![2; 25], vec![1; 25], vec![2; 5]].concat();
        assert_eq!(played(&mut s, t0, 10..80), expected);
    }

    #[test]
    fn steps_of_half_a_beat_keep_time() {
        // Meia batida são 12,5 quadros: os passos alternam 12 e 13, sem deriva (o QLC+ só conta
        // batidas inteiras)
        let (mut s, t0) = in_beats(500);
        let expected = [vec![1; 12], vec![2; 13], vec![1; 12], vec![2; 13]].concat();
        assert_eq!(played(&mut s, t0, 0..50), expected);
    }

    #[test]
    fn tap_just_before_a_beat_is_that_beat() {
        // Passos de 1 batida: o tap no quadro 45 (pouco antes da batida do quadro 50) troca o passo
        // no toque, e as batidas seguintes contam dele
        let (mut s, t0) = in_beats(1000);
        assert_eq!(
            played(&mut s, t0, 0..45),
            [vec![1; 25], vec![2; 20]].concat()
        );
        s.apply(Command::Bpm(120), t0 + TICK * 45);
        assert_eq!(
            played(&mut s, t0, 45..75),
            [vec![1; 25], vec![2; 5]].concat()
        );
    }

    #[test]
    fn editing_a_running_chaser_keeps_the_current_step() {
        let (mut s, t0) = started(chaser(3, 1000, Loop, Forward));
        assert_eq!(played(&mut s, t0, 0..1), [1]);
        let edit = |s: &mut State, k: u32, c: ChaserProgram| {
            let runs = vec![run("c", Program::Chaser(c))];
            s.apply(Command::Running(runs), t0 + TICK * k);
        };
        // O valor novo do passo atual vale na hora, sem recomeçar
        let mut c = chaser(3, 1000, Loop, Forward);
        c.steps[0].program = scene(&[(3, 70)], 0);
        edit(&mut s, 1, c.clone());
        assert_eq!(played(&mut s, t0, 1..2), [7]);
        // Com um hold menor que o tempo que já passou, o passo troca na hora
        c.steps[0].hold = Span::Ms(10);
        edit(&mut s, 2, c);
        assert_eq!(played(&mut s, t0, 2..3), [2]);
    }

    #[test]
    fn chaser_inside_a_step_starts_and_stops_with_it() {
        // O passo 1 é outro chaser (pan 10 e 20, um quadro cada); o passo 2, uma cena com o pan em 90
        let step = |program| Step {
            program,
            times: Imposed::default(),
            hold: Span::Ms(100),
        };
        let (mut s, t0) = started(ChaserProgram {
            steps: vec![
                step(Program::Chaser(chaser(2, 0, Loop, Forward))),
                step(scene(&[(3, 90)], 0)),
            ],
            ..chaser(0, 0, Loop, Forward)
        });
        assert_eq!(played(&mut s, t0, 0..10), [1, 2, 1, 2, 1, 9, 9, 9, 9, 9]);
    }

    #[test]
    fn an_infinite_hold_waits_for_the_go() {
        let (mut s, t0) = started(chaser(3, u32::MAX, Loop, Forward));
        assert_eq!(played(&mut s, t0, 0..500), vec![1; 500]);
        go(&mut s, t0 + TICK * 500, StepAction::Next);
        assert_eq!(played(&mut s, t0, 500..502), [2, 2]);
    }

    #[test]
    fn the_entry_delay_counts_in_the_step() {
        // Delay de entrada de 2 quadros e hold de 3: cada passo dura 5 quadros, e o que sai apaga
        // enquanto o seguinte espera
        let mut c = chaser(2, 60, Loop, Forward);
        for step in &mut c.steps {
            step.times.delay_in = Some(Span::Ms(40));
        }
        let (mut s, t0) = started(c);
        assert_eq!(
            played(&mut s, t0, 0..12),
            [0, 0, 1, 1, 1, 0, 0, 2, 2, 2, 0, 0]
        );
    }

    #[test]
    fn a_paused_chaser_holds_its_step() {
        // Passos de 5 quadros: pausado no quarto, o passo continua o mesmo; ao continuar, faltam 2
        let (mut s, t0) = started(chaser(2, 100, Loop, Forward));
        let pause = |s: &mut State, k: u32, paused: bool| {
            let c = Run {
                paused,
                ..run("c", Program::Chaser(chaser(2, 100, Loop, Forward)))
            };
            s.apply(Command::Running(vec![c]), t0 + TICK * k);
        };
        assert_eq!(played(&mut s, t0, 0..3), [1, 1, 1]);
        pause(&mut s, 3, true);
        assert_eq!(played(&mut s, t0, 3..20), vec![1; 17]);
        pause(&mut s, 20, false);
        assert_eq!(played(&mut s, t0, 20..24), [1, 1, 2, 2]);
    }
}

//! Coleção com linha do tempo (F11b): cada item acende a criação dele no início e a para no fim, com o
//! fade out dela. Os itens com duração formam a passada, que se repete no Laço, roda uma vez no
//! "Uma vez" e, no Vai e volta, volta com os inícios espelhados (cada criação toca para a frente). Os
//! itens sem duração acendem uma vez, no início deles, e ficam até a coleção parar: com todos assim
//! (a coleção de antes), tudo acende junto, como sempre.

use std::time::{Duration, Instant};

use rlc_core::{Priority, RunOrder};

use crate::{Ctx, Imposed, Node, Program};

/// Uma coleção no motor. Os tempos são milésimos de batida, com `beats` (seguindo o BPM master), ou
/// milissegundos.
#[derive(Clone)]
pub struct TimelineProgram {
    pub items: Vec<TimelineItem>,
    /// Laço, Uma vez (`SingleShot`) ou Vai e volta; o Aleatório vale como o Laço.
    pub run_order: RunOrder,
    pub beats: bool,
    /// Comprimento da passada; `None` = o fim do último item com duração.
    pub length: Option<u32>,
}

#[derive(Clone)]
pub struct TimelineItem {
    pub program: Program,
    pub start: u32,
    /// `None` = até a coleção parar.
    pub duration: Option<u32>,
    /// `None` = a da criação disparada.
    pub priority: Option<Priority>,
}

pub(crate) struct TimelineRunner {
    timeline: TimelineProgram,
    /// Tempos impostos pelo passo de um chaser de fora.
    imposed: Imposed,
    /// Começo da linha do tempo na hora e na batida (na grade da batida, para os itens caírem no
    /// tempo): a pausa empurra os dois.
    since: Instant,
    since_beat: u64,
    /// O que cada item acendeu e em que passada.
    lit: Vec<Option<(Node, u64)>>,
    /// Onde está o cursor (o playhead da UI), na unidade da coleção.
    pub(crate) position: u32,
    /// Fim do "Uma vez": os itens com duração não acendem mais.
    ended: bool,
}

impl TimelineRunner {
    /// Os itens que começam no zero acendem já (sem andar), como a coleção de antes.
    pub(crate) fn new(timeline: TimelineProgram, imposed: Imposed, ctx: &mut Ctx) -> Self {
        let mut runner = Self {
            lit: timeline.items.iter().map(|_| None).collect(),
            timeline,
            imposed,
            since: ctx.now,
            since_beat: ctx.beat_pos / 1000 * 1000,
            position: 0,
            ended: false,
        };
        runner.advance(ctx, false);
        runner
    }

    pub(crate) fn tick(&mut self, ctx: &mut Ctx) {
        self.advance(ctx, true);
    }

    /// Acende os itens que entraram na janela, para os que saíram e anda os acesos (com `tick`).
    fn advance(&mut self, ctx: &mut Ctx, tick: bool) {
        let t = if self.timeline.beats {
            ctx.beat_pos.saturating_sub(self.since_beat)
        } else {
            ctx.now.saturating_duration_since(self.since).as_millis() as u64
        };
        let items = &self.timeline.items;
        let length = u64::from(self.timeline.length.unwrap_or_else(|| {
            let ends = items
                .iter()
                .filter_map(|i| Some(i.start.saturating_add(i.duration?)));
            ends.max().unwrap_or(0)
        }));
        let (pass, at) = t
            .checked_div(length)
            .map_or((0, t), |pass| (pass, t % length));
        let order = self.timeline.run_order;
        if order == RunOrder::SingleShot && length > 0 && t >= length {
            self.ended = true;
        }
        let back = order == RunOrder::PingPong && pass % 2 == 1;
        // Sem passada, o cursor para no último início
        let position = match length {
            0 => t.min(items.iter().map(|i| u64::from(i.start)).max().unwrap_or(0)),
            _ if self.ended => length,
            _ if back => length - at,
            _ => at,
        };
        self.position = position.min(u64::from(u32::MAX)) as u32;
        let imposed = self.imposed;
        for (item, lit) in items.iter().zip(&mut self.lit) {
            let start = u64::from(item.start);
            let (on, pass) = match item.duration {
                None => (t >= start, 0),
                Some(_) if self.ended || length == 0 => (false, pass),
                Some(d) => {
                    let end = (start + u64::from(d)).min(length);
                    let window = if back {
                        length.saturating_sub(end)..length.saturating_sub(start)
                    } else {
                        start..end
                    };
                    (window.contains(&at), pass)
                }
            };
            // O item de outra passada acabou: recomeça nesta
            if let Some((node, p)) = lit
                && (!on || *p != pass)
            {
                ctx.stop(node);
                *lit = None;
            }
            match lit {
                Some((node, _)) if tick => with(ctx, item.priority, |ctx| ctx.tick(node)),
                Some(_) => {}
                // Um chaser de dentro já acende o primeiro passo neste quadro (no começo, no primeiro)
                None if on => {
                    let node = with(ctx, item.priority, |ctx| {
                        let mut node = ctx.start(item.program.clone(), imposed);
                        if tick {
                            ctx.tick(&mut node);
                        }
                        node
                    });
                    *lit = Some((node, pass));
                }
                None => {}
            }
        }
    }

    /// Terminou sozinha: no "Uma vez", com a passada acabada; e os itens sem duração terminaram (o
    /// Single Shot de dentro que apagou), como a coleção de antes.
    pub(crate) fn done(&self) -> bool {
        !self.lit.is_empty()
            && self
                .timeline
                .items
                .iter()
                .zip(&self.lit)
                .all(|(item, lit)| match item.duration {
                    None => lit.as_ref().is_some_and(|(node, _)| node.done()),
                    Some(_) => self.ended,
                })
    }

    pub(crate) fn nodes(&self) -> impl Iterator<Item = &Node> {
        self.lit.iter().flatten().map(|(node, _)| node)
    }

    pub(crate) fn shift(&mut self, gap: Duration, beats: u64) {
        self.since += gap;
        self.since_beat += beats;
        for (node, _) in self.lit.iter_mut().flatten() {
            node.shift(gap, beats);
        }
    }

    /// Edição rodando: os acesos recebem os valores novos na hora, e as janelas novas valem no quadro
    /// seguinte. O item que mudou de prioridade recomeça; o que saiu apaga.
    pub(crate) fn update(&mut self, timeline: TimelineProgram, imposed: Imposed, ctx: &mut Ctx) {
        let old = std::mem::replace(&mut self.timeline, timeline);
        self.imposed = imposed;
        let n = self.timeline.items.len();
        for (node, _) in self.lit.drain(n.min(self.lit.len())..).flatten() {
            ctx.stop(&node);
        }
        self.lit.resize_with(n, || None);
        for (i, (item, lit)) in self.timeline.items.iter().zip(&mut self.lit).enumerate() {
            let Some((node, _)) = lit else { continue };
            if old.items.get(i).map(|o| o.priority) != Some(item.priority) {
                ctx.stop(node);
                *lit = None;
            } else {
                with(ctx, item.priority, |ctx| {
                    ctx.update(node, item.program.clone(), imposed);
                });
            }
        }
    }
}

/// Com a prioridade do item, tudo o que ele acende fica nela (também quando a da criação muda).
fn with<R>(ctx: &mut Ctx, priority: Option<Priority>, f: impl FnOnce(&mut Ctx) -> R) -> R {
    let saved = (ctx.priority, ctx.pinned);
    if let Some(p) = priority {
        (ctx.priority, ctx.pinned) = (p, true);
    }
    let result = f(ctx);
    (ctx.priority, ctx.pinned) = saved;
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::{patched, run, scene};
    use crate::{ChaserProgram, Command, Run, Span, State, Step, TICK};
    use rlc_core::Direction;

    fn item(program: Program, start: u32, duration: Option<u32>) -> TimelineItem {
        TimelineItem {
            program,
            start,
            duration,
            priority: None,
        }
    }

    /// Dimmer no 1 de 0 a 100 ms e azul no 5 de 60 a 160 ms: a passada tem 160 ms (8 quadros).
    fn two(run_order: RunOrder) -> TimelineProgram {
        TimelineProgram {
            items: vec![
                item(scene(&[(1, 200)], 0), 0, Some(100)),
                item(scene(&[(5, 100)], 0), 60, Some(100)),
            ],
            run_order,
            beats: false,
            length: None,
        }
    }

    fn started(timeline: TimelineProgram) -> (State, Instant) {
        let mut s = patched();
        let t0 = Instant::now();
        s.apply(
            Command::Running(vec![run("k", Program::Timeline(timeline))]),
            t0,
        );
        (s, t0)
    }

    /// Canais 1 e 5 em cada quadro.
    fn played(s: &mut State, t0: Instant, ticks: std::ops::Range<u32>) -> Vec<[u8; 2]> {
        ticks
            .map(|k| {
                let f = s.render(t0 + TICK * k)[0].1;
                [f[0], f[4]]
            })
            .collect()
    }

    const A: [u8; 2] = [200, 0];
    const B: [u8; 2] = [0, 100];
    const AB: [u8; 2] = [200, 100];
    const OFF: [u8; 2] = [0, 0];

    #[test]
    fn items_light_in_their_window_and_the_loop_repeats() {
        let (mut s, t0) = started(two(RunOrder::Loop));
        let pass = [A, A, A, AB, AB, B, B, B];
        assert_eq!(played(&mut s, t0, 0..16), [pass, pass].concat());
        assert!(s.done().is_empty());
    }

    #[test]
    fn ping_pong_mirrors_the_starts_on_the_way_back() {
        let (mut s, t0) = started(two(RunOrder::PingPong));
        let forward = [A, A, A, AB, AB, B, B, B];
        let back = [B, B, B, AB, AB, A, A, A];
        assert_eq!(played(&mut s, t0, 0..16), [forward, back].concat());
        // O cursor volta do fim para o começo
        s.render(t0 + TICK * 9);
        assert_eq!(s.timelines(), [("k".to_string(), 140)]);
    }

    #[test]
    fn once_ends_after_the_pass() {
        let (mut s, t0) = started(two(RunOrder::SingleShot));
        let pass = [A, A, A, AB, AB, B, B, B];
        assert_eq!(played(&mut s, t0, 0..10), [&pass[..], &[OFF, OFF]].concat());
        assert_eq!(s.done(), ["k"]);
    }

    #[test]
    fn the_length_cuts_the_pass_and_the_position_follows() {
        let mut timeline = two(RunOrder::Loop);
        timeline.length = Some(60);
        let (mut s, t0) = started(timeline);
        // O azul começa no fim da passada: nunca acende; o dimmer recomeça a cada passada
        assert_eq!(played(&mut s, t0, 0..8), vec![A; 8]);
        s.render(t0 + TICK * 5);
        assert_eq!(s.timelines(), [("k".to_string(), 40)]);
    }

    #[test]
    fn items_without_duration_light_once_and_stay() {
        // A coleção de antes: tudo começa no zero, acende junto e fica
        let timeline = TimelineProgram {
            items: vec![
                item(scene(&[(1, 200)], 0), 0, None),
                item(scene(&[(5, 100)], 0), 0, None),
            ],
            run_order: RunOrder::Loop,
            beats: true,
            length: None,
        };
        let (mut s, t0) = started(timeline.clone());
        assert_eq!(played(&mut s, t0, 0..200), vec![AB; 200]);
        assert_eq!(s.timelines(), [("k".to_string(), 0)]);
        // Um item sem duração que começa depois fica aceso por cima do laço dos outros (aqui, o
        // dimmer de 100 ms, que recomeça a cada passada)
        let mut timeline = two(RunOrder::Loop);
        timeline.items[1] = item(scene(&[(5, 100)], 0), 100, None);
        let (mut s, t0) = started(timeline);
        let expected = [vec![A; 5], vec![AB; 11]].concat();
        assert_eq!(played(&mut s, t0, 0..16), expected);
    }

    #[test]
    fn it_ends_when_its_single_shot_chasers_end_like_before() {
        let chaser = ChaserProgram {
            steps: vec![Step {
                program: scene(&[(3, 10)], 0),
                times: Imposed::default(),
                hold: Span::Ms(100),
            }],
            run_order: RunOrder::SingleShot,
            direction: Direction::Forward,
            keep_last: false,
        };
        let timeline = TimelineProgram {
            items: vec![item(Program::Chaser(chaser), 0, None)],
            run_order: RunOrder::Loop,
            beats: true,
            length: None,
        };
        let (mut s, t0) = started(timeline);
        played(&mut s, t0, 0..4);
        assert!(s.done().is_empty());
        played(&mut s, t0, 4..6);
        assert_eq!(s.done(), ["k"]);
    }

    #[test]
    fn items_in_beats_follow_the_master() {
        // A 120 BPM (25 quadros por batida): uma batida cada um, em laço
        let timeline = TimelineProgram {
            items: vec![
                item(scene(&[(1, 200)], 0), 0, Some(1000)),
                item(scene(&[(5, 100)], 0), 1000, Some(1000)),
            ],
            run_order: RunOrder::Loop,
            beats: true,
            length: None,
        };
        let mut s = patched();
        let t0 = Instant::now();
        s.apply(Command::Bpm(120), t0);
        s.apply(
            Command::Running(vec![run("k", Program::Timeline(timeline))]),
            t0,
        );
        let expected = [vec![A; 25], vec![B; 25], vec![A; 25]].concat();
        assert_eq!(played(&mut s, t0, 0..75), expected);
    }

    #[test]
    fn the_item_priority_wins_over_the_collection_one_also_after_edits() {
        // O mesmo dimmer: o item em Alta vale com o valor menor (sem ele, o HTP daria 200)
        let mut timeline = TimelineProgram {
            items: vec![
                item(scene(&[(1, 50)], 0), 0, None),
                item(scene(&[(1, 200)], 0), 0, None),
            ],
            run_order: RunOrder::Loop,
            beats: false,
            length: None,
        };
        timeline.items[0].priority = Some(Priority::High);
        let (mut s, t0) = started(timeline.clone());
        assert_eq!(played(&mut s, t0, 0..1), [[50, 0]]);
        let edit = Run {
            priority: Priority::Low,
            ..run("k", Program::Timeline(timeline.clone()))
        };
        s.apply(Command::Running(vec![edit]), t0 + TICK);
        assert_eq!(played(&mut s, t0, 1..2), [[50, 0]]);
        // Sem a prioridade no item, volta o HTP
        timeline.items[0].priority = None;
        let edit = run("k", Program::Timeline(timeline));
        s.apply(Command::Running(vec![edit]), t0 + TICK * 2);
        assert_eq!(played(&mut s, t0, 2..4), [[200, 0], [200, 0]]);
    }

    #[test]
    fn a_paused_timeline_holds_its_position() {
        let (mut s, t0) = started(two(RunOrder::Loop));
        let pause = |s: &mut State, k: u32, paused: bool| {
            let k1 = Run {
                paused,
                ..run("k", Program::Timeline(two(RunOrder::Loop)))
            };
            s.apply(Command::Running(vec![k1]), t0 + TICK * k);
        };
        assert_eq!(played(&mut s, t0, 0..2), [A, A]);
        pause(&mut s, 2, true);
        assert_eq!(played(&mut s, t0, 2..20), vec![A; 18]);
        assert_eq!(s.timelines(), [("k".to_string(), 20)]);
        pause(&mut s, 20, false);
        assert_eq!(played(&mut s, t0, 20..24), [A, AB, AB, B]);
    }

    #[test]
    fn a_removed_item_goes_dark() {
        let (mut s, t0) = started(two(RunOrder::Loop));
        assert_eq!(played(&mut s, t0, 3..4), [AB]);
        let mut timeline = two(RunOrder::Loop);
        timeline.items.pop();
        s.apply(
            Command::Running(vec![run("k", Program::Timeline(timeline))]),
            t0 + TICK * 4,
        );
        assert_eq!(played(&mut s, t0, 4..5), [A]);
    }
}

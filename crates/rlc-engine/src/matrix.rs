//! RGB Matrix no motor, como o `RGBMatrix` do QLC+: a cada passo o algoritmo desenha um mapa de cores
//! (uma por célula da grade de cabeças), e cada canal que muda faz fade até o valor novo (o fade in, ou
//! o fade out quando o novo é 0). Todas as cabeças ficam num fader só, como uma cena: a criação entra
//! a partir do que está embaixo e, ao parar, volta no fade out até lá.
//!
//! Os mapas dos scripts chegam de uma thread à parte (`rlc-script`), que o motor nunca espera: o do
//! passo seguinte é pedido assim que o atual aparece, e, se ele não chegou quando o passo acaba, o
//! atual continua até chegar.

use std::collections::VecDeque;
use std::sync::Arc;
use std::time::{Duration, Instant};

use rlc_core::{Direction, Id, RunOrder, ScrollStyle};
use rlc_script::{Reply, Session};

use crate::{Ctx, Imposed, SceneLevel, Span, TICK, Times, is_valid, text};

/// Cores do passo: `map[y][x]` em 0xRRGGBB.
pub type Map = Vec<Vec<u32>>;

/// Um RGB Matrix com a grade já nos endereços DMX (o motor não conhece fixtures).
#[derive(Clone)]
pub struct MatrixProgram {
    /// A criação, para o aviso de erro.
    pub function: Id,
    pub source: MatrixSource,
    /// Quantas cores o algoritmo usa (a cor do passo vai da inicial à final só com mais de uma).
    pub accepts: usize,
    /// Cores [r, g, b]: a inicial e a final.
    pub colors: Vec<[u8; 3]>,
    pub run_order: RunOrder,
    pub direction: Direction,
    /// Tamanho da grade do grupo de fixtures.
    pub width: u16,
    pub height: u16,
    pub cells: Vec<MatrixCell>,
    /// Duração de um passo.
    pub step: Span,
    pub fade_in: Span,
    pub fade_out: Span,
}

/// Quem desenha o mapa de cada passo.
#[derive(Clone, PartialEq, Debug)]
pub enum MatrixSource {
    /// A cor inicial na grade toda: um passo só.
    Plain,
    /// Um texto, na fonte de pontos 5 × 7.
    Text {
        text: String,
        style: ScrollStyle,
        x: i16,
        y: i16,
    },
    /// Uma imagem (pixels 0xRRGGBB, linha a linha, o transparente em 0). Sem imagem, a grade fica apagada.
    Image {
        width: u16,
        height: u16,
        pixels: Arc<[u32]>,
        style: ScrollStyle,
        x: i16,
        y: i16,
    },
    /// O texto de um script do QLC+ e o valor de cada propriedade dele.
    Script {
        source: Arc<str>,
        properties: Vec<(String, String)>,
    },
}

/// Uma cabeça da grade e os canais que recebem a cor dela.
#[derive(Clone, PartialEq, Debug)]
pub struct MatrixCell {
    pub x: u16,
    pub y: u16,
    pub output: CellOutput,
}

/// Como a cor de uma cabeça vira canais (o valor dos níveis não conta).
#[derive(Clone, PartialEq, Debug)]
pub enum CellOutput {
    Rgb([SceneLevel; 3]),
    /// Sem canais RGB: ciano, magenta e amarelo.
    Cmy([SceneLevel; 3]),
    /// Dimmer, branco, âmbar ou UV: o cinza da cor.
    Grey(SceneLevel),
}

impl CellOutput {
    fn levels(&self) -> &[SceneLevel] {
        match self {
            CellOutput::Rgb(l) | CellOutput::Cmy(l) => l,
            CellOutput::Grey(l) => std::slice::from_ref(l),
        }
    }

    /// O valor do canal `part` para a cor 0xRRGGBB. Com o canal fino, o valor tem 16 bits.
    fn value(&self, color: u32, part: usize) -> u16 {
        let [r, g, b] = [(color >> 16) as u8, (color >> 8) as u8, color as u8];
        let v = match self {
            CellOutput::Rgb(_) => [r, g, b][part],
            CellOutput::Cmy(_) => 255 - [r, g, b][part],
            // O cinza do QLC+ (`rgbToGrey`)
            CellOutput::Grey(_) => {
                (0.299 * f64::from(r) + 0.587 * f64::from(g) + 0.114 * f64::from(b)).round() as u8
            }
        };
        match self.levels()[part].fine {
            Some(_) => u16::from(v) << 8,
            None => v.into(),
        }
    }
}

fn pack([r, g, b]: [u8; 3]) -> u32 {
    u32::from(r) << 16 | u32::from(g) << 8 | u32::from(b)
}

impl MatrixProgram {
    /// Quantos passos tem o algoritmo, quando ele mesmo sabe dizer (um script só diz depois de aberto).
    pub fn steps(&self) -> Option<usize> {
        match &self.source {
            MatrixSource::Plain => Some(1),
            MatrixSource::Text { text, style, .. } => Some(text::steps(text, *style).max(1)),
            MatrixSource::Image {
                width,
                height,
                style,
                ..
            } => Some(
                match style {
                    ScrollStyle::Static => 1,
                    ScrollStyle::Horizontal => usize::from(*width),
                    ScrollStyle::Vertical => usize::from(*height),
                }
                .max(1),
            ),
            MatrixSource::Script { .. } => None,
        }
    }

    /// A cor do passo `step` entre `steps`: da inicial à final em inteiros, como o `RGBMatrixStep` do
    /// QLC+.
    pub fn color(&self, step: usize, steps: usize) -> [u8; 3] {
        let first = self.colors.first().copied().unwrap_or([0; 3]);
        let (Some(last), true) = (self.colors.get(1), self.accepts > 1 && steps > 1) else {
            return first;
        };
        let (i, n) = (step as i32, steps as i32 - 1);
        std::array::from_fn(|k| {
            (i32::from(first[k]) + (i32::from(last[k]) - i32::from(first[k])) * i / n) as u8
        })
    }

    /// O mapa do passo, para os algoritmos nativos (o do script vem da thread dele).
    pub fn map(&self, step: usize) -> Option<Map> {
        let size = (self.width, self.height);
        match &self.source {
            MatrixSource::Plain => {
                let color = pack(self.color(step, 1));
                Some(vec![vec![color; self.width.into()]; self.height.into()])
            }
            MatrixSource::Text { text, style, x, y } => {
                let rgb = pack(self.color(step, self.steps()?));
                Some(text::map(
                    text,
                    *style,
                    (i32::from(*x), i32::from(*y)),
                    size,
                    rgb,
                    step,
                ))
            }
            MatrixSource::Image {
                width,
                height,
                pixels,
                style,
                x,
                y,
            } => {
                // A imagem dá a volta, como no QLC+; a rolagem anda uma célula por passo
                let (w, h) = (i32::from(*width), i32::from(*height));
                let (mut dx, mut dy) = (i32::from(*x), i32::from(*y));
                match style {
                    ScrollStyle::Static => {}
                    ScrollStyle::Horizontal => dx += step as i32,
                    ScrollStyle::Vertical => dy += step as i32,
                }
                let pixel = |cx: i32, cy: i32| {
                    let (px, py) = (
                        (cx + dx).rem_euclid(w.max(1)),
                        (cy + dy).rem_euclid(h.max(1)),
                    );
                    pixels.get((py * w + px) as usize).copied().unwrap_or(0)
                };
                Some(
                    (0..i32::from(size.1))
                        .map(|cy| (0..i32::from(size.0)).map(|cx| pixel(cx, cy)).collect())
                        .collect(),
                )
            }
            MatrixSource::Script { .. } => None,
        }
    }

    /// O canal de cada nível do fader: (célula, parte da saída), só dos endereços válidos.
    fn slots(&self) -> Vec<(usize, usize)> {
        let cells = self.cells.iter().enumerate();
        let levels = cells.flat_map(|(i, c)| {
            c.output
                .levels()
                .iter()
                .enumerate()
                .map(move |(p, l)| (i, p, l))
        });
        levels
            .filter(|(_, _, l)| is_valid(l))
            .map(|(i, p, _)| (i, p))
            .collect()
    }

    /// O valor de cada canal de `slots` para o mapa. Célula fora do mapa fica apagada.
    fn values(&self, map: &Map, slots: &[(usize, usize)]) -> Vec<u16> {
        slots
            .iter()
            .map(|&(i, part)| {
                let cell = &self.cells[i];
                let row = map.get(usize::from(cell.y));
                let color = row.and_then(|r| r.get(usize::from(cell.x))).copied();
                cell.output.value(color.unwrap_or(0), part)
            })
            .collect()
    }
}

/// Para que um mapa foi pedido.
#[derive(Clone, Copy, PartialEq, Debug)]
enum Purpose {
    /// O primeiro passo: aparece e começa a contar o tempo.
    Show,
    /// O passo atual de novo, com as cores editadas: aparece sem mexer na contagem.
    Refresh,
    /// O passo seguinte, adiantado.
    Next,
}

pub(crate) struct MatrixRunner {
    matrix: MatrixProgram,
    /// Tempos impostos pelo passo de um chaser (o RGB Matrix não tem delay nem curva próprios).
    imposed: Imposed,
    /// O fader de todos os canais das cabeças.
    fader: u64,
    /// De que célula e parte da saída é cada canal do fader, na ordem dele.
    slots: Vec<(usize, usize)>,
    /// A thread do script, quando o algoritmo é um.
    session: Option<Session>,
    /// Pedidos ao script que ainda não chegaram, em ordem.
    asked: VecDeque<(usize, Purpose)>,
    /// O mapa do passo seguinte, já pronto.
    ready: Option<(usize, Map)>,
    /// Quantos passos o algoritmo tem (1 até o script abrir).
    steps: usize,
    step: usize,
    /// O primeiro mapa já apareceu; até lá, nada anda.
    shown: bool,
    /// Sentido de agora: o Vai e volta vira nas pontas.
    forward: bool,
    /// Quando o passo atual começou, como no chaser: na hora e na posição da batida (milésimos). O
    /// passo que vem logo depois do anterior conta do fim dele, para não atrasar.
    since: Instant,
    start: u64,
    chained: bool,
    /// O script falhou: a criação parou.
    pub(crate) failure: Option<String>,
    pub(crate) done: bool,
}

impl MatrixRunner {
    pub(crate) fn new(matrix: MatrixProgram, imposed: Imposed, ctx: &mut Ctx) -> Self {
        let slots = matrix.slots();
        // Os canais entram apagados: o primeiro mapa os leva à cor, no fade in
        let levels = slots
            .iter()
            .map(|&(i, part)| matrix.cells[i].output.levels()[part].clone());
        let times = imposed.on(Times::fades(matrix.fade_in, matrix.fade_out));
        let session = match &matrix.source {
            MatrixSource::Plain | MatrixSource::Text { .. } | MatrixSource::Image { .. } => None,
            MatrixSource::Script { source, properties } => Some(Session::open(
                source.clone(),
                properties.clone(),
                matrix.width,
                matrix.height,
            )),
        };
        let mut runner = Self {
            fader: ctx.push(levels, times, false),
            slots,
            steps: matrix.steps().unwrap_or(1),
            session,
            asked: VecDeque::new(),
            ready: None,
            step: 0,
            shown: false,
            forward: matrix.direction == Direction::Forward,
            since: ctx.now,
            start: ctx.beat_pos,
            chained: false,
            failure: None,
            done: false,
            matrix,
            imposed,
        };
        // Os nativos já sabem os passos: o primeiro mapa sai agora
        if runner.session.is_none() {
            runner.begin(ctx);
        }
        runner
    }

    pub(crate) fn function(&self) -> &Id {
        &self.matrix.function
    }

    /// Pede o primeiro passo: o primeiro da ordem, ou o último no sentido contrário.
    fn begin(&mut self, ctx: &mut Ctx) {
        let step = if self.forward { 0 } else { self.steps - 1 };
        self.request(step, Purpose::Show, ctx);
    }

    /// Um quadro: recolhe o que o script respondeu e, quando o passo acaba, mostra o seguinte; no fim do
    /// Single Shot, para com o fade out.
    pub(crate) fn tick(&mut self, ctx: &mut Ctx) {
        self.collect(ctx);
        if self.done || !self.shown {
            return;
        }
        let Some((since, start)) = self.over(ctx) else {
            return;
        };
        let mut forward = self.forward;
        let next = next_step(self.matrix.run_order, &mut forward, self.step, self.steps);
        match next {
            Some(step) => match self.ready.take_if(|r| r.0 == step) {
                Some((_, map)) => {
                    self.forward = forward;
                    self.show(step, &map, ctx);
                    (self.since, self.start, self.chained) =
                        (since, start.unwrap_or(ctx.beat_pos), start.is_some());
                    self.prefetch(ctx);
                }
                // O mapa ainda não chegou: o atual continua
                None if self.asked.is_empty() => self.request(step, Purpose::Next, ctx),
                None => {}
            },
            None => {
                self.stop(ctx);
                self.done = true;
            }
        }
    }

    /// Pede o mapa do passo `step`. Os nativos respondem na hora.
    fn request(&mut self, step: usize, purpose: Purpose, ctx: &mut Ctx) {
        match &self.session {
            Some(session) => {
                let rgb = pack(self.matrix.color(step, self.steps));
                let colors = self.matrix.colors.iter().copied().map(pack).collect();
                session.request(step, rgb, colors);
                self.asked.push_back((step, purpose));
            }
            None => {
                let map = self.matrix.map(step).unwrap_or_default();
                self.arrive(step, purpose, map, ctx);
            }
        }
    }

    /// Adianta o mapa do passo que vem depois do atual.
    fn prefetch(&mut self, ctx: &mut Ctx) {
        let mut forward = self.forward;
        let order = self.matrix.run_order;
        if let Some(next) = next_step(order, &mut forward, self.step, self.steps) {
            self.request(next, Purpose::Next, ctx);
        }
    }

    /// Lê as respostas da thread do script.
    fn collect(&mut self, ctx: &mut Ctx) {
        let replies: Vec<Reply> = std::iter::from_fn(|| self.session.as_ref()?.poll()).collect();
        for reply in replies {
            match reply {
                Reply::Opened { steps } => {
                    self.steps = steps.max(1);
                    self.begin(ctx);
                }
                Reply::Map { step, map } => {
                    if let Some((asked, purpose)) = self.asked.pop_front() {
                        debug_assert_eq!(asked, step);
                        self.arrive(step, purpose, map, ctx);
                    }
                }
                Reply::Failed(message) => {
                    self.failure = Some(message);
                    self.session = None;
                    self.stop(ctx);
                    self.done = true;
                    return;
                }
            }
        }
    }

    /// Chegou o mapa que foi pedido.
    fn arrive(&mut self, step: usize, purpose: Purpose, map: Map, ctx: &mut Ctx) {
        match purpose {
            Purpose::Show => {
                self.show(step, &map, ctx);
                (self.since, self.start, self.chained) = (ctx.now, ctx.beat_pos, false);
                self.shown = true;
                self.prefetch(ctx);
            }
            Purpose::Refresh => {
                self.show(step, &map, ctx);
                self.ready = None;
                self.prefetch(ctx);
            }
            Purpose::Next => self.ready = Some((step, map)),
        }
    }

    pub(crate) fn stop(&self, ctx: &mut Ctx) {
        ctx.stop_fader(self.fader);
    }

    /// Pausado: o passo não anda.
    pub(crate) fn shift(&mut self, gap: Duration, beats: u64) {
        self.since += gap;
        self.start += beats;
    }

    /// O passo acabou? Devolve de onde conta o seguinte: no tempo, a hora em que este devia acabar (ele
    /// troca no quadro mais perto dela); em batidas, a posição em que devia acabar. No máximo um passo
    /// por quadro.
    fn over(&self, ctx: &Ctx) -> Option<(Instant, Option<u64>)> {
        match self.matrix.step {
            Span::Ms(ms) => {
                let due = self.since + Duration::from_millis(ms.into()).max(TICK);
                if ctx.now + TICK / 2 < due {
                    return None;
                }
                // Uma edição deixou o passo mais curto que o tempo que já passou: conta de agora
                let late = ctx.now.saturating_duration_since(due) >= TICK;
                Some((if late { ctx.now } else { due }, None))
            }
            Span::Beats(beats) => {
                // Disparado fora da grade da batida (ou da fração dela, num passo menor que uma
                // batida), o passo acaba no ponto seguinte da grade
                let length = u64::from(beats);
                let grid = length.clamp(1, 1000);
                let start = if self.chained {
                    self.start
                } else {
                    self.start / grid * grid
                };
                let end = start + length;
                (ctx.beat_pos >= end && ctx.now > self.since).then_some((ctx.now, Some(end)))
            }
        }
    }

    /// Mostra o mapa do passo: cada canal que muda vai do valor de agora ao novo, no fade in (ou no fade
    /// out, quando o novo é 0). Um fade maior que o passo nunca chega ao alvo, como no QLC+.
    fn show(&mut self, step: usize, map: &Map, ctx: &mut Ctx) {
        self.step = step;
        let values = self.matrix.values(map, &self.slots);
        let Some(fader) = ctx.faders.iter_mut().find(|f| f.serial == self.fader) else {
            return;
        };
        let (curve, times) = (fader.times.curve, fader.times);
        for (f, value) in fader.channels.iter_mut().zip(values) {
            if f.level.value == value {
                continue;
            }
            let i = usize::from(f.level.channel) - 1;
            let fades = f.universe.is_none_or(|u| ctx.universes[u].fades[i]);
            f.start = f.value(f.level.value, ctx.now, curve, fades, false);
            f.level.value = value;
            f.since = ctx.now;
            f.time = if value == 0 {
                times.fade_out
            } else {
                times.fade_in
            }
            .duration(ctx.beat);
        }
    }

    /// Edição com o RGB Matrix rodando: as cores, os tempos, a ordem e o sentido valem na hora, sem
    /// recomeçar. Outro algoritmo (ou texto de script, ou propriedade), outra grade ou outros
    /// endereços recomeçam do primeiro passo, com o que estava aceso saindo no fade out.
    pub(crate) fn update(&mut self, matrix: MatrixProgram, imposed: Imposed, ctx: &mut Ctx) {
        let old = &self.matrix;
        if (&matrix.source, &matrix.cells, matrix.width, matrix.height)
            != (&old.source, &old.cells, old.width, old.height)
        {
            self.stop(ctx);
            *self = Self::new(matrix, imposed, ctx);
            return;
        }
        let colors_changed = matrix.colors != old.colors || matrix.accepts != old.accepts;
        if matrix.direction != old.direction {
            self.forward = matrix.direction == Direction::Forward;
        }
        self.matrix = matrix;
        self.imposed = imposed;
        let times = self
            .imposed
            .on(Times::fades(self.matrix.fade_in, self.matrix.fade_out));
        if let Some(fader) = ctx.faders.iter_mut().find(|f| f.serial == self.fader) {
            fader.times = times;
        }
        // As cores novas valem na hora: o mapa do passo atual é pedido de novo
        if colors_changed && self.shown {
            self.ready = None;
            self.request(self.step.min(self.steps - 1), Purpose::Refresh, ctx);
        }
    }
}

/// O passo depois de `step` entre `n`, como o `checkNextStep` do QLC+; `None` no fim do Single Shot. O
/// Vai e volta vira sem repetir a ponta. O QLC+ não tem Aleatória no RGB Matrix: vale como Loop.
fn next_step(order: RunOrder, forward: &mut bool, step: usize, n: usize) -> Option<usize> {
    let (n, i) = (n as isize, step as isize + if *forward { 1 } else { -1 });
    if (0..n).contains(&i) {
        return Some(i as usize);
    }
    match order {
        RunOrder::SingleShot => None,
        RunOrder::PingPong => {
            *forward = !*forward;
            Some((if *forward { 1 } else { n - 2 }).clamp(0, n - 1) as usize)
        }
        RunOrder::Loop | RunOrder::Random => Some(if i >= n { 0 } else { n as usize - 1 }),
    }
}

#[cfg(test)]
mod tests {
    use std::time::Instant;

    use super::*;
    use crate::tests::{patched, run};
    use crate::{Command, Program, State};

    /// Canais de teste (todos de intensidade): 8, 9 e 10 são o RGB da cabeça. O motor calcula cada quadro
    /// com o tempo do quadro seguinte, então o primeiro já anda um tick.
    fn level(channel: u16, fine: Option<u16>) -> SceneLevel {
        SceneLevel {
            universe: "u1".into(),
            channel,
            fine,
            value: 0,
        }
    }

    fn rgb(first: u16) -> CellOutput {
        CellOutput::Rgb([0, 1, 2].map(|k| level(first + k, None)))
    }

    /// Uma cabeça RGB nos canais 8 a 10, com a cor dada e sem fades.
    fn matrix(color: [u8; 3]) -> MatrixProgram {
        MatrixProgram {
            function: "m".into(),
            source: MatrixSource::Plain,
            accepts: 1,
            colors: vec![color],
            run_order: RunOrder::Loop,
            direction: Direction::Forward,
            width: 1,
            height: 1,
            cells: vec![MatrixCell {
                x: 0,
                y: 0,
                output: rgb(8),
            }],
            step: Span::Ms(500),
            fade_in: Span::Ms(0),
            fade_out: Span::Ms(0),
        }
    }

    fn started(m: MatrixProgram) -> (State, Instant) {
        let mut s = patched();
        let t0 = Instant::now();
        s.apply(Command::Running(vec![run("m", Program::RgbMatrix(m))]), t0);
        (s, t0)
    }

    /// O que sai nos canais 8 a 13, no quadro `tick`.
    fn out(s: &mut State, t0: Instant, tick: u32) -> [u8; 6] {
        let f = s.render(t0 + TICK * tick)[0].1;
        std::array::from_fn(|k| f[7 + k])
    }

    #[test]
    fn the_color_goes_to_the_channels_of_each_head() {
        let mut m = matrix([255, 128, 0]);
        // Duas cabeças: a segunda, nos canais 11 a 13, na coluna 1
        m.width = 2;
        m.cells.push(MatrixCell {
            x: 1,
            y: 0,
            output: rgb(11),
        });
        let (mut s, t0) = started(m);
        assert_eq!(out(&mut s, t0, 1), [255, 128, 0, 255, 128, 0]);
    }

    #[test]
    fn cmy_and_grey_outputs() {
        let mut m = matrix([200, 100, 50]);
        m.cells = vec![
            MatrixCell {
                x: 0,
                y: 0,
                output: CellOutput::Cmy([0, 1, 2].map(|k| level(8 + k, None))),
            },
            MatrixCell {
                x: 0,
                y: 0,
                output: CellOutput::Grey(level(11, None)),
            },
        ];
        let (mut s, t0) = started(m);
        // 0,299 × 200 + 0,587 × 100 + 0,114 × 50 = 124,2
        assert_eq!(out(&mut s, t0, 1)[..4], [55, 155, 205, 124]);
    }

    #[test]
    fn a_fine_channel_takes_the_16_bit_value() {
        let mut m = matrix([255, 0, 0]);
        m.cells[0].output = CellOutput::Rgb([level(8, Some(11)), level(9, None), level(10, None)]);
        let (mut s, t0) = started(m);
        assert_eq!(out(&mut s, t0, 1)[..4], [255, 0, 0, 0]);
    }

    #[test]
    fn enters_and_leaves_with_the_fades() {
        let mut m = matrix([200, 0, 0]);
        m.fade_in = Span::Ms(1000);
        m.fade_out = Span::Ms(1000);
        let (mut s, t0) = started(m);
        assert_eq!(out(&mut s, t0, 0)[0], 4);
        assert_eq!(out(&mut s, t0, 24)[0], 100);
        assert_eq!(out(&mut s, t0, 49)[0], 200);
        s.apply(Command::Running(vec![]), t0 + TICK * 60);
        assert_eq!(out(&mut s, t0, 84)[0], 100);
        assert_eq!(out(&mut s, t0, 109)[0], 0);
    }

    #[test]
    fn editing_the_color_applies_at_once_without_restarting() {
        let (mut s, t0) = started(matrix([255, 0, 0]));
        out(&mut s, t0, 0);
        let blue = run("m", Program::RgbMatrix(matrix([0, 0, 255])));
        s.apply(Command::Running(vec![blue]), t0 + TICK * 5);
        assert_eq!(out(&mut s, t0, 6)[..3], [0, 0, 255]);
    }

    #[test]
    fn the_single_shot_stops_after_one_step_and_the_pause_holds_it() {
        let mut m = matrix([255, 0, 0]);
        m.run_order = RunOrder::SingleShot;
        m.step = Span::Ms(200);
        let (mut s, t0) = started(m.clone());
        // Pausada por 1 s, não anda
        let mut paused = run("m", Program::RgbMatrix(m.clone()));
        paused.paused = true;
        s.apply(Command::Running(vec![paused]), t0 + TICK * 2);
        for tick in 2..52 {
            s.render(t0 + TICK * tick);
        }
        assert!(s.done().is_empty());
        s.apply(
            Command::Running(vec![run("m", Program::RgbMatrix(m))]),
            t0 + TICK * 52,
        );
        for tick in 52..64 {
            s.render(t0 + TICK * tick);
        }
        assert_eq!(s.done(), vec!["m".to_string()]);
        assert_eq!(out(&mut s, t0, 64)[0], 0);
    }

    /// Os passos que o sequenciador escolhe, `len` vezes, num algoritmo de `n` passos.
    fn walk(order: RunOrder, direction: Direction, n: usize, len: usize) -> Vec<Option<usize>> {
        let mut forward = direction == Direction::Forward;
        let mut step = if forward { 0 } else { n - 1 };
        (0..len)
            .map(|_| {
                let next = next_step(order, &mut forward, step, n);
                step = next.unwrap_or(step);
                next
            })
            .collect()
    }

    #[test]
    fn the_run_orders_walk_the_steps_like_the_qlc() {
        use Direction::*;
        use RunOrder::*;
        let steps = |v: Vec<Option<usize>>| v.into_iter().flatten().collect::<Vec<_>>();
        assert_eq!(steps(walk(Loop, Forward, 3, 5)), [1, 2, 0, 1, 2]);
        assert_eq!(steps(walk(Loop, Backward, 3, 5)), [1, 0, 2, 1, 0]);
        // O Vai e volta não repete as pontas
        assert_eq!(
            steps(walk(PingPong, Forward, 4, 8)),
            [1, 2, 3, 2, 1, 0, 1, 2]
        );
        assert_eq!(walk(SingleShot, Forward, 2, 3), [Some(1), None, None]);
    }

    /// Um script de 4 passos: o passo `n` pinta a grade de vermelho `(n + 1) × 50`; a propriedade "gain"
    /// multiplica. Os mapas vêm de outra thread, então os testes esperam o relógio de verdade.
    const SCRIPT: &str = r#"
        (function () {
          var algo = new Object;
          algo.apiVersion = 2;
          algo.name = "Rampa";
          algo.author = "teste";
          var gain = 1;
          algo.properties = ["name:gain|type:range|display:Ganho|values:1,5|write:setGain|read:getGain"];
          algo.setGain = function (v) { gain = parseInt(v); };
          algo.getGain = function () { return gain; };
          algo.rgbMap = function (width, height, rgb, step) {
            var map = new Array(height);
            for (var y = 0; y < height; y++) {
              map[y] = new Array(width);
              for (var x = 0; x < width; x++) map[y][x] = ((step + 1) * 50 * gain) << 16;
            }
            return map;
          };
          algo.rgbMapStepCount = function (width, height) { return 4; };
          return algo;
        })()
    "#;

    fn scripted(properties: &[(&str, &str)]) -> MatrixProgram {
        // O script é uma expressão que devolve o objeto do algoritmo, como os do QLC+
        let source = SCRIPT;
        let properties = properties
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();
        MatrixProgram {
            source: MatrixSource::Script {
                source: source.into(),
                properties,
            },
            step: Span::Ms(100),
            ..matrix([0, 0, 0])
        }
    }

    /// Anda o relógio do teste até `want` sair no canal 8 (ou o tempo de verdade acabar). Devolve os
    /// vermelhos vistos, sem repetir seguidos.
    fn red_until(s: &mut State, t0: Instant, from: u32, want: usize, limit: u32) -> Vec<u8> {
        let mut seen: Vec<u8> = vec![];
        for tick in from..limit {
            let red = out(s, t0, tick)[0];
            if seen.last() != Some(&red) {
                seen.push(red);
            }
            if seen.len() >= want {
                break;
            }
            // O script roda na hora de verdade
            std::thread::sleep(Duration::from_millis(3));
        }
        seen
    }

    #[test]
    fn a_script_draws_each_step_in_order_without_blocking_the_engine() {
        let mut m = scripted(&[]);
        m.run_order = RunOrder::PingPong;
        let (mut s, t0) = started(m);
        // Apagada até o primeiro mapa chegar; depois os passos 0, 1, 2, 3, 2, 1 (sem repetir seguidos)
        let seen = red_until(&mut s, t0, 0, 7, 400);
        assert!(seen.len() >= 7, "{seen:?} {:?}", s.failed());
        assert_eq!(seen[..7], [0, 50, 100, 150, 200, 150, 100]);
    }

    #[test]
    fn a_script_property_applies_and_a_change_restarts_the_script() {
        let (mut s, t0) = started(scripted(&[("gain", "3")]));
        assert_eq!(red_until(&mut s, t0, 0, 2, 200)[..2], [0, 150]);
        // Outro valor da propriedade: recomeça do primeiro passo, com o ganho novo
        let again = run("m", Program::RgbMatrix(scripted(&[("gain", "2")])));
        s.apply(Command::Running(vec![again]), t0 + TICK * 200);
        let seen = red_until(&mut s, t0, 200, 3, 600);
        assert!(seen.contains(&100), "{seen:?}");
    }

    #[test]
    fn a_broken_script_stops_only_its_creation() {
        let mut m = scripted(&[]);
        m.source = MatrixSource::Script {
            source: "(function () { while (true) {} })()".into(),
            properties: vec![],
        };
        let (mut s, t0) = started(m);
        for tick in 0..300 {
            s.render(t0 + TICK * tick);
            if !s.failed().is_empty() {
                break;
            }
            std::thread::sleep(Duration::from_millis(5));
        }
        let failed = s.failed();
        assert_eq!(
            failed.len(),
            1,
            "o script em laço infinito deveria ter falhado"
        );
        assert_eq!(failed[0].0, "m");
        assert_eq!(s.done(), vec!["m".to_string()]);
    }
}

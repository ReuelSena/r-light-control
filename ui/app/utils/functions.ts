import type { Component } from 'vue'
import { Camera, Film, Layers, LayoutGrid, Orbit, Repeat } from 'lucide-vue-next'
import type { Curve } from '~/types/generated/Curve'
import type { Efx } from '~/types/generated/Efx'
import type { EfxFixture } from '~/types/generated/EfxFixture'
import type { EfxShape } from '~/types/generated/EfxShape'
import type { Fixture } from '~/types/generated/Fixture'
import type { Function } from '~/types/generated/Function'
import type { Project } from '~/types/generated/Project'
import type { RgbMatrix } from '~/types/generated/RgbMatrix'
import type { RgbScript } from '~/types/generated/RgbScript'
import type { Scene } from '~/types/generated/Scene'
import type { SceneValue } from '~/types/generated/SceneValue'

export type { FunctionType } from '~/types/generated/FunctionType'

/** Ícone de cada tipo de criação do Studio, usado em todo o app para identificá-lo (E12). */
export const functionIcons: Record<FunctionType, Component> = {
  scene: Camera,
  chaser: Repeat,
  sequence: Film,
  collection: Layers,
  efx: Orbit,
  rgbMatrix: LayoutGrid,
}

export const functionTypes = Object.keys(functionIcons) as FunctionType[]

/** Hold infinito (∞) dos passos de chaser: o passo espera o Go. O mesmo valor do contrato (`INFINITE`). */
export const INFINITE = 4294967295

/** Curvas dos fades (R2) e os tipos de atributo com tempos próprios na cena (R3), na ordem das telas. */
export const curves = ['linear', 'easeIn', 'easeOut', 'easeInOut'] as const
export const attributeKinds = ['intensity', 'color', 'position', 'effects'] as const

const TAU = Math.PI * 2

/** Quanto do caminho já foi andado (0 a 1) na fração `x` do tempo, em cada curva (`ease` do motor). */
export function ease(curve: Curve, x: number) {
  switch (curve) {
    case 'linear': return x
    case 'easeIn': return x * x
    case 'easeOut': return 1 - (1 - x) * (1 - x)
    case 'easeInOut': return x * x * (3 - 2 * x)
  }
}

/** Quanto do desenho já foi andado (0 a 1) na fração `u` da volta: a curva desenhada (os pontos ligados por
 * retas) ou a pronta, como o `speed` do motor (crates/rlc-engine/src/efx.rs). */
export function efxSpeed(shape: EfxShape, u: number) {
  const points = shape.speedPoints
  if (!points || points.length < 2) return ease(shape.speed, u)
  const k = u * (points.length - 1)
  const i = Math.min(Math.floor(k), points.length - 2)
  return (points[i]! + (points[i + 1]! - points[i]!) * (k - i)) / 255
}

/** Ponto do desenho à mão (-1 a 1) na fração `u` do caminho fechado, pela distância percorrida. */
function along(path: [number, number][], u: number): [number, number] {
  const points = path.map(([x, y]) => [x / 127, y / 127] as [number, number])
  if (!points.length) return [0, 0]
  const segments = points.map((a, i) => [a, points[(i + 1) % points.length]!] as const)
  const length = ([a, b]: (typeof segments)[number]) => Math.hypot(b[0] - a[0], b[1] - a[1])
  let left = u * segments.reduce((sum, s) => sum + length(s), 0)
  for (const s of segments) {
    const l = length(s)
    if (left <= l && l > 0) return [s[0][0] + (s[1][0] - s[0][0]) * (left / l), s[0][1] + (s[1][1] - s[0][1]) * (left / l)]
    left -= l
  }
  return points[0]!
}

/** Ponto do desenho do EFX (-1 a 1) no ângulo `t` (0 a 2π). Algoritmos portados do QLC+ (NOTICE). */
function efxShape(efx: EfxShape, t: number): [number, number] {
  const { cos, sin, round, PI } = Math
  // Nos quadrados, cada quarto da volta é um lado (f anda de -1 a 1 dentro do quarto)
  const side = (corners: [number, number][]) => corners[Math.min(3, Math.floor(t / (PI / 2)))]!
  const f = ((t % (PI / 2)) / (PI / 2)) * 2 - 1
  switch (efx.algorithm) {
    case 'circle': return [cos(t + PI / 2), cos(t)]
    case 'eight': return [cos(t * 2 + PI / 2), cos(t)]
    case 'line': return [cos(t), cos(t)]
    case 'line2': return [t / PI - 1, t / PI - 1]
    case 'diamond': return [cos(t - PI / 2) ** 3, cos(t) ** 3]
    case 'square': return side([[f, 1], [1, -f], [-f, -1], [-1, f]])
    case 'squareChoppy': return [round(cos(t)), round(sin(t))]
    case 'squareTrue': return side([[1, 1], [1, -1], [-1, -1], [-1, 1]])
    case 'leaf': return [cos(t + PI / 2) ** 5, cos(t)]
    case 'lissajous': return [wave(efx.xFrequency, efx.xPhase, t), wave(efx.yFrequency, efx.yPhase, t)]
    case 'freehand': return along(efx.path, t / TAU)
  }
}

/** Um eixo do Lissajous; com a frequência 0, um vai e vem em linha reta, como no QLC+. */
function wave(frequency: number, phase: number, t: number) {
  const p = (phase * Math.PI) / 180
  if (frequency > 0) return Math.cos(frequency * t - p)
  const k = ((t + p) / Math.PI) % 2
  return (k < 1 ? k : 2 - k) * 2 - 1
}

/** Ponto do EFX já girado, escalado e deslocado, na escala do pan/tilt (0 a 255). */
export function efxPoint(efx: EfxShape, t: number) {
  const [x, y] = efxShape(efx, ((t % TAU) + TAU) % TAU)
  const r = (efx.rotation * Math.PI) / 180
  return {
    x: efx.xOffset + x * Math.cos(r) * efx.width + y * Math.sin(r) * efx.height,
    y: efx.yOffset - x * Math.sin(r) * efx.width + y * Math.cos(r) * efx.height,
  }
}

type EfxItem = Pick<EfxFixture, 'reverse' | 'startOffset' | 'invertPan' | 'invertTilt' | 'swapAxes'> & { fixture: string; shape: EfxShape }

/** Fixtures do EFX, como no servidor (`efx_expand`): só as com pan ou tilt, os grupos com as fixtures que estão
 * neles agora (na ordem da grade, com os ajustes do item, cada uma `spread` graus depois da anterior), cada
 * fixture no primeiro lugar em que aparece. A que tem item próprio usa os ajustes dele, também dentro de um
 * grupo (a exceção). O desenho é o do item, ou o geral. */
export function efxFixtures(project: Project, efx: Efx) {
  const moves = (f: Fixture) => modeChannels(project, f).some((c) => c.attribute === 'pan' || c.attribute === 'tilt')
  const list: EfxItem[] = []
  for (const x of efx.fixtures) {
    const fixtures = 'group' in x ? groupFixtures(project, x.group).filter(moves) : project.fixtures.filter((f) => f.id === x.fixture && moves(f))
    fixtures.forEach((f, k) => {
      const own = efx.fixtures.find((o) => 'fixture' in o && o.fixture === f.id) ?? { ...x, startOffset: (x.startOffset + k * x.spread) % 360 }
      const { reverse, startOffset, invertPan, invertTilt, swapAxes } = own
      if (!list.some((o) => o.fixture === f.id)) list.push({ fixture: f.id, reverse, startOffset, invertPan, invertTilt, swapAxes, shape: own.shape ?? efx })
    })
  }
  return list
}

/** Posição de cada fixture do EFX (`efxFixtures`) no instante `ms`, com as regras do motor
 * (crates/rlc-engine/src/efx.rs): a propagação, o vai e volta, o sentido, o início, a velocidade e o espelho.
 * Na prévia, o desenho se repete sem parar. */
export function efxPositions(efx: Efx, fixtures: EfxItem[], ms: number) {
  const { PI } = Math
  const turns = ms / Math.max(efx.timing.holdMs, 100)
  const n = fixtures.length
  return fixtures.map((f, i) => {
    // "Defasadas", cada uma 1/n da volta à frente; "Em sequência", 1/n atrás
    const at = turns + (efx.propagation === 'asymmetric' ? i / n : efx.propagation === 'serial' ? -i / n : 0)
    const lap = Math.floor(at)
    let a = (at - lap) * TAU
    if (efx.runOrder === 'pingPong' && lap % 2) a = TAU - a
    // Ao contrário, a linha anda meia volta à frente (ida e volta passam pelos mesmos pontos)
    if ((efx.direction === 'backward') !== f.reverse) a = f.shape.algorithm === 'line' ? (a > PI ? a - PI : a + PI) : TAU - a
    a = (a + (f.startOffset * PI) / 180) % TAU
    return efxMirror(f, efxPoint(f.shape, efxSpeed(f.shape, a / TAU) * TAU))
  })
}

/** O ponto do desenho com a troca dos eixos e o espelho do item, em volta do centro do desenho. */
export function efxMirror(f: EfxItem, p: { x: number; y: number }) {
  let [dx, dy] = [p.x - f.shape.xOffset, p.y - f.shape.yOffset]
  if (f.swapAxes) [dx, dy] = [dy, dx]
  return { x: f.shape.xOffset + (f.invertPan ? -dx : dx), y: f.shape.yOffset + (f.invertTilt ? -dy : dy) }
}

/** Valores de uma cena por fixture, como no servidor (`scene_values` em crates/rlc-server/src/apply.rs): os
 * dos grupos dela nas fixtures que estão em cada um agora (o de depois vence) e, por cima, os próprios. */
export function sceneValues(project: Project, scene: Pick<Scene, 'values' | 'groups'>): SceneValue[] {
  const values = new Map<string, SceneValue>()
  for (const g of scene.groups) {
    for (const f of groupFixtures(project, g.group)) {
      const channels = modeChannels(project, f)
      channels.forEach((c, channel) => {
        const v = g.values.find((v) => v.attribute === c.attribute && v.index === attributeIndex(channels, channel))
        if (v) values.set(`${f.id}:${channel}`, { fixture: f.id, channel, value: v.value })
      })
    }
  }
  for (const v of scene.values) values.set(`${v.fixture}:${v.channel}`, v)
  return [...values.values()]
}

/** O algoritmo de um RGB Matrix como o valor de um `<select>`: "plain", "text", "image" ou "script:<id>". */
export const algorithmKey = (a: RgbMatrix['algorithm']) => (a.type === 'script' ? `script:${a.script}` : a.type)

export function algorithmOf(key: string): RgbMatrix['algorithm'] {
  if (key.startsWith('script:')) return { type: 'script', script: key.slice(7), properties: [] }
  if (key === 'text') return { type: 'text', text: 'R LIGHT', style: 'horizontal', x: 0, y: 0 }
  if (key === 'image') return { type: 'image', image: null, style: 'horizontal', x: 0, y: 0 }
  return { type: 'plain' }
}

/** O script de um RGB Matrix, se o algoritmo dele é um. */
export function matrixScript(m: RgbMatrix, scripts: RgbScript[]) {
  const a = m.algorithm
  return a.type === 'script' ? scripts.find((s) => s.id === a.script) : undefined
}

/** Quantas cores o algoritmo usa: a cor fixa, uma; o texto, duas; a imagem, nenhuma (os pixels vêm
 * dela); e um script, as que ele aceita. */
export function matrixColors(m: RgbMatrix, scripts: RgbScript[]) {
  switch (m.algorithm.type) {
    case 'plain': return 1
    case 'text': return 2
    case 'image': return 0
    case 'script': return matrixScript(m, scripts)?.accepts ?? 1
  }
}

/** O passo que o RGB Matrix mostra na `k`-ésima troca, entre `steps` passos: o Loop volta ao começo, o
 * Vai e volta não repete as pontas e o Uma vez fica no último (as contas do motor, `next_step`). */
export function matrixStep(m: RgbMatrix, steps: number, k: number) {
  const pingPong = m.runOrder === 'pingPong'
  const period = pingPong ? Math.max(2 * steps - 2, 1) : steps
  const j = m.runOrder === 'singleShot' ? Math.min(k, steps - 1) : k % period
  const i = pingPong && j >= steps ? period - j : j
  return m.direction === 'forward' ? i : steps - 1 - i
}

/** A criação dispara `target`, direto ou por outra (passos de chaser e coleções). Como no QLC+, uma
 * criação não pode conter a si mesma: as listas de passos e de coleção escondem essas. */
export function contains(functions: Function[], fn: Function, target: string) {
  const members = (f: Function) => (f.type === 'chaser' ? f.steps.map((s) => s.function) : f.type === 'collection' ? f.functions : [])
  // Cópia: a lista da coleção é o estado do app
  const stack = [...members(fn)]
  const seen = new Set<string>()
  while (stack.length) {
    const id = stack.pop()!
    if (id === target) return true
    if (seen.has(id)) continue
    seen.add(id)
    const f = functions.find((x) => x.id === id)
    if (f) stack.push(...members(f))
  }
  return false
}

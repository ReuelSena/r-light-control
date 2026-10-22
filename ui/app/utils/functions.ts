import type { Component } from 'vue'
import { Camera, Film, Layers, LayoutGrid, Orbit, Repeat } from 'lucide-vue-next'
import type { Efx } from '~/types/generated/Efx'
import type { Fixture } from '~/types/generated/Fixture'
import type { Function } from '~/types/generated/Function'
import type { Project } from '~/types/generated/Project'
import type { RgbMatrix } from '~/types/generated/RgbMatrix'
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

const TAU = Math.PI * 2

/** Ponto do desenho do EFX (-1 a 1) no ângulo `t` (0 a 2π). Algoritmos portados do QLC+ (NOTICE). */
function efxShape(efx: Efx, t: number): [number, number] {
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
export function efxPoint(efx: Efx, t: number) {
  const [x, y] = efxShape(efx, ((t % TAU) + TAU) % TAU)
  const r = (efx.rotation * Math.PI) / 180
  return {
    x: efx.xOffset + x * Math.cos(r) * efx.width + y * Math.sin(r) * efx.height,
    y: efx.yOffset - x * Math.sin(r) * efx.width + y * Math.cos(r) * efx.height,
  }
}

type EfxItem = { fixture: string; reverse: boolean; startOffset: number }

/** Fixtures do EFX, como no servidor (`efx_expand`): só as com pan ou tilt, os grupos com as fixtures que estão
 * neles agora (na ordem da grade, com os ajustes do item, cada uma `spread` graus depois da anterior), cada
 * fixture no primeiro lugar em que aparece. A que tem item próprio usa os ajustes dele, também dentro de um
 * grupo (a exceção). */
export function efxFixtures(project: Project, efx: Efx) {
  const moves = (f: Fixture) => modeChannels(project, f).some((c) => c.attribute === 'pan' || c.attribute === 'tilt')
  const list: EfxItem[] = []
  for (const x of efx.fixtures) {
    const fixtures = 'group' in x ? groupFixtures(project, x.group).filter(moves) : project.fixtures.filter((f) => f.id === x.fixture && moves(f))
    fixtures.forEach((f, k) => {
      const own = efx.fixtures.find((o) => 'fixture' in o && o.fixture === f.id) ?? { ...x, startOffset: (x.startOffset + k * x.spread) % 360 }
      if (!list.some((o) => o.fixture === f.id)) list.push({ fixture: f.id, reverse: own.reverse, startOffset: own.startOffset })
    })
  }
  return list
}

/** Posição de cada fixture do EFX (`efxFixtures`) no instante `ms`, com as regras do motor
 * (crates/rlc-engine/src/efx.rs): a propagação, o vai e volta, o sentido e o início. Na prévia, o desenho se
 * repete sem parar. */
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
    if ((efx.direction === 'backward') !== f.reverse) a = efx.algorithm === 'line' ? (a > PI ? a - PI : a + PI) : TAU - a
    return efxPoint(efx, a + (f.startOffset * PI) / 180)
  })
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

export const matrixAlgorithms = ['plain', 'chase', 'stripes', 'gradient', 'random']

const rgb = (hex = '#000000') => [1, 3, 5].map((i) => parseInt(hex.slice(i, i + 2), 16))

/** Cor [r, g, b] da célula (x, y) do RGB Matrix no passo `step`, numa grade de largura `w`. */
export function matrixColor(m: RgbMatrix, x: number, y: number, w: number, step: number): number[] {
  const [a, b] = [rgb(m.colors[0]), rgb(m.colors[1] ?? '#000000')]
  const s = m.direction === 'backward' ? -step : step
  const pos = (((x + s) % w) + w) % w
  switch (m.algorithm) {
    case 'chase': return pos === 0 ? a : [0, 0, 0]
    case 'stripes': return pos % 2 ? b : a
    case 'gradient': return a.map((c, i) => Math.round(c + (b[i]! - c) * (w > 1 ? pos / (w - 1) : 0)))
    case 'random': return Math.sin((x * 12.9898 + y * 78.233 + step) * 43758.5453) % 1 > 0.4 ? a : [0, 0, 0]
    default: return a
  }
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

import type { Component } from 'vue'
import { Camera, Film, Layers, LayoutGrid, Orbit, Repeat } from 'lucide-vue-next'
import type { Efx } from '~/types/generated/Efx'
import type { Function } from '~/types/generated/Function'
import type { RgbMatrix } from '~/types/generated/RgbMatrix'

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
    case 'lissajous': return [cos(Math.max(1, efx.xFrequency) * t - (efx.xPhase * PI) / 180), cos(Math.max(1, efx.yFrequency) * t - (efx.yPhase * PI) / 180)]
  }
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

/** Posição de cada fixture do EFX no instante `ms`: propagação, deslocamento inicial e sentido. */
export function efxPositions(efx: Efx, ms: number) {
  const base = (ms / Math.max(efx.timing.holdMs, 100)) * TAU
  return efx.fixtures.map((f, i) => {
    const spread = efx.propagation === 'parallel' ? 0 : (i * TAU) / efx.fixtures.length
    const t = base + spread + (f.startOffset * Math.PI) / 180
    return efxPoint(efx, (efx.direction === 'backward') !== f.reverse ? -t : t)
  })
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

import type { Component } from 'vue'
import { Clock, ListOrdered, Move, RectangleHorizontal, SlidersVertical, Sparkles, SquareDashed, Timer, Type, Workflow } from 'lucide-vue-next'
import type { InputControl } from '~/types/generated/InputControl'
import type { Rect } from '~/types/generated/Rect'
import type { VcLayouts } from '~/types/generated/VcLayouts'
import type { VcWidget } from '~/types/generated/VcWidget'

// Grade e tipos de elemento do Virtual Console (E11, VC-1, VC-2).

export type WidgetType = VcWidget['type']
export type ConsoleFormat = keyof VcLayouts

export const layoutFormats: ConsoleFormat[] = ['phone', 'tablet', 'desktop']

/** Colunas da grade de cada formato. */
export const formatColumns: Record<ConsoleFormat, number> = { phone: 64, tablet: 128, desktop: 192 }

/**
 * Lado da célula (px) de cada formato, no zoom de 100% do console. É fixo: a janela não o muda, então a
 * área do console tem sempre o mesmo tamanho (colunas × célula) e rola se não couber na tela.
 */
export const formatCell: Record<ConsoleFormat, number> = { phone: 7, tablet: 4, desktop: 6 }

/** Linhas do cabeçalho de uma moldura, onde os elementos de dentro não entram. */
export const frameHeader = 4

/** Ícone, tamanho inicial (em células) e entradas externas de cada tipo; `keys`: aceita tecla do teclado. */
export const widgetKinds: Record<WidgetType, { icon: Component; size: [number, number]; inputs: InputControl[]; keys: boolean }> = {
  button: { icon: RectangleHorizontal, size: [16, 16], inputs: ['main'], keys: true },
  slider: { icon: SlidersVertical, size: [16, 48], inputs: ['main'], keys: false },
  cueList: { icon: ListOrdered, size: [64, 48], inputs: ['main', 'previous', 'pause', 'stop'], keys: true },
  xyPad: { icon: Move, size: [32, 48], inputs: ['x', 'y'], keys: false },
  speedDial: { icon: Timer, size: [48, 32], inputs: ['tap'], keys: true },
  frame: { icon: SquareDashed, size: [64, 48], inputs: [], keys: false },
  clock: { icon: Clock, size: [32, 16], inputs: [], keys: false },
  animation: { icon: Sparkles, size: [48, 48], inputs: ['main'], keys: false },
  label: { icon: Type, size: [32, 8], inputs: [], keys: false },
  macro: { icon: Workflow, size: [16, 16], inputs: ['main'], keys: true },
}
export const widgetTypes = Object.keys(widgetKinds) as WidgetType[]

/**
 * Cores dos elementos: cada cor viva com a versão escura dela (fundo para texto na cor viva)
 * e os cinzas do branco ao preto.
 */
export const widgetColors = {
  pairs: [
    ['#ff0000', '#400000'],
    ['#ff8000', '#402000'],
    ['#ffbf00', '#403000'],
    ['#ffff00', '#404000'],
    ['#00ff00', '#004000'],
    ['#00ffff', '#004040'],
    ['#00bfff', '#003040'],
    ['#0080ff', '#002040'],
    ['#8000ff', '#200040'],
    ['#ff00aa', '#40002b'],
  ],
  grays: ['#ffffff', '#cccccc', '#999999', '#666666', '#404040', '#262626', '#000000'],
}

export const overlaps = (a: Rect, b: Rect) => a.x < b.x + b.w && b.x < a.x + a.w && a.y < b.y + b.h && b.y < a.y + a.h

/** Primeiro lugar livre para um w×h, lendo da esquerda para a direita e de cima para baixo. */
export function freeSpot(taken: Rect[], w: number, h: number, cols: number, top = 0): Rect {
  w = Math.min(w, cols)
  for (let y = top; ; y++) {
    for (let x = 0; x + w <= cols; x++) {
      const rect = { x, y, w, h }
      if (!taken.some((t) => overlaps(t, rect))) return rect
    }
  }
}

/**
 * Leva o layout de uma página de um formato para outro: os mesmos tamanhos, arrumados em ordem
 * de leitura na grade do destino. Dentro das molduras, a primeira linha é do cabeçalho.
 */
export function copyLayout(widgets: VcWidget[], from: ConsoleFormat, to: ConsoleFormat) {
  const byPosition = (a: VcWidget, b: VcWidget) => a.layouts[from]!.y - b.layouts[from]!.y || a.layouts[from]!.x - b.layouts[from]!.x
  const placed = widgets.filter((w) => w.layouts[from]).sort(byPosition)
  const cols = formatColumns[to]
  const top: Rect[] = []
  for (const w of placed.filter((x) => !x.frame)) {
    const { w: width, h } = w.layouts[from]!
    const inner: Rect[] = []
    for (const child of placed.filter((x) => x.frame === w.id)) {
      const rect = child.layouts[from]!
      child.layouts[to] = freeSpot(inner, rect.w, rect.h, Math.min(width, cols), frameHeader)
      inner.push(child.layouts[to]!)
    }
    // A moldura cresce se os filhos não couberem
    w.layouts[to] = freeSpot(top, width, Math.max(h, ...inner.map((r) => r.y + r.h)), cols)
    top.push(w.layouts[to]!)
  }
}

/**
 * Combinação de teclas no formato guardado nas entradas (ex.: "Shift+A", "F1", "Space"). As teclas do
 * teclado numérico têm nome próprio ("Num 1", "Num -"), separadas das da fileira de cima das letras.
 */
export function keyCombo(e: KeyboardEvent) {
  const numpad = { NumpadAdd: '+', NumpadSubtract: '-', NumpadMultiply: '*', NumpadDivide: '/', NumpadDecimal: ',', NumpadEnter: 'Enter' }[e.code]
  const key = e.code.startsWith('Numpad') ? `Num ${numpad ?? e.code.slice(6)}` : e.key === ' ' ? 'Space' : e.key.length === 1 ? e.key.toUpperCase() : e.key
  return [e.ctrlKey && 'Ctrl', e.altKey && 'Alt', e.shiftKey && 'Shift', key].filter(Boolean).join('+')
}

/**
 * Atalhos padrão do app, que nenhum elemento pode usar: −, + e Enter do teclado numérico (seção master),
 * Ctrl + − / Ctrl + + (zoom), Alt + E (editar o console) e Alt + I (dicas de atalho). As mesmas teclas da
 * fileira de cima continuam livres.
 */
export function isReservedKey(e: KeyboardEvent) {
  return ['NumpadAdd', 'NumpadSubtract', 'NumpadEnter'].includes(e.code) || (e.ctrlKey && ['-', '+', '='].includes(e.key))
    || ['Alt+E', 'Alt+I'].includes(keyCombo(e))
}

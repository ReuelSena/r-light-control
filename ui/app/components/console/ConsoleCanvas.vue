<script setup lang="ts">
import { Piano } from 'lucide-vue-next'
import type { Rect } from '~/types/generated/Rect'
import type { VcWidget } from '~/types/generated/VcWidget'

// Uma página do console num formato (VC-2): elementos numa grade de células quadradas, que
// acompanham a largura. Em edição, como no Patch:
// - o toque escolhe; Ctrl + clique ou toque longo soma outros do mesmo nível;
// - arrastar move os escolhidos (no toque, só depois de escolhidos) e o canto redimensiona;
// - um elemento solto dentro de uma moldura passa a ser dela (e sai ao ser solto fora);
// - o destino aparece antes de soltar: verde se livre, vermelho se sobrepõe ou sai da grade;
// - as setas do teclado deslocam os escolhidos (`nudge`, chamado pela tela com as mesmas regras do arrasto).
const props = defineProps<{ widgets: VcWidget[]; format: ConsoleFormat; editing: boolean; zoom: number }>()
const selection = defineModel<string[]>('selection', { required: true })
const { send } = useEngine()
const { t } = useI18n()
const prefs = usePrefs()

// Teclas do elemento para a etiqueta do canto (setas e espaço viram símbolos); nada com as dicas de atalho desligadas
const keyNames: Record<string, string> = { ArrowLeft: '←', ArrowRight: '→', ArrowUp: '↑', ArrowDown: '↓', Space: '␣' }
const keysOf = (w: VcWidget) =>
  !prefs.value.keyHints ? '' : w.inputs.flatMap((i) => (i.key ? [i.key.split('+').map((k) => keyNames[k] ?? k).join('+')] : [])).join(' ')

const cols = computed(() => formatColumns[props.format])
const area = ref<HTMLElement>()
const width = ref(0)
let observer: ResizeObserver | undefined
// Mede quem rola (e não a própria grade, que pode ficar mais larga que ele)
watch(area, (el) => el && (observer ??= new ResizeObserver(([entry]) => (width.value = entry!.contentRect.width))).observe(el.parentElement!))
onBeforeUnmount(() => observer?.disconnect())
// Na edição a célula guarda o tamanho da operação: o painel de propriedades não aperta os elementos
// (a grade rola para o lado), e o que se vê editando é o que se vê operando
const fit = computed(() => Math.max(6, Math.floor(width.value / cols.value)))
const operating = ref(0)
watchEffect(() => !props.editing && (operating.value = fit.value))
// O zoom do console (ao lado do Editar) aumenta ou diminui a célula; a grade rola se passar da tela
const cell = computed(() => Math.max(6, Math.floor((props.editing ? Math.max(fit.value, operating.value) : fit.value) * props.zoom)))

const byId = computed(() => new Map(props.widgets.map((w) => [w.id, w])))
const rectOf = (w: VcWidget) => w.layouts[props.format]
// Posição na página: dentro de uma moldura, soma o canto dela
function absolute(w: VcWidget): Rect | undefined {
  const r = rectOf(w) ?? undefined
  const frame = w.frame ? byId.value.get(w.frame) : undefined
  if (!r || !w.frame) return r
  const f = frame && rectOf(frame)
  return f ? { ...r, x: f.x + r.x, y: f.y + r.y } : undefined
}
// Molduras primeiro, para ficarem por baixo dos filhos
const placed = computed(() => props.widgets.filter((w) => absolute(w)).sort((a, b) => Number(b.type === 'frame') - Number(a.type === 'frame')))
const rows = computed(() => Math.max(0, ...placed.value.map((w) => absolute(w)!.y + absolute(w)!.h)) + (props.editing ? 16 : 0))
const px = (r: Rect) => ({ left: `${r.x * cell.value}px`, top: `${r.y * cell.value}px`, width: `${r.w * cell.value}px`, height: `${r.h * cell.value}px` })

// ---------- Edição ----------
type Drag = { ids: string[]; resize: boolean; x0: number; y0: number; dx: number; dy: number; moved: boolean }
const drag = ref<Drag>()
let holdTimer: ReturnType<typeof setTimeout> | undefined
let held = false
let pending: string | undefined // toque num elemento ainda não escolhido: escolhe ao soltar

const inside = (r: Rect, f: Rect) => r.x >= f.x && r.x + r.w <= f.x + f.w && r.y >= f.y + frameHeader && r.y + r.h <= f.y + f.h

// Ctrl + clique e toque longo somam ou tiram; só elementos do mesmo nível (mesma moldura) andam juntos
function toggle(w: VcWidget) {
  const same = selection.value.filter((id) => byId.value.get(id)?.frame === w.frame)
  selection.value = same.includes(w.id) ? same.filter((id) => id !== w.id) : [...same, w.id]
}

function down(e: PointerEvent, w: VcWidget, resize = false) {
  if (!props.editing || e.button > 0) return
  if (e.ctrlKey || e.metaKey) return toggle(w)
  const touch = e.pointerType !== 'mouse'
  held = false
  clearTimeout(holdTimer)
  if (touch) {
    holdTimer = setTimeout(() => {
      held = true
      drag.value = undefined
      toggle(w)
    }, 500)
  }
  if (!selection.value.includes(w.id)) {
    if (touch) return (pending = w.id)
    selection.value = [w.id]
  }
  ;(e.currentTarget as HTMLElement).setPointerCapture(e.pointerId)
  drag.value = { ids: resize ? [w.id] : [...selection.value], resize, x0: e.clientX, y0: e.clientY, dx: 0, dy: 0, moved: false }
}

function move(e: PointerEvent) {
  const d = drag.value
  if (!d) return
  d.dx = e.clientX - d.x0
  d.dy = e.clientY - d.y0
  if (!d.moved && Math.abs(d.dx) + Math.abs(d.dy) > 4) {
    d.moved = true
    clearTimeout(holdTimer)
  }
}

function up(w: VcWidget) {
  clearTimeout(holdTimer)
  if (held) return
  if (pending === w.id) selection.value = [w.id]
  pending = undefined
  const d = drag.value
  if (d && !d.moved && !d.resize) selection.value = [w.id] // clique sem arrasto num grupo escolhe só ele
  if (d?.moved && plan.value?.ok) send({ op: 'saveWidgets', widgets: plan.value.widgets })
  drag.value = undefined
}

function cancel() {
  clearTimeout(holdTimer)
  pending = undefined
  drag.value = undefined
}

// Destino de mover (ou redimensionar) os elementos `ids` em `cx` × `cy` células, e se ele é válido
function planFor(ids: string[], cx: number, cy: number, resize: boolean) {
  // Filhos de uma moldura que também está andando vão junto com ela
  const moving = ids.map((id) => byId.value.get(id)!).filter((w) => w && rectOf(w) && !ids.includes(w.frame ?? ''))
  const widgets: VcWidget[] = []
  const ghosts: Rect[] = []
  let ok = true
  for (const w of moving) {
    const r = rectOf(w)!
    let frame = w.frame
    // Mínimo de 4 × 4 células, para caber um botão de toque; o texto pode ter 2 de altura
    const minH = w.type === 'label' ? 2 : 4
    let next = resize ? { ...r, w: Math.max(4, r.w + cx), h: Math.max(minH, r.h + cy) } : { ...r, x: r.x + cx, y: r.y + cy }
    // Sozinho, um elemento entra na moldura onde é solto, ou sai dela
    if (!resize && moving.length === 1 && w.type !== 'frame') {
      const abs = { ...absolute(w)!, x: absolute(w)!.x + cx, y: absolute(w)!.y + cy }
      const host = placed.value.find((f) => f.type === 'frame' && inside(abs, rectOf(f)!))
      frame = host?.id ?? null
      next = host ? { ...abs, x: abs.x - rectOf(host)!.x, y: abs.y - rectOf(host)!.y } : abs
    }
    const box = frame ? rectOf(byId.value.get(frame)!)! : undefined
    const children = placed.value.filter((c) => c.frame === w.id).map((c) => rectOf(c)!)
    const fits = box ? inside({ ...next, x: box.x + next.x, y: box.y + next.y }, box) : next.x >= 0 && next.y >= 0 && next.x + next.w <= cols.value
    const free = !placed.value.some((s) => s.frame === frame && s.id !== w.id && !ids.includes(s.id) && overlaps(rectOf(s)!, next))
    // Uma moldura não pode encolher por cima dos filhos
    const holds = children.every((c) => c.x + c.w <= next.w && c.y + c.h <= next.h)
    ok &&= fits && free && holds
    ghosts.push(box ? { ...next, x: box.x + next.x, y: box.y + next.y } : next)
    widgets.push({ ...w, frame, layouts: { ...w.layouts, [props.format]: next } })
  }
  return { ok, widgets, ghosts }
}

const plan = computed(() => {
  const d = drag.value
  return d?.moved ? planFor(d.ids, Math.round(d.dx / cell.value), Math.round(d.dy / cell.value), d.resize) : undefined
})

// Teclado: desloca os escolhidos em `cx` × `cy` células; fica onde está se o destino não for válido
function nudge(cx: number, cy: number) {
  const { ok, widgets } = planFor(selection.value, cx, cy, false)
  if (ok && widgets.length) send({ op: 'saveWidgets', widgets })
}
defineExpose({ nudge })

// Durante o arrasto, os elementos (e os filhos das molduras) seguem o dedo; o destino é o contorno
function itemStyle(w: VcWidget) {
  const d = drag.value
  const follows = d?.moved && !d.resize && (d.ids.includes(w.id) || d.ids.includes(w.frame ?? ''))
  return { ...px(absolute(w)!), transform: follows ? `translate(${d.dx}px, ${d.dy}px)` : undefined, zIndex: follows ? 10 : w.type === 'frame' ? 0 : 1 }
}
</script>

<template>
  <div
    ref="area"
    class="canvas"
    :class="{ editing }"
    :style="{ height: `${rows * cell}px`, minWidth: `${cols * cell}px`, '--cell': `${cell}px` }"
    @pointerdown.self="editing && (selection = [])"
  >
    <div
      v-for="w in placed"
      :key="w.id"
      class="item"
      :class="{ selected: editing && selection.includes(w.id) }"
      :style="itemStyle(w)"
      :data-widget="w.id"
      @pointerdown="down($event, w)"
      @pointermove="move"
      @pointerup="up(w)"
      @pointercancel="cancel"
      @contextmenu="editing && $event.preventDefault()"
    >
      <div v-if="w.type === 'frame'" class="frame">
        <header class="caption">
          <span class="truncate">{{ w.caption }}</span>
          <span v-if="w.solo" class="solo">SOLO</span>
        </header>
      </div>
      <ConsoleWidget v-else :widget="w" :class="{ 'pointer-events-none': editing }" />
      <!-- No canto: a tecla que aciona o elemento (sempre) e, na edição, se há controle MIDI -->
      <span v-if="keysOf(w) || (editing && w.inputs.some((i) => i.midi))" class="badges">
        <Piano v-if="editing && w.inputs.some((i) => i.midi)" :size="12" :aria-label="t('vc.inputs')" />
        <span v-if="keysOf(w)" class="rl-key">{{ keysOf(w) }}</span>
      </span>
      <span v-if="editing && selection.length === 1 && selection[0] === w.id" class="handle" @pointerdown.stop="down($event, w, true)" />
    </div>
    <div v-for="(g, i) in plan?.ghosts ?? []" :key="i" class="ghost" :class="plan?.ok ? 'ok' : 'bad'" :style="px(g)" />
  </div>
</template>

<style scoped>
.canvas {
  position: relative;
  min-height: 100%;
}

/* Em edição, a grade aparece: um filete fraco a cada célula e um mais forte a cada duas */
.editing {
  background-image:
    linear-gradient(to right, rgb(255 255 255 / 0.06) 1px, transparent 1px),
    linear-gradient(to bottom, rgb(255 255 255 / 0.06) 1px, transparent 1px),
    linear-gradient(to right, rgb(255 255 255 / 0.03) 1px, transparent 1px),
    linear-gradient(to bottom, rgb(255 255 255 / 0.03) 1px, transparent 1px);
  background-size: calc(var(--cell) * 2) calc(var(--cell) * 2), calc(var(--cell) * 2) calc(var(--cell) * 2), var(--cell) var(--cell), var(--cell) var(--cell);
}

.item {
  position: absolute;
  padding: 3px;
}

.editing .item {
  cursor: grab;
}

/* Escolhido: o arrasto no toque vale só para ele (o resto continua rolando a página) */
.item.selected {
  touch-action: none;
  outline: 2px solid var(--color-select);
  outline-offset: -1px;
  border-radius: 8px;
}

.frame {
  height: 100%;
  border: 1px solid #000;
  border-radius: 8px;
  background: rgb(255 255 255 / 0.03);
  box-shadow: inset 0 2px 6px rgb(0 0 0 / 0.6);
}

.caption {
  display: flex;
  align-items: center;
  gap: 8px;
  height: calc(var(--cell) * 2 - 6px);
  padding: 0 10px;
  color: var(--color-fg-muted);
  font-size: 11px;
  font-weight: 700;
  letter-spacing: 0.08em;
  text-transform: uppercase;
}

.solo {
  padding: 1px 6px;
  border-radius: 4px;
  background: var(--color-select);
  color: #111;
  font-size: 9px;
}

.badges {
  position: absolute;
  top: 6px;
  right: 6px;
  display: flex;
  max-width: calc(100% - 12px);
  align-items: center;
  gap: 4px;
  color: var(--color-lcd);
  pointer-events: none;
}



.handle {
  position: absolute;
  right: 0;
  bottom: 0;
  width: 22px;
  height: 22px;
  border-radius: 6px 0 8px;
  background: var(--color-select);
  cursor: nwse-resize;
  touch-action: none;
}

.ghost {
  position: absolute;
  z-index: 20;
  border: 2px dashed;
  border-radius: 8px;
  pointer-events: none;
}

.ok {
  border-color: var(--color-on);
  background: rgb(0 255 0 / 0.08);
}

.bad {
  border-color: var(--color-danger);
  background: rgb(255 0 0 / 0.1);
}
</style>

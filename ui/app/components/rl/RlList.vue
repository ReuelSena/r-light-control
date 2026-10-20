<script setup lang="ts" generic="T extends { id: string; label?: string; detail?: string }">
// Lista de itens selecionáveis, com linhas altas para toque. O slot `item` personaliza a linha.
// Com `reorderable`, arrastar uma linha a leva para outra posição: no mouse, clicar e arrastar; no
// toque, segurar e arrastar. O `reorder` diz o item e a posição final dele (a partir de 0).
const props = defineProps<{ items: T[]; reorderable?: boolean }>()
const emit = defineEmits<{ reorder: [id: string, to: number] }>()
const selected = defineModel<string>()

const drag = ref<{ id: string; from: number; to: number; dy: number; active: boolean }>()
const list = ref<HTMLElement>()
let start = { x: 0, y: 0 }
let holdTimer: ReturnType<typeof setTimeout> | undefined
let pointer = 0
let skipClick = false

const rows = () => [...list.value!.children] as HTMLElement[]

function activate() {
  if (!drag.value) return
  drag.value.active = true
  skipClick = true
  list.value!.setPointerCapture(pointer)
}

function down(e: PointerEvent) {
  const row = props.reorderable && e.button <= 0 ? (e.target as HTMLElement).closest('li') : null
  skipClick = false
  if (!row) return
  const from = rows().indexOf(row)
  start = { x: e.clientX, y: e.clientY }
  pointer = e.pointerId
  drag.value = { id: props.items[from]!.id, from, to: from, dy: 0, active: false }
  if (e.pointerType !== 'mouse') holdTimer = setTimeout(activate, 400)
}

function move(e: PointerEvent) {
  const d = drag.value
  if (!d) return
  const dx = e.clientX - start.x
  d.dy = e.clientY - start.y
  const moved = Math.hypot(dx, d.dy)
  // No mouse começa ao mover 6 px; no toque, antes de segurar 400 ms o gesto rola a lista
  if (!d.active && e.pointerType === 'mouse' && moved > 6) activate()
  if (!d.active) {
    if (moved > 8) clearTimeout(holdTimer)
    return
  }
  // Posição final: antes da primeira linha (sem contar a arrastada) cujo meio está abaixo do ponteiro
  const others = rows().filter((_, i) => i !== d.from)
  const at = others.findIndex((li) => e.clientY < li.getBoundingClientRect().top + li.offsetHeight / 2)
  d.to = at < 0 ? others.length : at
}

function up() {
  clearTimeout(holdTimer)
  const d = drag.value
  drag.value = undefined
  if (d?.active && d.to !== d.from) emit('reorder', d.id, d.to)
}

function cancel() {
  clearTimeout(holdTimer)
  drag.value = undefined
}

// Depois de um arrasto, o clique que o navegador ainda manda não escolhe a linha
function pick(id: string) {
  if (skipClick) return (skipClick = false)
  selected.value = id
}

// Filete do destino: antes da linha que vai ficar depois da arrastada, ou depois da última
const indicator = (i: number) => {
  const d = drag.value
  if (!d?.active || d.to === d.from || i === d.from) return undefined
  const k = i > d.from ? i - 1 : i // posição na lista sem a arrastada
  if (k === d.to) return 'before'
  return d.to === props.items.length - 1 && k === d.to - 1 ? 'after' : undefined
}
</script>

<template>
  <!-- Durante o arrasto por toque, o movimento não pode virar rolagem -->
  <ul
    ref="list"
    class="rl-list sunken"
    role="listbox"
    @pointerdown="down"
    @pointermove="move"
    @pointerup="up"
    @pointercancel="cancel"
    @touchmove="drag?.active && $event.preventDefault()"
  >
    <li
      v-for="(item, i) in items"
      :key="item.id"
      role="option"
      :aria-selected="item.id === selected"
      :class="[indicator(i), { active: item.id === selected, dragging: drag?.active && i === drag.from }]"
      :style="drag?.active && i === drag.from ? { transform: `translateY(${drag.dy}px)` } : undefined"
      @click="pick(item.id)"
    >
      <slot name="item" :item="item">
        <span class="truncate">{{ item.label }}</span>
        <small v-if="item.detail">{{ item.detail }}</small>
      </slot>
    </li>
  </ul>
</template>

<style scoped>
.rl-list {
  overflow: auto;
  border-radius: 6px;
}

li {
  position: relative;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
  min-height: 48px;
  padding: 0 12px;
  border-bottom: 1px solid #000;
  box-shadow: inset 0 1px 0 rgb(255 255 255 / 0.03);
  cursor: pointer;
}

li.active {
  background: color-mix(in srgb, var(--color-select) 22%, transparent);
  box-shadow: inset 3px 0 0 var(--color-select);
}

/* Arrastada: segue o ponteiro por cima das outras */
li.dragging {
  z-index: 5;
  opacity: 0.85;
  background: var(--color-surface-3);
  box-shadow: 0 6px 16px rgb(0 0 0 / 0.6);
  pointer-events: none;
}

/* Filete do destino, antes ou depois da linha */
li.before::before,
li.after::after {
  content: '';
  position: absolute;
  z-index: 6;
  right: 0;
  left: 0;
  height: 3px;
  background: var(--color-on);
}

li.before::before {
  top: -2px;
}

li.after::after {
  bottom: -2px;
}

small {
  flex-shrink: 0;
  color: var(--color-fg-muted);
  font-size: 12px;
}
</style>

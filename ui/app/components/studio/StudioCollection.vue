<script setup lang="ts">
import { Copy, Plus, Trash2 } from 'lucide-vue-next'
import type { Collection } from '~/types/generated/Collection'
import type { Function } from '~/types/generated/Function'
import type { Priority } from '~/types/generated/Priority'
import type { TimelineItem } from '~/types/generated/TimelineItem'

// Editor da coleção como linha do tempo. Início, duração e comprimento são u32 em milésimos da unidade
// (ms ou milésimos de batida), então 1000 é uma unidade inteira da régua nos dois casos. As trilhas são a
// `layer`; a prioridade (e não a trilha) decide a mistura. Durante o arraste só o rascunho local muda: a
// operação sai ao soltar.
const props = defineProps<{ fn: Extract<Function, { type: 'collection' }> }>()
const { t } = useI18n()
const { state, send } = useEngine()

const ROW = 40
const zoomSteps = [20, 30, 45, 60, 90, 120, 180]
const orders = ['loop', 'singleShot', 'pingPong'] as const
const priorityColor: Record<Priority, string> = { low: 'var(--color-position)', normal: 'var(--color-intensity)', high: 'var(--color-speed)' }

const adding = ref(false)
const selected = ref<number | null>(null)
const zoom = ref(60) // px por unidade
const draft = ref<Pick<Collection, 'items' | 'length'> | null>(null)

const items = computed(() => draft.value?.items ?? props.fn.items)
const length = computed(() => (draft.value ? draft.value.length : props.fn.length))
const autoEnd = computed(() => Math.max(0, ...items.value.map((i) => (i.duration === null ? 0 : i.start + i.duration))))
const rows = computed(() => Math.max(3, ...items.value.map((i) => i.layer + 2)))
// Unidades visíveis: o conteúdo mais um respiro, para ter onde arrastar para a direita
const extent = computed(() => Math.max(8, Math.ceil(Math.max(length.value ?? autoEnd.value, ...items.value.map((i) => i.start)) / 1000) + 4))
const ticks = computed(() => Array.from({ length: Math.ceil(extent.value / Math.ceil(40 / zoom.value)) }, (_, k) => k * Math.ceil(40 / zoom.value)))
const clock = computed(() => state.value.timelineClock.find((c) => c.collection === props.fn.id)?.position)

const px = (v: number) => (v / 1000) * zoom.value
const fnOf = (item: TimelineItem) => state.value.project.functions.find((f) => f.id === item.function)
const colorOf = (item: TimelineItem) => priorityColor[item.priority ?? props.fn.priority]
const number = (e: Event) => Number((e.target as HTMLInputElement).value) || 0

function save(changes: Partial<Collection>) {
  return send({ op: 'saveFunction', function: { ...props.fn, ...changes } })
}

function saveItem(i: number, changes: Partial<TimelineItem>) {
  save({ items: props.fn.items.map((item, k) => (k === i ? { ...item, ...changes } : item)) })
}

function add(id: string) {
  const layer = Math.min(255, Math.max(-1, ...props.fn.items.map((i) => i.layer)) + 1)
  save({ items: [...props.fn.items, { function: id, start: 0, duration: null, layer, priority: null }] })
  selected.value = props.fn.items.length
}

function remove(i: number) {
  selected.value = null
  save({ items: props.fn.items.filter((_, k) => k !== i) })
}

function duplicate(i: number) {
  const item = props.fn.items[i]!
  save({ items: [...props.fn.items, { ...item, start: item.start + (item.duration ?? 1000) }] })
  selected.value = props.fn.items.length
}

type Mode = 'move' | 'left' | 'right' | 'length'

// Arraste por eventos de ponteiro na janela (mouse e toque, com `touch-action: none` nos clipes).
// Encaixa nas unidades inteiras; Alt solta o encaixe (passo de 10).
function down(e: PointerEvent, i: number, mode: Mode) {
  if (mode !== 'length') selected.value = i
  const base = { items: props.fn.items, length: props.fn.length }
  const orig = base.items[i]
  const x0 = e.clientX
  const y0 = e.clientY
  const move = (ev: PointerEvent) => {
    const step = ev.altKey ? 10 : 1000
    const snap = (v: number) => Math.max(0, Math.round(v / step) * step)
    const dt = ((ev.clientX - x0) / zoom.value) * 1000
    if (mode === 'length') {
      draft.value = { items: base.items, length: Math.max(step, snap((base.length ?? autoEnd.value) + dt)) }
      return
    }
    const o = orig!
    const end = o.duration === null ? extent.value * 1000 : o.start + o.duration
    let changes: Partial<TimelineItem>
    if (mode === 'move') {
      changes = { start: snap(o.start + dt), layer: Math.min(255, Math.max(0, Math.round(o.layer + (ev.clientY - y0) / ROW))) }
    } else if (mode === 'right') {
      changes = { duration: Math.max(step, snap(end + dt) - o.start) }
    } else {
      const start = Math.min(snap(o.start + dt), end - step)
      changes = o.duration === null ? { start } : { start, duration: end - start }
    }
    draft.value = { items: base.items.map((item, k) => (k === i ? { ...item, ...changes } : item)), length: base.length }
  }
  const up = async () => {
    window.removeEventListener('pointermove', move)
    window.removeEventListener('pointerup', up)
    window.removeEventListener('pointercancel', up)
    const d = draft.value
    if (!d) return
    try {
      if (JSON.stringify(d) !== JSON.stringify(base)) await save(d)
    } finally {
      draft.value = null
    }
  }
  window.addEventListener('pointermove', move)
  window.addEventListener('pointerup', up)
  window.addEventListener('pointercancel', up)
}

function wheel(e: WheelEvent) {
  if (!e.ctrlKey) return
  e.preventDefault()
  zoom.value = stepValue(zoomSteps, zoom.value, e.deltaY < 0 ? 1 : -1)
}
</script>

<template>
  <div class="flex flex-col gap-3">
    <!-- Barra: o que entra, como repete e o comprimento; o zoom fica no fim -->
    <div class="flex flex-wrap items-center gap-x-4 gap-y-2">
      <RlButton size="sm" @press="adding = true">
        <Plus :size="16" />
        {{ t('collection.add') }}
      </RlButton>
      <RlSegmented
        size="sm"
        :model-value="orders.includes(fn.runOrder as never) ? (fn.runOrder as (typeof orders)[number]) : 'loop'"
        :options="orders.map((value) => ({ value, label: t(`runOrder.${value}`) }))"
        @update:model-value="(runOrder) => save({ runOrder })"
      />
      <span class="flex items-center gap-1.5 text-sm">
        <RlSwitch :model-value="fn.beats" @update:model-value="(beats) => save({ beats })">{{ t('collection.beats') }}</RlSwitch>
        <RlInfo :text="t('collection.beatsInfo')" />
      </span>
      <span class="flex items-center gap-1.5 text-sm">
        {{ t('collection.length') }}
        <RlInfo :text="t('collection.lengthInfo')" />
        <RlTimeField class="w-28" :beats="fn.beats" :model-value="fn.length ?? autoEnd" @update:model-value="(ms) => save({ length: ms || null })" />
        <RlButton v-if="fn.length !== null" size="sm" @press="save({ length: null })">{{ t('collection.auto') }}</RlButton>
      </span>
    </div>

    <!-- Linha do tempo: o cabeçalho tem o zoom; a coluna fixa à esquerda numera as trilhas e diz a unidade da régua -->
    <div class="sunken rounded-md">
      <div class="flex items-center justify-between gap-2 border-b border-line px-2 py-1.5">
        <span class="flex items-center gap-1.5 text-sm font-bold">{{ t('collection.timeline') }} <RlInfo :text="t('collection.timelineInfo')" /></span>
        <RlStepper v-model="zoom" :steps="zoomSteps" :default="60" :format="(v) => `${Math.round((v / 60) * 100)}%`" />
      </div>
      <div class="flex">
      <div class="shrink-0 border-r border-line text-xs text-fg-muted" style="width: 36px">
        <div class="flex h-6 items-center justify-center border-b border-line">{{ fn.beats ? 'bt' : 's' }}</div>
        <div v-for="r in rows" :key="r" class="flex items-center justify-center" :style="{ height: `${ROW}px` }">{{ r }}</div>
      </div>
      <div class="min-w-0 flex-1 overflow-x-auto" @wheel="wheel">
        <div class="relative" :style="{ width: `${extent * zoom}px` }">
          <div class="relative h-6 border-b border-line text-xs text-fg-muted">
            <span v-for="tick in ticks" :key="tick" class="absolute top-1 pl-1" :style="{ left: `${tick * zoom}px` }">{{ tick }}</span>
          </div>
          <div
            class="relative"
            :style="{
              height: `${rows * ROW}px`,
              backgroundImage: `linear-gradient(90deg, var(--color-line) 1px, transparent 1px), linear-gradient(var(--color-line) 1px, transparent 1px)`,
              backgroundSize: `${zoom}px 100%, 100% ${ROW}px`,
            }"
            @pointerdown.self="selected = null"
          >
            <div
              v-for="(item, i) in items"
              :key="i"
              class="absolute flex cursor-grab items-center gap-1.5 overflow-hidden rounded px-2 text-sm font-bold text-fg select-none active:cursor-grabbing"
              :class="selected === i ? 'outline-2 outline-select' : ''"
              :style="{
                left: `${px(item.start)}px`,
                top: `${item.layer * ROW + 3}px`,
                width: `${item.duration === null ? ((extent * 1000 - item.start) * zoom) / 1000 : Math.max(px(item.duration), 8)}px`,
                height: `${ROW - 6}px`,
                touchAction: 'none',
                border: `1px solid ${colorOf(item)}`,
                background: `color-mix(in srgb, ${colorOf(item)} 35%, #000)`,
              }"
              @pointerdown="down($event, i, 'move')"
            >
              <component :is="functionIcons[fnOf(item)?.type ?? 'scene']" :size="16" class="shrink-0" />
              <span class="truncate">{{ fnOf(item)?.name }}</span>
              <span v-if="item.duration === null" class="ml-auto shrink-0">∞</span>
              <span v-if="item.duration !== null" class="absolute inset-y-0 left-0 w-2 cursor-ew-resize bg-white/20" @pointerdown.stop="down($event, i, 'left')" />
              <span class="absolute inset-y-0 right-0 w-2 cursor-ew-resize bg-white/20" @pointerdown.stop="down($event, i, 'right')" />
            </div>
            <div v-if="clock !== undefined" class="pointer-events-none absolute inset-y-0 w-0.5 bg-lcd" :style="{ left: `${px(clock)}px` }" />
          </div>
          <!-- Comprimento do laço: a linha atravessa régua e trilhas; a alça fica na régua -->
          <div class="pointer-events-none absolute top-0 bottom-0 w-3 -translate-x-1/2" :style="{ left: `${px(length ?? autoEnd)}px` }">
            <div class="mx-auto h-full w-0.5" :class="length === null ? 'bg-fg-dim' : 'bg-select'" />
            <div class="pointer-events-auto absolute top-0 h-6 w-3 cursor-ew-resize" style="touch-action: none" @pointerdown="down($event, 0, 'length')" />
          </div>
        </div>
      </div>
      </div>
    </div>

    <!-- Os mesmos clipes em campos numéricos, com a cor da prioridade. Largo: uma linha por clipe, com cabeçalho. Estreito: um cartão por clipe -->
    <div v-if="items.length" class="sunken rounded-md">
      <div class="hidden grid-cols-[minmax(9rem,1fr)_7rem_9rem_4.5rem_9rem_5.5rem] items-center gap-2 border-b border-line px-2 py-1.5 text-xs text-fg-muted @3xl:grid">
        <span>{{ t('collection.clip') }}</span>
        <span>{{ t('collection.start') }}</span>
        <span>{{ t('collection.duration') }}</span>
        <span>{{ t('collection.track') }}</span>
        <span class="flex items-center gap-1">{{ t('studio.priority') }} <RlInfo :text="t('collection.priorityInfo')" /></span>
        <span />
      </div>
      <div
        v-for="(item, i) in items"
        :key="i"
        class="grid grid-cols-2 items-end gap-2 border-b border-line px-2 py-2 last:border-b-0 @3xl:grid-cols-[minmax(9rem,1fr)_7rem_9rem_4.5rem_9rem_5.5rem] @3xl:items-center @3xl:py-1.5"
        :class="selected === i ? 'bg-select/15' : ''"
        @pointerdown="selected = i"
      >
        <span class="col-span-2 flex min-w-0 items-center gap-2 @3xl:col-span-1">
          <span class="size-2.5 shrink-0 rounded-full" :style="{ background: colorOf(item) }" />
          <component :is="functionIcons[fnOf(item)?.type ?? 'scene']" :size="16" class="shrink-0 text-fg-muted" />
          <span class="truncate">{{ fnOf(item)?.name }}</span>
        </span>
        <label class="field-label @3xl:text-base">
          <span class="@3xl:hidden">{{ t('collection.start') }}</span>
          <RlTimeField :beats="fn.beats" :model-value="item.start" @update:model-value="(start) => saveItem(i, { start })" />
        </label>
        <label class="field-label @3xl:text-base">
          <span class="@3xl:hidden">{{ t('collection.duration') }}</span>
          <RlTimeField
            :beats="fn.beats"
            infinite
            :model-value="item.duration ?? INFINITE"
            @update:model-value="(ms) => saveItem(i, { duration: ms === INFINITE ? null : Math.max(ms, 10) })"
          />
        </label>
        <label class="field-label @3xl:text-base">
          <span class="@3xl:hidden">{{ t('collection.track') }}</span>
          <input class="field" type="number" min="1" max="256" :value="item.layer + 1" @change="saveItem(i, { layer: Math.min(255, Math.max(0, Math.round(number($event)) - 1)) })">
        </label>
        <label class="field-label @3xl:text-base">
          <span class="@3xl:hidden">{{ t('studio.priority') }}</span>
          <select class="field" :value="item.priority ?? 'inherit'" @change="saveItem(i, { priority: ($event.target as HTMLSelectElement).value === 'inherit' ? null : (($event.target as HTMLSelectElement).value as Priority) })">
            <option value="inherit">{{ t('collection.inherit') }}</option>
            <option v-for="p in (['low', 'normal', 'high'] as const)" :key="p" :value="p">{{ t(`priority.${p}`) }}</option>
          </select>
        </label>
        <span class="col-span-2 flex justify-end gap-1 @3xl:col-span-1">
          <RlButton size="sm" :aria-label="t('common.duplicate')" :title="t('common.duplicate')" @press="duplicate(i)"><Copy :size="14" /></RlButton>
          <RlButton size="sm" :aria-label="t('common.remove')" :title="t('common.remove')" @press="remove(i)"><Trash2 :size="14" /></RlButton>
        </span>
      </div>
    </div>
    <p v-else class="text-sm text-fg-muted">{{ t('collection.empty') }}</p>

    <RlDialog v-model:open="adding" :title="t('collection.add')" sheet>
      <div class="grid gap-1.5">
        <RlButton
          v-for="f in state.project.functions.filter((x) => x.id !== fn.id && !contains(state.project.functions, x, fn.id))"
          :key="f.id"
          class="justify-start!"
          @press="add(f.id); adding = false"
        >
          <component :is="functionIcons[f.type]" :size="18" />
          {{ f.name }}
        </RlButton>
      </div>
    </RlDialog>
  </div>
</template>

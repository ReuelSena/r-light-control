<script setup lang="ts">
import {
  AlignEndHorizontal, AlignEndVertical, AlignStartHorizontal, AlignStartVertical, Copy, EyeOff, Keyboard, LayoutTemplate,
  Monitor, Pencil, Plus, Settings2, Smartphone, Tablet, TextCursorInput, Trash2,
} from 'lucide-vue-next'
import type { Rect } from '~/types/generated/Rect'
import type { VcWidget } from '~/types/generated/VcWidget'

// Console (E11, E12): páginas de elementos montados pelo usuário, com a seção master ao lado.
// Cada aparelho abre o layout do seu formato (VC-2): os elementos são os mesmos, só a posição muda.
// Em edição: paleta, propriedades (VC-1), alinhamento, páginas e cópia de layout entre formatos.
const { t } = useI18n()
const { state, send } = useEngine()
const prefs = usePrefs()
const vc = useConsole()
const project = computed(() => state.value.project)
const phone = useMediaQuery('(max-width: 639px)')
const tablet = useMediaQuery('(max-width: 1023px)')
const format = computed<ConsoleFormat>(() =>
  prefs.value.consoleFormat !== 'auto' ? prefs.value.consoleFormat : phone.value ? 'phone' : tablet.value ? 'tablet' : 'desktop',
)
const formatIcons = { phone: Smartphone, tablet: Tablet, desktop: Monitor }

const page = ref(project.value.console.pages[0]?.id ?? '')
// Fora da tela (no app.vue), decide se o Ctrl + Z desfaz (edição) ou vai para os elementos (operação)
const editing = useState('consoleEditing', () => false)
onBeforeUnmount(() => (editing.value = false))
const selection = ref<string[]>([])
// Copiados com Ctrl + C, para colar com Ctrl + V (também em outra página ou formato)
const clipboard = useState<VcWidget[]>('consoleClipboard', () => [])
const canvas = ref<{ nudge: (cx: number, cy: number) => void }>()
const propsOpen = ref(false)
const renaming = ref(false)
const copying = ref(false)
const placing = ref(false)
const confirmRemove = ref<'widgets' | 'page'>()
const notice = ref('')

watch([page, editing, format], () => (selection.value = []))
watch(selection, () => (confirmRemove.value = undefined))
watch(() => project.value.console.pages, (pages) => {
  if (!pages.some((p) => p.id === page.value)) page.value = pages[0]?.id ?? ''
})

const pageTabs = computed(() => project.value.console.pages.map((p) => ({ value: p.id, label: p.name })))
const widgets = computed(() => project.value.console.widgets.filter((w) => w.page === page.value))
const selected = computed(() => widgets.value.filter((w) => selection.value.includes(w.id)))
const cols = computed(() => formatColumns[format.value])
const rectIn = (w: VcWidget) => w.layouts[format.value]
// Fora deste formato: os que não têm posição nele (o filho de uma moldura fora do formato volta com ela)
const unplaced = computed(() => widgets.value.filter((w) => !rectIn(w) && (!w.frame || widgets.value.some((f) => f.id === w.frame && rectIn(f)))))
const clone = <T,>(value: T): T => JSON.parse(JSON.stringify(value))
const say = (text: string) => {
  notice.value = text
  setTimeout(() => notice.value === text && (notice.value = ''), 3000)
}

// Lugar livre na página ou dentro de uma moldura (abaixo do cabeçalho); `reserved`: o que já foi separado
function spot(frame: string | null, w: number, h: number, reserved: Rect[] = []): Rect {
  const box = frame ? rectIn(widgets.value.find((f) => f.id === frame)!) : undefined
  const taken = widgets.value.filter((x) => x.frame === frame && rectIn(x)).map((x) => rectIn(x)!)
  return freeSpot([...taken, ...reserved], w, h, box?.w ?? cols.value, box ? frameHeader : 0)
}

// ---------- Paleta ----------
function blank(type: WidgetType): VcWidget {
  const firstOf = (kind: string) => project.value.functions.find((f) => f.type === kind)?.id ?? null
  const kinds = {
    button: { type, action: 'function', function: null, fixtures: [], groups: [], mode: 'toggle', highlight: 'face', icon: null },
    slider: { type, mode: 'level', channelGroups: [], function: null, knob: false },
    cueList: { type, chaser: firstOf('chaser') },
    xyPad: { type, fixtures: [] },
    speedDial: { type, functions: [], fadeIn: true, fadeOut: true, hold: false, beats: true },
    frame: { type, solo: false },
    clock: { type, mode: 'clock', countdownS: 300 },
    animation: { type, matrix: firstOf('rgbMatrix') },
    label: { type, background: null, border: null },
  }
  const [w, h] = widgetKinds[type].size
  const layouts = { phone: null, tablet: null, desktop: null, [format.value]: spot(null, w, h) }
  return { id: uuidv7(), page: page.value, frame: null, caption: t(`vc.kind.${type}`), color: '#ff8000', layouts, inputs: [], ...kinds[type] } as VcWidget
}

function add(type: WidgetType) {
  const widget = blank(type)
  send({ op: 'saveWidgets', widgets: [widget] })
  selection.value = [widget.id]
}

// ---------- Escolhidos ----------
// Põe cópias dos elementos nesta página e neste formato, cada uma no primeiro lugar livre. Dentro de uma
// moldura só se a moldura estiver nesta página; senão a cópia fica solta. Na duplicação, o tamanho vem do
// formato aberto; um elemento copiado de outro formato (ou escondido em todos) usa o tamanho que tiver.
function paste(sources: VcWidget[]) {
  const copies: VcWidget[] = []
  for (const w of sources) {
    const r = rectIn(w) ?? layoutFormats.map((f) => w.layouts[f]).find(Boolean) ?? { w: widgetKinds[w.type].size[0], h: widgetKinds[w.type].size[1] }
    const frame = widgets.value.some((f) => f.id === w.frame && rectIn(f)) ? w.frame : null
    const reserved = copies.filter((c) => c.frame === frame).map((c) => c.layouts[format.value]!)
    copies.push({ ...clone(w), id: uuidv7(), page: page.value, frame, layouts: { phone: null, tablet: null, desktop: null, [format.value]: spot(frame, r.w, r.h, reserved) } })
  }
  send({ op: 'saveWidgets', widgets: copies })
  selection.value = copies.map((c) => c.id)
}

const duplicate = () => paste(selected.value.map((w) => ({ ...w, caption: t('studio.copyName', { name: w.caption }) })))

function removeNow() {
  send({ op: 'removeWidgets', ids: selection.value })
  selection.value = []
}

function remove() {
  if (confirmRemove.value !== 'widgets') return (confirmRemove.value = 'widgets')
  removeNow()
}

// Só sai deste formato; nos outros continua onde está (VC-2)
function hide() {
  const gone = new Set(selection.value)
  const affected = widgets.value.filter((w) => gone.has(w.id) || gone.has(w.frame ?? ''))
  send({ op: 'saveWidgets', widgets: affected.map((w) => ({ ...w, layouts: { ...w.layouts, [format.value]: null } })) })
  selection.value = []
}

// Alinha pela borda do primeiro escolhido; não alinha se der sobreposição
function align(side: 'left' | 'right' | 'top' | 'bottom') {
  const ref = rectIn(selected.value[0]!)!
  const moved = selected.value.map((w) => {
    const r = { ...rectIn(w)! }
    if (side === 'left') r.x = ref.x
    if (side === 'right') r.x = ref.x + ref.w - r.w
    if (side === 'top') r.y = ref.y
    if (side === 'bottom') r.y = ref.y + ref.h - r.h
    return { ...w, layouts: { ...w.layouts, [format.value]: r } }
  })
  const others = widgets.value.filter((w) => w.frame === selected.value[0]!.frame && rectIn(w) && !selection.value.includes(w.id)).map((w) => rectIn(w)!)
  const rects = moved.map((w) => w.layouts[format.value]!)
  const clash = rects.some((r, i) => r.x < 0 || r.y < 0 || others.some((o) => overlaps(o, r)) || rects.some((o, j) => i !== j && overlaps(o, r)))
  if (clash) return say(t('vc.alignClash'))
  send({ op: 'saveWidgets', widgets: moved })
}

// ---------- Páginas e formatos ----------
async function addPage() {
  const id = uuidv7()
  await send({ op: 'saveConsolePage', page: { id, name: t('vc.pageName', { n: project.value.console.pages.length + 1 }) } })
  page.value = id
}

function removePage() {
  if (confirmRemove.value !== 'page') return (confirmRemove.value = 'page')
  send({ op: 'removeConsolePage', id: page.value })
  confirmRemove.value = undefined
}

// Substitui o layout deste formato pelo do outro
function copyFrom(from: ConsoleFormat) {
  const copies = clone(widgets.value)
  for (const w of copies) w.layouts[format.value] = null
  copyLayout(copies, from, format.value)
  send({ op: 'saveWidgets', widgets: copies })
  copying.value = false
}

// Volta ao formato no primeiro lugar livre; uma moldura traz os filhos que estavam fora
function place(w: VcWidget) {
  const size = rectIn(w) ?? layoutFormats.map((f) => w.layouts[f]).find(Boolean) ?? { w: widgetKinds[w.type].size[0], h: widgetKinds[w.type].size[1] }
  const placedWidget = { ...w, layouts: { ...w.layouts, [format.value]: spot(w.frame, size.w, size.h) } }
  const children = widgets.value.filter((c) => c.frame === w.id && !rectIn(c))
  const inner: Rect[] = []
  const box = placedWidget.layouts[format.value]!
  const placedChildren = children.map((c) => {
    const r = layoutFormats.map((f) => c.layouts[f]).find(Boolean) ?? { w: 16, h: 16 }
    const rect = freeSpot(inner, r.w, r.h, box.w, frameHeader)
    inner.push(rect)
    return { ...c, layouts: { ...c.layouts, [format.value]: rect } }
  })
  box.h = Math.max(box.h, ...inner.map((r) => r.y + r.h))
  send({ op: 'saveWidgets', widgets: [placedWidget, ...placedChildren] })
  if (unplaced.value.length <= 1 + children.length) placing.value = false
}

// ---------- Atalhos de teclado (fora da edição) ----------
// Atalhos padrão: seção master no teclado numérico, Editar e dicas de atalho; valem também na edição
const shortcuts: Record<string, () => unknown> = {
  'Num -': () => send({ op: 'setBlackout', on: !state.value.master.blackout }),
  'Num +': () => send({ op: 'stopAllFunctions' }),
  'Num Enter': () => vc.tapBpm(),
  'Alt+E': () => (editing.value = !editing.value),
  'Alt+I': () => (prefs.value.keyHints = !prefs.value.keyHints),
}

// Na edição, atalhos dos elementos escolhidos: Delete remove; Ctrl + D duplica; Ctrl + C e Ctrl + V copiam
// e colam; as setas deslocam 4 células (com Shift, 8; com Ctrl, 1, o menor passo da grade)
const arrows: Record<string, [number, number]> = { ArrowLeft: [-1, 0], ArrowRight: [1, 0], ArrowUp: [0, -1], ArrowDown: [0, 1] }
const editShortcuts: Record<string, () => unknown> = {
  Delete: () => selection.value.length && removeNow(),
  'Ctrl+D': () => selection.value.length && duplicate(),
  'Ctrl+C': () => selection.value.length && (clipboard.value = clone(selected.value)),
  'Ctrl+V': () => clipboard.value.length && paste(clipboard.value),
}

function editKey(e: KeyboardEvent) {
  const arrow = arrows[e.key]
  if (e.altKey || (arrow && !selection.value.length)) return false
  const step = e.ctrlKey ? 1 : e.shiftKey ? 8 : 4
  const action = arrow ? () => canvas.value?.nudge(arrow[0] * step, arrow[1] * step) : editShortcuts[keyCombo(e)]
  if (!action) return false
  e.preventDefault()
  // Só as setas repetem enquanto a tecla fica apertada
  if (arrow || !e.repeat) action()
  return true
}

function onKey(e: KeyboardEvent, down: boolean) {
  const target = e.target as HTMLElement
  // Digitando num campo, só valem as combinações com Alt (que não escrevem nada)
  if (!e.altKey && target.closest('input, textarea, select')) return
  // Atrás de um diálogo aberto, o teclado não mexe nos elementos
  if (down && editing.value && !target.closest('[role="dialog"]') && editKey(e)) return
  if (e.repeat) return
  // Teclas reservadas nunca acionam elementos (o zoom com Ctrl fica no app.vue)
  if (isReservedKey(e)) {
    const action = down && shortcuts[keyCombo(e)]
    if (action) {
      e.preventDefault()
      action()
    }
    return
  }
  if (editing.value) return
  const combo = keyCombo(e)
  for (const w of widgets.value) {
    for (const input of w.inputs) {
      if (input.key !== combo) continue
      e.preventDefault()
      vc.input(w, input.control, down)
    }
  }
}
const keydown = (e: KeyboardEvent) => onKey(e, true)
const keyup = (e: KeyboardEvent) => onKey(e, false)
onMounted(() => {
  window.addEventListener('keydown', keydown)
  window.addEventListener('keyup', keyup)
})
onBeforeUnmount(() => {
  window.removeEventListener('keydown', keydown)
  window.removeEventListener('keyup', keyup)
})
</script>

<template>
  <div class="flex h-full flex-col gap-2 p-2 @3xl:flex-row">
    <ConsoleMaster class="@3xl:order-last" />

    <div class="flex min-h-0 min-w-0 flex-1 flex-col gap-2">
      <RlTabs v-model="page" :tabs="pageTabs" class="min-h-0 flex-1">
        <template #actions>
          <div class="flex items-center gap-2">
            <RlStepper v-model="prefs.consoleZoom" :steps="zoomSteps" :title="t('vc.zoom')" />
            <RlButton
              v-model="prefs.keyHints"
              mode="toggle"
              highlight="led"
              size="sm"
              :aria-label="t('vc.keyHints')"
              :title="`${t('vc.keyHints')} (Alt+I)`"
            >
              <Keyboard :size="16" />
            </RlButton>
            <RlButton v-model="editing" mode="toggle" highlight="led" size="sm" :title="`${t('vc.edit')} (Alt+E)`">
              <Pencil :size="16" />
              {{ t('vc.edit') }}
            </RlButton>
          </div>
        </template>

        <div class="flex h-full min-h-0 flex-col gap-2">
          <div v-if="editing" class="flex flex-col gap-1.5">
            <!-- Formato em edição e paleta (o toque adiciona no primeiro lugar livre) -->
            <div class="flex flex-wrap items-center gap-1">
              <RlLcd class="flex items-center gap-1.5">
                <component :is="formatIcons[format]" :size="14" />
                {{ t(`vc.formats.${format}`) }}
              </RlLcd>
              <RlInfo :text="t('vc.info.formats')" />
              <RlButton v-for="type in widgetTypes" :key="type" size="sm" :title="t(`vc.kind.${type}`)" @press="add(type)">
                <component :is="widgetKinds[type].icon" :size="16" />
                <span class="@max-5xl:hidden">{{ t(`vc.kind.${type}`) }}</span>
              </RlButton>
            </div>
            <div v-if="selection.length" class="flex flex-wrap items-center gap-1">
              <RlButton v-if="selection.length === 1" class="@5xl:hidden!" size="sm" @press="propsOpen = true">
                <Settings2 :size="16" />
                {{ t('vc.properties') }}
              </RlButton>
              <template v-if="selection.length > 1">
                <RlButton size="sm" :aria-label="t('vc.align.left')" :title="t('vc.align.left')" @press="align('left')"><AlignStartVertical :size="16" /></RlButton>
                <RlButton size="sm" :aria-label="t('vc.align.right')" :title="t('vc.align.right')" @press="align('right')"><AlignEndVertical :size="16" /></RlButton>
                <RlButton size="sm" :aria-label="t('vc.align.top')" :title="t('vc.align.top')" @press="align('top')"><AlignStartHorizontal :size="16" /></RlButton>
                <RlButton size="sm" :aria-label="t('vc.align.bottom')" :title="t('vc.align.bottom')" @press="align('bottom')"><AlignEndHorizontal :size="16" /></RlButton>
              </template>
              <RlButton size="sm" @press="duplicate"><Copy :size="16" />{{ t('common.duplicate') }}</RlButton>
              <RlButton size="sm" @press="hide"><EyeOff :size="16" />{{ t('vc.hide') }}</RlButton>
              <RlButton size="sm" :model-value="confirmRemove === 'widgets'" color="var(--color-danger)" @press="remove">
                <Trash2 :size="16" />
                {{ confirmRemove === 'widgets' ? t('common.confirmRemove') : t('common.remove') }}
              </RlButton>
            </div>
            <div v-else class="flex flex-wrap items-center gap-1">
              <RlButton size="sm" @press="addPage"><Plus :size="16" />{{ t('vc.addPage') }}</RlButton>
              <RlButton size="sm" @press="renaming = true"><TextCursorInput :size="16" />{{ t('common.rename') }}</RlButton>
              <RlButton
                v-if="pageTabs.length > 1"
                size="sm"
                :model-value="confirmRemove === 'page'"
                color="var(--color-danger)"
                @press="removePage"
              >
                <Trash2 :size="16" />
                {{ confirmRemove === 'page' ? t('common.confirmRemove') : t('vc.removePage') }}
              </RlButton>
              <RlButton size="sm" @press="copying = true"><LayoutTemplate :size="16" />{{ t('vc.copyLayout') }}</RlButton>
              <RlButton v-if="unplaced.length" size="sm" @press="placing = true"><EyeOff :size="16" />{{ t('vc.unplaced', { n: unplaced.length }) }}</RlButton>
            </div>
            <p v-if="notice" class="text-sm text-flash">{{ notice }}</p>
          </div>

          <div class="min-h-0 flex-1 overflow-auto">
            <ConsoleCanvas ref="canvas" v-model:selection="selection" :widgets="widgets" :format="format" :editing="editing" :zoom="prefs.consoleZoom" />
            <p v-if="!widgets.some(rectIn)" class="p-4 text-sm text-fg-muted">{{ editing ? t('vc.emptyEditing') : t('vc.empty') }}</p>
          </div>
        </div>
      </RlTabs>
    </div>

    <!-- Na edição, o painel fica sempre no lugar: escolher ou soltar um elemento não mexe no layout -->
    <aside v-if="editing" class="panel w-80 shrink-0 overflow-y-auto @max-5xl:hidden">
      <ConsoleProperties v-if="selected.length === 1" :widget="selected[0]!" />
      <template v-else>
        <h2 class="heading">{{ t('vc.properties') }}</h2>
        <p class="text-sm text-fg-muted">{{ selected.length ? t('vc.manySelected', { n: selected.length }) : t('vc.pickWidget') }}</p>
      </template>
    </aside>

    <RlDialog v-model:open="propsOpen" sheet :title="t('vc.properties')">
      <ConsoleProperties v-if="selected.length === 1" :widget="selected[0]!" />
    </RlDialog>

    <RlDialog v-model:open="renaming" :title="t('common.rename')">
      <input
        class="field w-full"
        :value="pageTabs.find((p) => p.value === page)?.label"
        @change="send({ op: 'saveConsolePage', page: { id: page, name: ($event.target as HTMLInputElement).value } })"
        @keydown.enter="renaming = false"
      >
    </RlDialog>

    <RlDialog v-model:open="copying" :title="t('vc.copyLayout')" :info="t('vc.info.copyLayout')">
      <div class="grid gap-2">
        <RlButton v-for="f in layoutFormats.filter((x) => x !== format)" :key="f" size="lg" class="justify-start!" @press="copyFrom(f)">
          <component :is="formatIcons[f]" :size="20" />
          {{ t(`vc.formats.${f}`) }}
          <span class="ml-auto text-xs text-fg-muted normal-case!">{{ t('vc.placedCount', { n: widgets.filter((w) => w.layouts[f]).length }) }}</span>
        </RlButton>
      </div>
    </RlDialog>

    <RlDialog v-model:open="placing" sheet :title="t('vc.unplaced', { n: unplaced.length })">
      <div class="grid gap-1.5">
        <RlButton v-for="w in unplaced" :key="w.id" class="justify-start!" @press="place(w)">
          <component :is="widgetKinds[w.type].icon" :size="18" />
          {{ w.caption }}
          <Plus :size="16" class="ml-auto" />
        </RlButton>
      </div>
    </RlDialog>
  </div>
</template>

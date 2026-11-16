<script setup lang="ts">
import { ChevronDown, ChevronUp, Keyboard, Plus, Wand, X } from 'lucide-vue-next'
import type { FunctionType } from '~/types/generated/FunctionType'
import type { InputControl } from '~/types/generated/InputControl'
import type { MacroStep } from '~/types/generated/MacroStep'
import type { MidiControl } from '~/types/generated/MidiControl'
import type { VcWidget } from '~/types/generated/VcWidget'
import type { WidgetInput } from '~/types/generated/WidgetInput'

// Propriedades do elemento escolhido (VC-1): texto, cor, o que ele faz e as entradas externas
// (controle MIDI com "aprender" e tecla do teclado), como no QLC+.
const props = defineProps<{ widget: VcWidget }>()
const { t } = useI18n()
const { state, send } = useEngine()
const project = computed(() => state.value.project)
const kind = computed(() => widgetKinds[props.widget.type])

function save(patch: object) {
  send({ op: 'saveWidgets', widgets: [{ ...props.widget, ...patch } as VcWidget] })
}
const toggled = (list: string[], id: string) => (list.includes(id) ? list.filter((x) => x !== id) : [...list, id])
const functionsOf = (...types: FunctionType[]) => project.value.functions.filter((f) => types.includes(f.type))
const movers = computed(() => project.value.fixtures.filter((f) => modeChannels(project.value, f).some((c) => c.attribute === 'pan')))
const pick = (e: Event) => (e.target as HTMLSelectElement).value || null

// Ícone do botão: ao ligar, começa com o do tipo da criação (ou o da ação) e pode ser trocado
const { icon } = useIconLibrary()
const pickingIcon = ref(false)
const picking = ref(false)
function defaultIcon() {
  const w = props.widget
  if (w.type === 'macro') return 'Workflow'
  if (w.type !== 'button') return null
  if (w.action === 'stopAll') return 'OctagonX'
  if (w.action === 'previous') return 'ChevronLeft'
  if (w.action === 'pause') return 'Pause'
  if (w.action !== 'function' && w.action !== 'exclusive') return 'LightbulbOff'
  const type = project.value.functions.find((f) => f.id === w.function)?.type ?? 'scene'
  return { scene: 'Camera', chaser: 'Repeat', sequence: 'Film', collection: 'Layers', efx: 'Orbit', rgbMatrix: 'LayoutGrid', phaser: 'AudioWaveform' }[type]
}

// ---------- Macro (E6) ----------
const stepTypes = ['start', 'stop', 'go', 'stopAll', 'blackout', 'bpm', 'wait'] as const
const chasers = computed(() => functionsOf('chaser', 'sequence'))
// Um passo novo (ou que trocou de tipo) começa com a primeira criação, o blackout ligado, 120 BPM ou 1 s
function newStep(type: MacroStep['type']): MacroStep {
  const first = (type === 'go' ? chasers.value : project.value.functions)[0]?.id ?? ''
  return { start: { type, function: first }, stop: { type, function: first }, go: { type, chaser: first }, stopAll: { type }, blackout: { type, on: true }, bpm: { type, bpm: 120 }, wait: { type, ms: 1000 } }[type] as MacroStep
}
const macroSteps = computed(() => (props.widget.type === 'macro' ? props.widget.steps : []))
const setSteps = (steps: MacroStep[]) => save({ steps })
const setStep = (i: number, step: MacroStep) => setSteps(macroSteps.value.map((s, n) => (n === i ? step : s)))
function moveStep(i: number, by: number) {
  const steps = [...macroSteps.value]
  steps.splice(i + by, 0, ...steps.splice(i, 1))
  setSteps(steps)
}

// ---------- Entradas externas ----------
const learning = ref<InputControl>()
const capturing = ref<InputControl>()
watch(() => props.widget.id, () => (learning.value = capturing.value = undefined))

const inputOf = (control: InputControl): WidgetInput => props.widget.inputs.find((i) => i.control === control) ?? { control, midi: null, key: null }
function setInput(input: WidgetInput) {
  save({ inputs: [...props.widget.inputs.filter((i) => i.control !== input.control), input].filter((i) => i.midi || i.key) })
}

// Simulado: o primeiro controle mexido no controlador vira a entrada (o MIDI de verdade é da F12)
function learn(control: InputControl) {
  if (learning.value === control) return (learning.value = undefined)
  learning.value = control
  setTimeout(() => {
    const device = state.value.midiDevices.find((d) => d.connected)
    if (learning.value !== control || !device) return
    learning.value = undefined
    const fader = ['slider', 'animation', 'xyPad'].includes(props.widget.type)
    const midi = { device: device.id, channel: 1, message: fader ? 'controlChange' : 'note', number: 20 + Math.floor(Math.random() * 80), feedback: device.output } as const
    setInput({ ...inputOf(control), midi })
  }, 1500)
}

const midiText = (m: MidiControl) =>
  `${state.value.midiDevices.find((d) => d.id === m.device)?.name ?? '?'} · ${t(`vc.midi.${m.message}`, { n: m.number })} · ${t('vc.midi.channel', { n: m.channel })}`

// Teclas do Seletor: o item (id do grupo ou da seleção, `all` ou `clear`) que espera a próxima tecla
const capturingItem = ref<string>()
watch(() => props.widget.id, () => (capturingItem.value = undefined))
const itemKey = (item: string) => (props.widget.type === 'selector' ? props.widget.keys.find((k) => k.item === item)?.key : undefined)
function setItemKey(item: string, key: string | null) {
  if (props.widget.type !== 'selector') return
  save({ keys: [...props.widget.keys.filter((k) => k.item !== item), ...(key ? [{ item, key }] : [])] })
}
// As linhas das teclas: os grupos e as seleções do Seletor, e Todos e Limpar quando o botão existe
const keyRows = computed(() => {
  const w = props.widget
  if (w.type !== 'selector') return []
  return [
    ...w.groups.flatMap((id) => project.value.fixtureGroups.filter((g) => g.id === id).map((g) => ({ id, name: g.name }))),
    ...w.selections.flatMap((id) => project.value.selections.filter((s) => s.id === id).map((s) => ({ id, name: s.name }))),
    ...(w.all ? [{ id: 'all', name: t('vc.selectAll') }] : []),
    ...(w.clear ? [{ id: 'clear', name: t('vc.selectNone') }] : []),
    ...(w.splits ? ['first', 'second', 'odd', 'even', 'wings', 'one', 'previous', 'next', 'all'].map((m) => ({ id: `split:${m}`, name: t(`vc.split.${m}`) })) : []),
  ]
})

// As teclas apertadas viram a entrada: uma só, ou uma sequência (cada tecla até KEY_SEQUENCE_MS depois da
// anterior; passado esse tempo sem teclas, vale o que foi digitado). Esc cancela.
// Teclas reservadas (seção master e zoom) não podem ser usadas: avisa e continua esperando outra
const reserved = ref('')
const typed = ref<string[]>([])
let finish: ReturnType<typeof setTimeout> | undefined
const typedText = computed(() => (typed.value.length ? `${keyLabel(typed.value.join(' '))} …` : t('vc.pressKey')))
watch([capturing, capturingItem], () => {
  reserved.value = ''
  typed.value = []
  clearTimeout(finish)
})
function onKey(e: KeyboardEvent) {
  if ((!capturing.value && !capturingItem.value) || ['Control', 'Shift', 'Alt', 'Meta'].includes(e.key)) return
  e.preventDefault()
  e.stopPropagation()
  if (isReservedKey(e)) return (reserved.value = keyCombo(e))
  clearTimeout(finish)
  if (e.key === 'Escape') return (capturing.value = capturingItem.value = undefined)
  typed.value.push(keyCombo(e))
  finish = setTimeout(() => {
    const key = typed.value.join(' ')
    if (capturingItem.value) setItemKey(capturingItem.value, key)
    else if (capturing.value) setInput({ ...inputOf(capturing.value), key })
    capturing.value = capturingItem.value = undefined
  }, KEY_SEQUENCE_MS)
}
onMounted(() => window.addEventListener('keydown', onKey, true))
onBeforeUnmount(() => {
  window.removeEventListener('keydown', onKey, true)
  clearTimeout(finish)
})
</script>

<template>
  <div class="flex flex-col gap-3">
    <div class="flex items-center gap-2">
      <component :is="kind.icon" :size="18" class="text-fg-muted" />
      <h2 class="heading">{{ t(`vc.kind.${widget.type}`) }}</h2>
    </div>

    <label class="field-label">
      {{ t('vc.caption') }}
      <input class="field" :value="widget.caption" @input="save({ caption: ($event.target as HTMLInputElement).value })">
    </label>
    <RlSwitch v-if="!['button', 'macro', 'label'].includes(widget.type)" :model-value="widget.title" @update:model-value="(title: boolean) => save({ title })">{{ t('vc.showTitle') }}</RlSwitch>
    <ConsoleColor v-if="widget.type !== 'frame'" :label="t('vc.colorLabel')" :model-value="widget.color" @update:model-value="(color) => save({ color })" />

    <!-- Botão -->
    <template v-if="widget.type === 'button'">
      <label class="field-label">
        {{ t('vc.action') }}
        <select class="field" :value="widget.action" @change="save({ action: pick($event) })">
          <option v-for="a in ['function', 'exclusive', 'previous', 'pause', 'blackout', 'blackoutFixtures', 'stopAll']" :key="a" :value="a">{{ t(`vc.actions.${a}`) }}</option>
        </select>
      </label>
      <!-- O Voltar anda só chasers e sequências -->
      <label v-if="['function', 'exclusive', 'previous', 'pause'].includes(widget.action)" class="field-label">
        {{ widget.action === 'previous' ? t('vc.chaser') : t('vc.function') }}
        <select class="field" :value="widget.function ?? ''" @change="save({ function: pick($event) })">
          <option value="">—</option>
          <optgroup v-for="type in widget.action === 'previous' ? (['chaser', 'sequence'] as const) : functionTypes" :key="type" :label="t(`functionTypePlural.${type}`)">
            <option v-for="f in functionsOf(type)" :key="f.id" :value="f.id">{{ f.name }}</option>
          </optgroup>
        </select>
      </label>
      <div v-if="widget.action === 'blackoutFixtures'" class="field-label">
        {{ t('vc.blackoutTargets') }}
        <RlButton size="sm" @press="picking = true">{{ t('vc.pickTargets', { n: widget.groups.length + widget.fixtures.length }) }}</RlButton>
      </div>
      <!-- As ações de um toque não têm modo; o Pausar acende enquanto a criação está pausada -->
      <!-- Alvo (VC-7): liga a criação só nas fixtures da seleção do Seletor -->
      <template v-if="widget.action === 'function'">
        <RlSwitch :model-value="widget.target" @update:model-value="(target: boolean) => save({ target })">
          <span class="flex items-center gap-1">{{ t('vc.target') }} <RlInfo :text="t('vc.info.target')" /></span>
        </RlSwitch>
        <div v-if="widget.target" class="field-label">
          {{ t('vc.category') }}
          <RlSegmented
            :model-value="widget.category ?? 'none'"
            :options="(['none', 'color', 'effect', 'animation'] as const).map((value) => ({ value, label: t(`vc.categories.${value}`) }))"
            @update:model-value="(c) => save({ category: c === 'none' ? null : c })"
          />
        </div>
      </template>
      <div v-if="!['stopAll', 'previous', 'pause'].includes(widget.action) && !(widget.action === 'function' && widget.target)" class="field-label">
        <span class="flex items-center gap-1">{{ t('vc.mode') }} <RlInfo :text="t('vc.info.buttonMode')" /></span>
        <RlSegmented :model-value="widget.mode" :options="(['toggle', 'flash', 'temp'] as const).map((value) => ({ value, label: t(`vc.modes.${value}`) }))" @update:model-value="(mode) => save({ mode })" />
      </div>
      <div v-if="!['stopAll', 'previous'].includes(widget.action)" class="field-label">
        {{ t('vc.highlight') }}
        <RlSegmented :model-value="widget.highlight" :options="[{ value: 'face', label: t('vc.highlights.face') }, { value: 'led', label: t('vc.highlights.led') }]" @update:model-value="(highlight) => save({ highlight })" />
      </div>
    </template>

    <!-- Macro (E6): os passos, executados em ordem -->
    <div v-else-if="widget.type === 'macro'" class="field-label">
      <span class="flex items-center gap-1">{{ t('vc.macroSteps') }} <RlInfo :text="t('vc.info.macro')" /></span>
      <div v-for="(s, i) in widget.steps" :key="i" class="sunken flex flex-col gap-1.5 rounded-md p-2">
        <div class="flex items-center gap-1">
          <span class="w-5 shrink-0 font-lcd text-sm">{{ i + 1 }}</span>
          <select class="field min-w-0 flex-1" :value="s.type" @change="setStep(i, newStep(pick($event) as MacroStep['type']))">
            <option v-for="type in stepTypes" :key="type" :value="type">{{ t(`vc.macroStep.${type}`) }}</option>
          </select>
          <RlButton size="sm" :disabled="i === 0" :aria-label="t('vc.moveUp')" @press="moveStep(i, -1)"><ChevronUp :size="14" /></RlButton>
          <RlButton size="sm" :disabled="i === widget.steps.length - 1" :aria-label="t('vc.moveDown')" @press="moveStep(i, 1)"><ChevronDown :size="14" /></RlButton>
          <RlButton size="sm" :aria-label="t('common.remove')" @press="setSteps(widget.steps.filter((_, n) => n !== i))"><X :size="14" /></RlButton>
        </div>
        <select v-if="s.type === 'start' || s.type === 'stop'" class="field" :value="s.function" @change="setStep(i, { ...s, function: pick($event) ?? '' })">
          <optgroup v-for="type in functionTypes" :key="type" :label="t(`functionTypePlural.${type}`)">
            <option v-for="f in functionsOf(type)" :key="f.id" :value="f.id">{{ f.name }}</option>
          </optgroup>
        </select>
        <select v-else-if="s.type === 'go'" class="field" :value="s.chaser" @change="setStep(i, { ...s, chaser: pick($event) ?? '' })">
          <option v-for="f in chasers" :key="f.id" :value="f.id">{{ f.name }}</option>
        </select>
        <RlSegmented
          v-else-if="s.type === 'blackout'"
          :model-value="s.on ? 'on' : 'off'"
          :options="[{ value: 'on', label: t('vc.blackoutOn') }, { value: 'off', label: t('vc.blackoutOff') }]"
          @update:model-value="(v) => setStep(i, { ...s, on: v === 'on' })"
        />
        <span v-else-if="s.type === 'bpm'" class="flex items-center gap-2">
          <input
            type="number"
            min="30"
            max="300"
            class="field flex-1"
            :value="s.bpm"
            @change="setStep(i, { ...s, bpm: Math.min(300, Math.max(30, Math.round(Number(($event.target as HTMLInputElement).value) || 120))) })"
          >
          <span class="text-xs text-fg-dim">BPM</span>
        </span>
        <RlTimeField v-else-if="s.type === 'wait'" :model-value="s.ms" @update:model-value="(ms: number) => setStep(i, { ...s, ms })" />
      </div>
      <RlButton size="sm" @press="setSteps([...widget.steps, newStep('start')])">
        <Plus :size="14" />
        {{ t('vc.addStep') }}
      </RlButton>
    </div>

    <!-- Slider -->
    <template v-else-if="widget.type === 'slider'">
      <div class="field-label">
        <span class="flex items-center gap-1">{{ t('vc.mode') }} <RlInfo :text="t('vc.info.sliderMode')" /></span>
        <RlSegmented :model-value="widget.mode" :options="(['level', 'playback', 'submaster', 'rate'] as const).map((value) => ({ value, label: t(`vc.sliderModes.${value}`) }))" @update:model-value="(mode) => save({ mode })" />
      </div>
      <div v-if="widget.mode === 'rate'" class="field-label">
        {{ t('vc.functions', { n: widget.functions.length }) }}
        <div class="flex flex-wrap gap-1.5">
          <RlButton
            v-for="f in project.functions"
            :key="f.id"
            size="sm"
            highlight="led"
            :model-value="widget.functions.includes(f.id)"
            @press="save({ functions: toggled(widget.functions, f.id) })"
          >
            <component :is="functionIcons[f.type]" :size="14" />
            {{ f.name }}
          </RlButton>
        </div>
      </div>
      <div v-if="widget.mode === 'level'" class="field-label">
        {{ t('groups.channelGroups') }}
        <div class="flex flex-wrap gap-1.5">
          <RlButton
            v-for="g in project.channelGroups"
            :key="g.id"
            size="sm"
            highlight="led"
            :model-value="widget.channelGroups.includes(g.id)"
            @press="save({ channelGroups: toggled(widget.channelGroups, g.id) })"
          >
            <component :is="channelGroupLook(project, g).icon" :size="14" />
            {{ g.name }}
          </RlButton>
        </div>
      </div>
      <label v-if="widget.mode === 'playback'" class="field-label">
        {{ t('vc.function') }}
        <select class="field" :value="widget.function ?? ''" @change="save({ function: pick($event) })">
          <option value="">—</option>
          <option v-for="f in project.functions" :key="f.id" :value="f.id">{{ f.name }}</option>
        </select>
      </label>
      <div class="field-label">
        {{ t('vc.look') }}
        <RlSegmented :model-value="widget.knob ? 'knob' : 'fader'" :options="[{ value: 'fader', label: t('vc.fader') }, { value: 'knob', label: t('vc.knob') }]" @update:model-value="(v) => save({ knob: v === 'knob' })" />
      </div>
    </template>

    <label v-else-if="widget.type === 'cueList'" class="field-label">
      <span class="flex items-center gap-1">{{ t('vc.chaser') }} <RlInfo :text="t('vc.info.cueList')" /></span>
      <select class="field" :value="widget.chaser ?? ''" @change="save({ chaser: pick($event) })">
        <option value="">—</option>
        <option v-for="f in functionsOf('chaser', 'sequence')" :key="f.id" :value="f.id">{{ f.name }}</option>
      </select>
    </label>

    <!-- Seletor (VC-7): o que o aparelho pode escolher -->
    <template v-else-if="widget.type === 'selector'">
      <div class="field-label">
        <span class="flex items-center gap-1">{{ t('groups.fixtureGroups') }} <RlInfo :text="t('vc.info.selector')" /></span>
        <div class="flex flex-wrap gap-1.5">
          <RlButton
            v-for="{ group: g } in groupTree(project)"
            :key="g.id"
            size="sm"
            highlight="led"
            :model-value="widget.groups.includes(g.id)"
            @press="save({ groups: toggled(widget.groups, g.id) })"
          >
            {{ g.name }}
          </RlButton>
        </div>
      </div>
      <div v-if="project.selections.length" class="field-label">
        {{ t('vc.selections') }}
        <div class="flex flex-wrap gap-1.5">
          <RlButton
            v-for="s in project.selections"
            :key="s.id"
            size="sm"
            highlight="led"
            :model-value="widget.selections.includes(s.id)"
            @press="save({ selections: toggled(widget.selections, s.id) })"
          >
            {{ s.name }}
          </RlButton>
        </div>
      </div>
      <div class="field-label">
        <span class="flex items-center gap-1">{{ t('vc.split.title') }} <RlInfo :text="t('vc.info.split')" /></span>
        <RlButton size="sm" highlight="led" :model-value="widget.splits" @press="save({ splits: !widget.splits })">{{ t('vc.split.bar') }}</RlButton>
        <RlButton size="sm" highlight="led" :model-value="widget.all" @press="save({ all: !widget.all })">{{ t('vc.showAll') }}</RlButton>
        <RlButton size="sm" highlight="led" :model-value="widget.clear" @press="save({ clear: !widget.clear })">{{ t('vc.showClear') }}</RlButton>
      </div>
      <!-- Uma tecla por item; Todos e Limpar só aparecem com o botão ligado -->
      <div class="field-label">
        <span class="flex items-center gap-1">{{ t('vc.selectorKeys') }} <RlInfo :text="t('vc.info.selectorKeys')" /></span>
        <div v-for="row in keyRows" :key="row.id" class="flex items-center gap-2">
          <span class="w-24 shrink-0 truncate text-xs">{{ row.name }}</span>
          <span class="min-w-0 flex-1 truncate font-lcd text-sm" :class="{ 'text-fg-dim': !itemKey(row.id) }">
            {{ capturingItem === row.id ? typedText : itemKey(row.id) ?? t('vc.noKey') }}
          </span>
          <RlButton size="sm" highlight="led" color="var(--color-flash)" :model-value="capturingItem === row.id" :aria-label="t('vc.key')" @press="capturingItem = capturingItem === row.id ? undefined : row.id">
            <Keyboard :size="14" />
          </RlButton>
          <RlButton v-if="itemKey(row.id)" size="sm" :aria-label="t('common.remove')" @press="setItemKey(row.id, null)"><X :size="14" /></RlButton>
        </div>
        <p v-if="capturingItem && reserved" class="text-xs text-flash">{{ t('vc.reservedKey', { key: reserved }) }}</p>
      </div>
    </template>

    <div v-else-if="widget.type === 'xyPad'" class="field-label">
      {{ t('vc.movers') }}
      <RlButton size="sm" @press="picking = true">{{ t('vc.pickTargets', { n: widget.groups.length + widget.fixtures.length }) }}</RlButton>
    </div>

    <!-- Speed dial (VC-3) -->
    <template v-else-if="widget.type === 'speedDial'">
      <RlSwitch :model-value="widget.beats" @update:model-value="(beats: boolean) => save({ beats })">{{ t('vc.followBpm') }}</RlSwitch>
      <div class="field-label">
        <span class="flex items-center gap-1">{{ t('vc.times') }} <RlInfo :text="t('vc.info.speedDial')" /></span>
        <div class="flex flex-wrap gap-x-4 gap-y-1">
          <RlSwitch v-for="time in ['fadeIn', 'fadeOut', 'hold'] as const" :key="time" :model-value="widget[time]" @update:model-value="(on: boolean) => save({ [time]: on })">
            {{ t(`vc.time.${time}`) }}
          </RlSwitch>
        </div>
      </div>
      <div class="field-label">
        {{ t('vc.functions', { n: widget.functions.length }) }}
        <div class="flex flex-wrap gap-1.5">
          <RlButton
            v-for="f in project.functions"
            :key="f.id"
            size="sm"
            highlight="led"
            :model-value="widget.functions.includes(f.id)"
            @press="save({ functions: toggled(widget.functions, f.id) })"
          >
            <component :is="functionIcons[f.type]" :size="14" />
            {{ f.name }}
          </RlButton>
        </div>
      </div>
    </template>

    <RlSwitch v-else-if="widget.type === 'frame'" :model-value="widget.solo" @update:model-value="(solo: boolean) => save({ solo })">
      <span class="flex items-center gap-1">{{ t('vc.solo') }} <RlInfo :text="t('vc.info.solo')" /></span>
    </RlSwitch>

    <template v-else-if="widget.type === 'clock'">
      <div class="field-label">
        {{ t('vc.mode') }}
        <RlSegmented :model-value="widget.mode" :options="(['clock', 'stopwatch', 'countdown'] as const).map((value) => ({ value, label: t(`vc.clockModes.${value}`) }))" @update:model-value="(mode) => save({ mode })" />
      </div>
      <label v-if="widget.mode === 'countdown'" class="field-label">
        {{ t('vc.countdown') }}
        <RlTimeField :model-value="widget.countdownS * 1000" @update:model-value="(ms: number) => save({ countdownS: Math.round(ms / 1000) })" />
      </label>
    </template>

    <label v-else-if="widget.type === 'animation'" class="field-label">
      {{ t('functionType.rgbMatrix') }}
      <select class="field" :value="widget.matrix ?? ''" @change="save({ matrix: pick($event) })">
        <option value="">—</option>
        <option v-for="f in functionsOf('rgbMatrix')" :key="f.id" :value="f.id">{{ f.name }}</option>
      </select>
    </label>

    <template v-else-if="widget.type === 'label'">
      <ConsoleColor :label="t('vc.background')" none :model-value="widget.background" @update:model-value="(background) => save({ background })" />
      <ConsoleColor :label="t('vc.border')" none :model-value="widget.border" @update:model-value="(border) => save({ border })" />
    </template>

    <!-- Ícone do botão e da macro -->
    <template v-if="widget.type === 'button' || widget.type === 'macro'">
      <div class="field-label">
        {{ t('vc.icon') }}
        <div class="flex min-h-11 items-center gap-3">
          <RlSwitch :model-value="!!widget.icon" @update:model-value="(on: boolean) => save({ icon: on ? defaultIcon() : null })">
            {{ t('vc.showIcon') }}
          </RlSwitch>
          <RlButton v-if="widget.icon" size="sm" class="ml-auto" @press="pickingIcon = true">
            <component :is="icon(widget.icon)" v-if="icon(widget.icon)" :size="16" />
            {{ t('vc.changeIcon') }}
          </RlButton>
        </div>
      </div>
      <ConsoleIcons v-model:open="pickingIcon" :model-value="widget.icon" @update:model-value="(name) => save({ icon: name })" />
    </template>

    <!-- Entradas externas (VC-1) -->
    <section v-if="kind.inputs.length" class="divider flex flex-col gap-3">
      <h3 class="heading flex items-center gap-1">{{ t('vc.inputs') }} <RlInfo :text="t('vc.info.inputs')" /></h3>
      <div v-for="control in kind.inputs" :key="control" class="flex flex-col gap-1.5">
        <span v-if="kind.inputs.length > 1" class="text-xs font-bold text-fg-muted uppercase">
          {{ t(`vc.control.${widget.type === 'cueList' && control === 'main' ? 'go' : control}`) }}
        </span>
        <div class="flex items-center gap-2">
          <span class="min-w-0 flex-1 text-xs" :class="{ 'text-fg-dim': !inputOf(control).midi }">
            {{ learning === control ? t('vc.learning') : inputOf(control).midi ? midiText(inputOf(control).midi!) : t('vc.noMidi') }}
          </span>
          <RlButton size="sm" highlight="led" color="var(--color-flash)" :model-value="learning === control" @press="learn(control)">
            <Wand :size="14" />
            {{ t('vc.learn') }}
          </RlButton>
          <RlButton v-if="inputOf(control).midi" size="sm" :aria-label="t('common.remove')" @press="setInput({ ...inputOf(control), midi: null })"><X :size="14" /></RlButton>
        </div>
        <RlSwitch
          v-if="inputOf(control).midi && state.midiDevices.find((d) => d.id === inputOf(control).midi!.device)?.output"
          :model-value="inputOf(control).midi!.feedback"
          @update:model-value="(feedback: boolean) => setInput({ ...inputOf(control), midi: { ...inputOf(control).midi!, feedback } })"
        >
          {{ t('vc.feedback') }}
        </RlSwitch>
        <div v-if="kind.keys" class="flex items-center gap-2">
          <span class="min-w-0 flex-1 truncate font-lcd text-sm" :class="{ 'text-fg-dim': !inputOf(control).key }">
            {{ capturing === control ? typedText : inputOf(control).key ?? t('vc.noKey') }}
          </span>
          <RlButton size="sm" highlight="led" color="var(--color-flash)" :model-value="capturing === control" @press="capturing = capturing === control ? undefined : control">
            <Keyboard :size="14" />
            {{ t('vc.key') }}
          </RlButton>
          <RlButton v-if="inputOf(control).key" size="sm" :aria-label="t('common.remove')" @press="setInput({ ...inputOf(control), key: null })"><X :size="14" /></RlButton>
        </div>
        <p v-if="capturing === control && reserved" class="text-xs text-flash">{{ t('vc.reservedKey', { key: reserved }) }}</p>
      </div>
    </section>

    <!-- Alvos do blackout de fixtures e do XY Pad -->
    <FixturesPicker
      v-if="widget.type === 'xyPad' || widget.type === 'button'"
      v-model:open="picking"
      :title="widget.type === 'xyPad' ? t('vc.movers') : t('vc.blackoutTargets')"
      :groups="groupTree(project)"
      :fixtures="widget.type === 'xyPad' ? movers : project.fixtures"
      :is-group-on="(id: string) => 'groups' in widget && widget.groups.includes(id)"
      :is-fixture-on="(id: string) => 'fixtures' in widget && widget.fixtures.includes(id)"
      clearable
      @group="(id: string) => 'groups' in widget && save({ groups: toggled(widget.groups, id) })"
      @fixture="(id: string) => 'fixtures' in widget && save({ fixtures: toggled(widget.fixtures, id) })"
      @clear="save({ groups: [], fixtures: [] })"
    />
  </div>
</template>

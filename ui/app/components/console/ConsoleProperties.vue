<script setup lang="ts">
import { Keyboard, Wand, X } from 'lucide-vue-next'
import type { FunctionType } from '~/types/generated/FunctionType'
import type { InputControl } from '~/types/generated/InputControl'
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
function defaultIcon() {
  const w = props.widget
  if (w.type !== 'button') return null
  if (w.action === 'stopAll') return 'OctagonX'
  if (w.action !== 'function' && w.action !== 'exclusive') return 'LightbulbOff'
  const type = project.value.functions.find((f) => f.id === w.function)?.type ?? 'scene'
  return { scene: 'Camera', chaser: 'Repeat', sequence: 'Film', collection: 'Layers', efx: 'Orbit', rgbMatrix: 'LayoutGrid' }[type]
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

// A próxima tecla apertada vira a entrada; Esc cancela
// Teclas reservadas (seção master e zoom) não podem ser usadas: avisa e continua esperando outra
const reserved = ref('')
watch(capturing, () => (reserved.value = ''))
function onKey(e: KeyboardEvent) {
  if (!capturing.value || ['Control', 'Shift', 'Alt', 'Meta'].includes(e.key)) return
  e.preventDefault()
  e.stopPropagation()
  if (isReservedKey(e)) return (reserved.value = keyCombo(e))
  if (e.key !== 'Escape') setInput({ ...inputOf(capturing.value), key: keyCombo(e) })
  capturing.value = undefined
}
onMounted(() => window.addEventListener('keydown', onKey, true))
onBeforeUnmount(() => window.removeEventListener('keydown', onKey, true))
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
    <ConsoleColor v-if="widget.type !== 'frame'" :label="t('vc.colorLabel')" :model-value="widget.color" @update:model-value="(color) => save({ color })" />

    <!-- Botão -->
    <template v-if="widget.type === 'button'">
      <label class="field-label">
        {{ t('vc.action') }}
        <select class="field" :value="widget.action" @change="save({ action: pick($event) })">
          <option v-for="a in ['function', 'exclusive', 'blackout', 'blackoutFixtures', 'stopAll']" :key="a" :value="a">{{ t(`vc.actions.${a}`) }}</option>
        </select>
      </label>
      <label v-if="widget.action === 'function' || widget.action === 'exclusive'" class="field-label">
        {{ t('vc.function') }}
        <select class="field" :value="widget.function ?? ''" @change="save({ function: pick($event) })">
          <option value="">—</option>
          <optgroup v-for="type in functionTypes" :key="type" :label="t(`functionTypePlural.${type}`)">
            <option v-for="f in functionsOf(type)" :key="f.id" :value="f.id">{{ f.name }}</option>
          </optgroup>
        </select>
      </label>
      <div v-if="widget.action === 'blackoutFixtures'" class="field-label">
        {{ t('vc.blackoutTargets') }}
        <div class="flex flex-wrap gap-1.5">
          <RlButton
            v-for="g in project.fixtureGroups"
            :key="g.id"
            size="sm"
            highlight="led"
            :model-value="widget.groups.includes(g.id)"
            @press="save({ groups: toggled(widget.groups, g.id) })"
          >
            {{ g.name }}
          </RlButton>
          <RlButton
            v-for="f in project.fixtures"
            :key="f.id"
            size="sm"
            highlight="led"
            :color="f.color"
            :model-value="widget.fixtures.includes(f.id)"
            @press="save({ fixtures: toggled(widget.fixtures, f.id) })"
          >
            {{ f.name }}
          </RlButton>
        </div>
      </div>
      <template v-if="widget.action !== 'stopAll'">
        <div class="field-label">
          <span class="flex items-center gap-1">{{ t('vc.mode') }} <RlInfo :text="t('vc.info.buttonMode')" /></span>
          <RlSegmented :model-value="widget.mode" :options="(['toggle', 'flash', 'temp'] as const).map((value) => ({ value, label: t(`vc.modes.${value}`) }))" @update:model-value="(mode) => save({ mode })" />
        </div>
        <div class="field-label">
          {{ t('vc.highlight') }}
          <RlSegmented :model-value="widget.highlight" :options="[{ value: 'face', label: t('vc.highlights.face') }, { value: 'led', label: t('vc.highlights.led') }]" @update:model-value="(highlight) => save({ highlight })" />
        </div>
      </template>
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
      {{ t('vc.chaser') }}
      <select class="field" :value="widget.chaser ?? ''" @change="save({ chaser: pick($event) })">
        <option value="">—</option>
        <option v-for="f in functionsOf('chaser', 'sequence')" :key="f.id" :value="f.id">{{ f.name }}</option>
      </select>
    </label>

    <div v-else-if="widget.type === 'xyPad'" class="field-label">
      {{ t('vc.movers') }}
      <div class="flex flex-wrap gap-1.5">
        <RlButton
          v-for="f in movers"
          :key="f.id"
          size="sm"
          highlight="led"
          :color="f.color"
          :model-value="widget.fixtures.includes(f.id)"
          @press="save({ fixtures: toggled(widget.fixtures, f.id) })"
        >
          {{ f.name }}
        </RlButton>
      </div>
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

    <!-- Entradas externas (VC-1) -->
    <section v-if="kind.inputs.length" class="divider flex flex-col gap-3">
      <h3 class="heading flex items-center gap-1">{{ t('vc.inputs') }} <RlInfo :text="t('vc.info.inputs')" /></h3>
      <div v-for="control in kind.inputs" :key="control" class="flex flex-col gap-1.5">
        <span v-if="kind.inputs.length > 1" class="text-xs font-bold text-fg-muted uppercase">{{ t(`vc.control.${control}`) }}</span>
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
            {{ capturing === control ? t('vc.pressKey') : inputOf(control).key ?? t('vc.noKey') }}
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
  </div>
</template>

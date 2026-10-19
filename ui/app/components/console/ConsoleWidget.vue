<script setup lang="ts">
import { ChevronLeft, ChevronRight, Play, RotateCcw, Square } from 'lucide-vue-next'
import type { VcWidget } from '~/types/generated/VcWidget'

// Elemento do console em operação. A moldura é desenhada pelo ConsoleCanvas, por baixo dos filhos.
const props = defineProps<{ widget: VcWidget }>()
const { t } = useI18n()
const { state, send } = useEngine()
const { format } = useValueFormat()
const vc = useConsole()
const project = computed(() => state.value.project)
const fn = (id: string | null) => project.value.functions.find((f) => f.id === id)
// Só o relógio precisa do tempo correndo
const now = props.widget.type === 'clock' ? useClock(200) : ref(0)

const setButton = (on: boolean) => props.widget.type === 'button' && vc.setButton(props.widget, on)
// O botão só mostra ícone quando o usuário liga a opção (VC-1)
const { icon } = useIconLibrary()
const buttonIcon = computed(() => (props.widget.type === 'button' ? icon(props.widget.icon) : undefined))

// Nível 0–1 do slider e do animation (o animation começa com a intensidade cheia)
const level = computed({
  get: () => (vc.value(props.widget.id)?.[0] ?? (props.widget.type === 'animation' ? 255 : 0)) / 255,
  set: (v: number) => vc.setValue(props.widget.id, [Math.round(v * 255)]),
})
const xy = computed({
  get: () => {
    const [x = 128, y = 128] = vc.value(props.widget.id) ?? []
    return { x: x / 255, y: y / 255 }
  },
  set: (v: { x: number; y: number }) => vc.setValue(props.widget.id, [Math.round(v.x * 255), Math.round(v.y * 255)]),
})

// Cue list
const chaser = computed(() => (props.widget.type === 'cueList' ? vc.chaserOf(props.widget) : undefined))
const step = computed(() => (props.widget.type === 'cueList' ? vc.currentStep(props.widget) : -1))
const playing = computed(() => props.widget.type === 'cueList' && vc.running(props.widget.chaser))

// Speed dial (VC-3), sem valor, valem os tempos gravados. Em segundos, o knob vai de 0 a 20 s, com mais
// precisão nos tempos curtos; seguindo o BPM, ele escolhe frações e múltiplos da batida (em milésimos)
const MAX_MS = 20000
const BEATS = [0, 125, 250, 500, 1000, 2000, 4000, 8000, 16000]
const times = computed(() => vc.value(props.widget.id))
const dials = computed(() => {
  const w = props.widget
  return w.type === 'speedDial' ? (['fadeIn', 'fadeOut', 'hold'] as const).map((time, i) => ({ time, i })).filter(({ time }) => w[time]) : []
})
const inBeats = computed(() => props.widget.type === 'speedDial' && props.widget.beats)
const seconds = (ms: number) => `${(ms / 1000).toFixed(1)} s`
const beats = (v: number) => `${v && v < 1000 ? `1/${1000 / v}` : v / 1000} bt`
const knob = (v: number) => (inBeats.value ? BEATS.findLastIndex((b) => b <= v) / (BEATS.length - 1) : Math.sqrt(v / MAX_MS))
function setTime(i: number, v: number) {
  const next = [...(times.value ?? [0, 0, 0])]
  next[i] = inBeats.value ? BEATS[Math.round(v * (BEATS.length - 1))]! : Math.round((v * v * MAX_MS) / 50) * 50
  vc.setValue(props.widget.id, next)
}

// Relógio: cronômetro e contagem regressiva correm neste aparelho
const timer = reactive({ started: 0, elapsed: 0 })
const elapsed = computed(() => timer.elapsed + (timer.started ? now.value - timer.started : 0))
function startStop() {
  if (timer.started) timer.elapsed += Date.now() - timer.started
  timer.started = timer.started ? 0 : Date.now()
}
const clockText = computed(() => {
  const w = props.widget
  if (w.type !== 'clock') return ''
  if (w.mode === 'clock') return new Date(now.value).toLocaleTimeString('pt-BR')
  const ms = w.mode === 'countdown' ? Math.max(0, w.countdownS * 1000 - elapsed.value) : elapsed.value
  return new Date(ms).toISOString().slice(11, 21)
})

// Animation: muda o próprio RGB Matrix
const matrix = computed(() => {
  const f = props.widget.type === 'animation' ? fn(props.widget.matrix) : undefined
  return f?.type === 'rgbMatrix' ? f : undefined
})
const editingColor = ref<number>()
function setColor(i: number, color: string) {
  const m = matrix.value!
  send({ op: 'saveFunction', function: { ...m, colors: m.colors.map((c, n) => (n === i ? color : c)) } })
}
</script>

<template>
  <RlButton
    v-if="widget.type === 'button'"
    class="h-full w-full flex-col gap-1! px-1! text-[11px]!"
    :mode="widget.action === 'stopAll' ? 'action' : widget.mode"
    :highlight="widget.highlight"
    :color="widget.color"
    :model-value="vc.active(widget)"
    @update:model-value="setButton"
    @press="widget.action === 'stopAll' && setButton(true)"
  >
    <component :is="buttonIcon" v-if="buttonIcon" :size="18" class="shrink-0" />
    <span class="line-clamp-2 whitespace-normal text-center leading-tight wrap-anywhere">{{ widget.caption }}</span>
  </RlButton>

  <div
    v-else-if="widget.type === 'label'"
    class="flex h-full items-center justify-center rounded-lg px-2 text-center text-sm font-bold tracking-widest uppercase"
    :style="{ color: widget.color, background: widget.background ?? undefined, border: widget.border ? `2px solid ${widget.border}` : undefined }"
  >
    {{ widget.caption }}
  </div>

  <div v-else class="box">
    <header class="caption">
      <i :style="{ background: widget.color }" />
      <span class="truncate">{{ widget.caption }}</span>
    </header>

    <template v-if="widget.type === 'slider'">
      <RlKnob v-if="widget.knob" v-model="level" class="mx-auto" :color="widget.color" />
      <RlFader v-else v-model="level" class="min-h-0 flex-1" :cap="widget.mode === 'submaster' ? 'red' : 'silver'" />
      <RlLcd class="text-center text-xs">{{ format(level) }}</RlLcd>
    </template>

    <RlXYPad v-else-if="widget.type === 'xyPad'" v-model="xy" />

    <template v-else-if="widget.type === 'cueList'">
      <div class="sunken min-h-0 flex-1 overflow-y-auto rounded-md">
        <button
          v-for="(s, i) in chaser?.steps ?? []"
          :key="i"
          type="button"
          class="cue"
          :class="{ current: i === step }"
          @click="vc.step(widget, { go: i })"
        >
          <span class="font-lcd">{{ i + 1 }}</span>
          <span class="flex-1 truncate">{{ s.note || fn(s.function)?.name }}</span>
          <span class="font-lcd text-fg-muted">{{ seconds(s.timing.fadeInMs + s.timing.holdMs) }}</span>
        </button>
        <p v-if="!chaser" class="p-2 text-xs text-fg-muted">{{ t('vc.noChaser') }}</p>
      </div>
      <div class="grid grid-cols-3 gap-1">
        <RlButton size="sm" :aria-label="t('vc.control.previous')" @press="vc.input(widget, 'previous', true)"><ChevronLeft :size="18" /></RlButton>
        <RlButton
          size="sm"
          highlight="led"
          color="var(--color-on)"
          :model-value="playing"
          :aria-label="t('vc.control.main')"
          @press="vc.play(widget, !playing)"
        >
          <component :is="playing ? Square : Play" :size="16" />
        </RlButton>
        <RlButton size="sm" :aria-label="t('vc.control.next')" @press="vc.input(widget, 'next', true)"><ChevronRight :size="18" /></RlButton>
      </div>
    </template>

    <template v-else-if="widget.type === 'speedDial'">
      <div class="flex min-h-0 flex-1 items-center justify-around gap-1">
        <div v-for="d in dials" :key="d.time" class="flex flex-col items-center gap-1">
          <RlKnob :model-value="knob(times?.[d.i] ?? 0)" :size="44" :color="widget.color" @update:model-value="(v: number) => setTime(d.i, v)" />
          <RlLcd class="text-xs" :class="{ 'text-fg-dim!': !times }">
            {{ times ? (widget.beats ? beats(times[d.i] ?? 0) : seconds(times[d.i] ?? 0)) : t('vc.recorded') }}
          </RlLcd>
          <span class="text-[10px] font-bold text-fg-muted uppercase">{{ t(`vc.time.${d.time}`) }}</span>
          <!-- Seguindo o BPM, o tempo de agora pelo BPM master -->
          <span v-if="widget.beats && times" class="text-[10px] text-fg-dim">
            {{ `${(((times[d.i] ?? 0) * 60) / state.master.bpm / 1000).toFixed(2)} s` }}
          </span>
        </div>
      </div>
      <div class="flex gap-1">
        <RlButton v-if="widget.hold || widget.beats" size="sm" class="flex-1" @press="vc.tap(widget)">
          {{ widget.beats ? t('vc.tapBpm') : t('vc.tap') }}
        </RlButton>
        <RlButton size="sm" class="flex-1" :model-value="!times" highlight="led" @press="vc.setValue(widget.id, [])">{{ t('vc.recorded') }}</RlButton>
      </div>
    </template>

    <template v-else-if="widget.type === 'clock'">
      <RlLcd class="flex flex-1 items-center justify-center text-xl">{{ clockText }}</RlLcd>
      <div v-if="widget.mode !== 'clock'" class="flex gap-1">
        <RlButton size="sm" class="flex-1" highlight="led" color="var(--color-on)" :model-value="!!timer.started" @press="startStop">
          <component :is="timer.started ? Square : Play" :size="14" />
        </RlButton>
        <RlButton size="sm" class="flex-1" :aria-label="t('vc.reset')" @press="Object.assign(timer, { elapsed: 0, started: timer.started && Date.now() })">
          <RotateCcw :size="14" />
        </RlButton>
      </div>
    </template>

    <template v-else-if="widget.type === 'animation'">
      <p v-if="!matrix" class="text-xs text-fg-muted">{{ t('vc.noMatrix') }}</p>
      <div v-else class="flex min-h-0 flex-1 gap-2">
        <RlFader v-model="level" class="min-h-0" :default="1" />
        <div class="flex min-w-0 flex-1 flex-col gap-2">
          <RlButton
            size="sm"
            highlight="led"
            color="var(--color-on)"
            :model-value="state.running.includes(matrix.id)"
            @press="send({ op: 'setFunctionRunning', id: matrix.id, running: !state.running.includes(matrix.id) })"
          >
            <Play :size="14" />
          </RlButton>
          <select class="field min-w-0" :value="matrix.algorithm" @change="send({ op: 'saveFunction', function: { ...matrix, algorithm: ($event.target as HTMLSelectElement).value } })">
            <option v-for="a in matrixAlgorithms" :key="a" :value="a">{{ t(`matrixAlgorithm.${a}`) }}</option>
          </select>
          <div class="flex gap-1">
            <button
              v-for="(c, i) in matrix.colors"
              :key="i"
              type="button"
              class="swatch"
              :style="{ background: c }"
              :aria-label="t('vc.color', { n: i + 1 })"
              @click="editingColor = i"
            />
          </div>
        </div>
      </div>
      <RlDialog :open="editingColor !== undefined" :title="t('vc.color', { n: (editingColor ?? 0) + 1 })" @update:open="editingColor = undefined">
        <RlColorPicker v-if="matrix && editingColor !== undefined" :model-value="matrix.colors[editingColor]!" @update:model-value="(c: string) => setColor(editingColor!, c)" />
      </RlDialog>
    </template>
  </div>
</template>

<style scoped>
.box {
  display: flex;
  flex-direction: column;
  gap: 4px;
  height: 100%;
  padding: 6px;
  overflow: hidden;
  border: 1px solid #000;
  border-radius: 8px;
  background: linear-gradient(#1b1d21, #15171a);
  box-shadow: inset 0 1px 0 rgb(255 255 255 / 0.05);
}

.caption {
  display: flex;
  flex-shrink: 0;
  align-items: center;
  gap: 6px;
  color: var(--color-fg-muted);
  font-size: 10px;
  font-weight: 700;
  letter-spacing: 0.08em;
  text-transform: uppercase;

  i {
    flex-shrink: 0;
    width: 8px;
    height: 8px;
    border-radius: 2px;
  }
}

.cue {
  display: flex;
  gap: 8px;
  width: 100%;
  min-height: 36px;
  align-items: center;
  padding: 0 8px;
  border-bottom: 1px solid rgb(255 255 255 / 0.05);
  font-size: 12px;
  text-align: left;
}

.current {
  background: color-mix(in srgb, var(--color-select) 30%, transparent);
  color: #fff;
}

.swatch {
  flex: 1;
  height: 36px;
  border: 1px solid #000;
  border-radius: 6px;
  box-shadow: inset 0 1px 0 rgb(255 255 255 / 0.35), 0 2px 3px rgb(0 0 0 / 0.6);
}
</style>

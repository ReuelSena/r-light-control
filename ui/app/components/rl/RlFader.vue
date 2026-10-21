<script setup lang="ts">
// Fader vertical de mesa (E9): arrasto relativo, botão FINO para precisão, toque duplo
// para voltar ao padrão e roda do mouse. Cada dedo controla o seu fader (multitoque).
const props = withDefaults(defineProps<{
  default?: number
  /** Nível de saída (0–1). Sem ele, o medidor não aparece. */
  meter?: number
  meterColor?: string
  cap?: 'silver' | 'red'
  /** Escala ao lado do trilho: o rótulo e a posição (0–1). Sem ela, vai de 0 a 100%. */
  marks?: { label: number | string; at: number }[]
  /** Passo da roda e das setas com o FINO, de 0 a 1 (0,1% do curso). */
  fineStep?: number
}>(), {
  default: 0,
  meter: undefined,
  meterColor: undefined,
  cap: 'silver',
  marks: () => [100, 75, 50, 25, 0].map((label) => ({ label, at: label / 100 })),
  fineStep: 0.001,
})

const model = defineModel<number>({ required: true })
// Otimista sob o dedo: o gesto não espera a volta do servidor
const value = useOptimistic(model)
const { t } = useI18n()
const fine = ref(false)
const track = ref<HTMLElement>()

// Curso total = altura do trilho; no modo fino o mesmo gesto anda 10× menos.
const drag = useDrag(
  (_, dy) => (value.value = clamp01(value.value - (dy / track.value!.clientHeight) * (fine.value ? 0.1 : 1))),
  () => (value.value = props.default),
)

// Com FINO, a roda e as setas andam `fineStep` por passo de 1%
const fineScale = () => (fine.value ? props.fineStep / 0.01 : 1)
const keySteps: Record<string, number> = { ArrowUp: 0.01, ArrowDown: -0.01, PageUp: 0.1, PageDown: -0.1 }

function key(e: KeyboardEvent) {
  const step = keySteps[e.key]
  if (step === undefined) return
  e.preventDefault()
  value.value = clamp01(value.value + step * fineScale())
}

// Roda do mouse: cada clique (100 px, ou 3 linhas no Firefox) anda 1%; com FINO, 0,1%.
// Proporcional ao deslocamento, para o trackpad não disparar.
function wheel(e: WheelEvent) {
  const clicks = e.deltaY / (e.deltaMode === 1 ? 3 : 100)
  value.value = clamp01(value.value - clicks * 0.01 * fineScale())
}
</script>

<template>
  <div class="rl-fader">
    <div class="body" @wheel.prevent="wheel">
      <div class="scale">
        <span v-for="mark in marks" :key="mark.label" :style="{ bottom: `${mark.at * 100}%` }">{{ mark.label }}</span>
      </div>
      <div
        ref="track"
        class="track"
        role="slider"
        tabindex="0"
        aria-valuemin="0"
        aria-valuemax="100"
        :aria-valuenow="Math.round(value * 100)"
        v-on="drag"
        @keydown="key"
      >
        <div class="slot sunken" />
        <div class="cap" :class="cap" :style="{ '--v': value }" />
      </div>
      <RlMeter v-if="meter !== undefined" :level="meter" :color="meterColor" />
    </div>
    <RlButton v-model="fine" class="fine" mode="toggle" size="sm" color="var(--color-flash)">
      {{ t('rl.fine') }}
    </RlButton>
  </div>
</template>

<style scoped>
.rl-fader {
  display: flex;
  flex-direction: column;
  gap: 6px;
  min-height: 0;
}

.body {
  flex: 1;
  display: flex;
  gap: 3px;
  min-height: 140px;
}

/* Régua alinhada ao curso do centro do cap: meia altura do cap (20 px) em cima e embaixo */
.scale {
  position: relative;
  width: 20px;
  margin: 20px 0;
  background: repeating-linear-gradient(to top, var(--color-fg-dim) 0 1px, transparent 1px 10%) right / 4px 100% no-repeat;
}

.scale span {
  position: absolute;
  right: 6px;
  transform: translateY(50%);
  color: var(--color-fg-dim);
  font-size: 9px;
  font-variant-numeric: tabular-nums;
}

.track {
  position: relative;
  flex: 1;
  min-width: 40px;
  container-type: size;
  touch-action: none;
  outline: none;
}

.track:focus-visible .cap {
  outline: 2px solid var(--color-select);
}

.slot {
  position: absolute;
  left: 50%;
  top: 20px;
  bottom: 20px;
  width: 6px;
  transform: translateX(-50%);
  border-radius: 3px;
}

.cap {
  position: absolute;
  left: 50%;
  top: 0;
  width: 34px;
  height: 40px;
  border-radius: 3px;
  /* Só transform (cqh = altura do trilho): o navegador não recalcula layout a cada valor */
  transform: translate(-50%, calc((1 - var(--v)) * (100cqh - 40px)));
  background:
    linear-gradient(transparent 46%, #151515 46% 54%, transparent 54%),
    repeating-linear-gradient(transparent 0 3px, rgb(0 0 0 / 0.22) 3px 4px),
    linear-gradient(#f6f7f8, #a2a7ac 42%, #d9dcdf 58%, #80858a);
  box-shadow: 0 4px 6px rgb(0 0 0 / 0.75), inset 0 1px 0 #fff;
  will-change: transform;
}

.cap.red {
  background:
    linear-gradient(transparent 46%, #300000 46% 54%, transparent 54%),
    repeating-linear-gradient(transparent 0 3px, rgb(0 0 0 / 0.2) 3px 4px),
    linear-gradient(#ff7070, #d00000 42%, #ff2020 58%, #800000);
  box-shadow: 0 4px 6px rgb(0 0 0 / 0.75), 0 0 12px rgb(255 0 0 / 0.4), inset 0 1px 0 #ffc0c0;
}

.fine {
  width: 100%;
}
</style>

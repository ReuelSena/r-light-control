<script setup lang="ts">
// Knob giratório: arrasto vertical relativo (150 px = curso todo), roda do mouse (1% por clique)
// e toque duplo para voltar ao padrão.
const props = withDefaults(defineProps<{ default?: number; color?: string; size?: number }>(), {
  default: 0,
  color: 'var(--color-select)',
  size: 56,
})

const model = defineModel<number>({ required: true })
// Otimista sob o dedo: o gesto não espera a volta do servidor
const value = useOptimistic(model)
const drag = useDrag((_, dy) => (value.value = clamp01(value.value - dy / 150)), () => (value.value = props.default))

// Mesma conta do fader: um clique da roda (100 px, ou 3 linhas no Firefox) = 1%
function wheel(e: WheelEvent) {
  value.value = clamp01(value.value - (e.deltaY / (e.deltaMode === 1 ? 3 : 100)) * 0.01)
}

// Arco de 270° (de -135° a +135° a partir do topo) num círculo de raio 44
const ARC = 0.75 * 2 * Math.PI * 44
</script>

<template>
  <div
    class="rl-knob"
    :style="{ width: `${size}px`, height: `${size}px` }"
    role="slider"
    aria-valuemin="0"
    aria-valuemax="100"
    :aria-valuenow="Math.round(value * 100)"
    v-on="drag"
    @wheel.prevent="wheel"
  >
    <svg viewBox="0 0 100 100">
      <circle cx="50" cy="50" r="44" class="arc" :stroke-dasharray="`${ARC} 999`" />
      <circle cx="50" cy="50" r="44" class="arc" :style="{ stroke: color }" :stroke-dasharray="`${ARC * value} 999`" />
    </svg>
    <div class="cap" :style="{ rotate: `${value * 270 - 135}deg` }" />
  </div>
</template>

<style scoped>
.rl-knob {
  position: relative;
  flex-shrink: 0;
  touch-action: none;
}

/* O traço do círculo começa às 3 h; girando 135° ele começa embaixo à esquerda */
svg {
  position: absolute;
  inset: 0;
  rotate: 135deg;
}

.arc {
  fill: none;
  stroke: #08090a;
  stroke-width: 7;
  stroke-linecap: round;
}

.cap {
  position: absolute;
  inset: 18%;
  border-radius: 50%;
  background: radial-gradient(circle at 35% 30%, #5d626a, #23262a 70%);
  box-shadow: 0 3px 5px rgb(0 0 0 / 0.7), inset 0 1px 0 rgb(255 255 255 / 0.15);
}

.cap::after {
  content: '';
  position: absolute;
  left: 50%;
  top: 8%;
  width: 3px;
  height: 30%;
  translate: -50% 0;
  border-radius: 2px;
  background: var(--color-fg);
}
</style>

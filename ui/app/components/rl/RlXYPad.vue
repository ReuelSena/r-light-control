<script setup lang="ts">
// Pad XY (pan/tilt): arrasto relativo como o fader (E9), com botão FINO. Toque duplo centraliza.
const value = defineModel<{ x: number; y: number }>({ required: true })
const { t } = useI18n()
const { format } = useValueFormat()
const fine = ref(false)
const pad = ref<HTMLElement>()

const drag = useDrag(
  (dx, dy) => {
    const k = fine.value ? 0.1 : 1
    value.value = {
      x: clamp01(value.value.x + (dx / pad.value!.clientWidth) * k),
      y: clamp01(value.value.y - (dy / pad.value!.clientHeight) * k),
    }
  },
  () => (value.value = { x: 0.5, y: 0.5 }),
)
</script>

<template>
  <div class="rl-xy">
    <div ref="pad" class="pad sunken" v-on="drag">
      <div class="line h" :style="{ bottom: `${value.y * 100}%` }" />
      <div class="line v" :style="{ left: `${value.x * 100}%` }" />
      <div class="dot" :style="{ left: `${value.x * 100}%`, bottom: `${value.y * 100}%` }" />
    </div>
    <div class="footer">
      <RlLcd class="flex-1">X {{ format(value.x) }} · Y {{ format(value.y) }}</RlLcd>
      <RlButton v-model="fine" mode="toggle" size="sm" color="var(--color-flash)">{{ t('rl.fine') }}</RlButton>
    </div>
  </div>
</template>

<style scoped>
.rl-xy {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.pad {
  position: relative;
  aspect-ratio: 1;
  border-radius: 6px;
  overflow: hidden;
  touch-action: none;
  background-image:
    linear-gradient(rgb(255 255 255 / 0.06) 1px, transparent 1px),
    linear-gradient(90deg, rgb(255 255 255 / 0.06) 1px, transparent 1px);
  background-size: 25% 25%;
}

.line {
  position: absolute;
  background: var(--color-position);
  opacity: 0.55;
}

.h {
  left: 0;
  right: 0;
  height: 1px;
}

.v {
  top: 0;
  bottom: 0;
  width: 1px;
}

.dot {
  position: absolute;
  width: 24px;
  height: 24px;
  translate: -50% 50%;
  border: 2px solid #fff;
  border-radius: 50%;
  background: var(--color-position);
  box-shadow: 0 0 14px var(--color-position);
}

.footer {
  display: flex;
  gap: 6px;
  align-items: center;
}
</style>

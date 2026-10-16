<script setup lang="ts">
// Seletor de cor: roda de matiz/saturação (toque absoluto), amostras de gelatina e valores RGB.
// A roda sempre usa brilho máximo: a intensidade fica com o dimmer.
const color = defineModel<string>({ required: true }) // '#rrggbb'
const wheel = ref<HTMLElement>()

const swatches = ['#ff0000', '#ff6a00', '#ffb000', '#ffff00', '#00ff00', '#00ffff', '#0060ff', '#0000ff', '#7a00ff', '#ff00ff', '#ff4fa0', '#ffd6a0', '#ffffff']

const rgb = computed(() => [1, 3, 5].map((i) => parseInt(color.value.slice(i, i + 2), 16)))
const toHex = (c: number[]) => '#' + c.map((v) => Math.round(v).toString(16).padStart(2, '0')).join('')

// Matiz (graus, sentido horário a partir do topo, como a roda) e saturação da cor atual
const hs = computed(() => {
  const [r = 0, g = 0, b = 0] = rgb.value.map((c) => c / 255)
  const max = Math.max(r, g, b)
  const d = max - Math.min(r, g, b)
  let h = 0
  if (d) h = max === r ? ((g - b) / d) % 6 : max === g ? (b - r) / d + 2 : (r - g) / d + 4
  return { h: ((h * 60 + 360) % 360) * (Math.PI / 180), s: max ? d / max : 0 }
})

function pick(e: PointerEvent) {
  const box = wheel.value!.getBoundingClientRect()
  const dx = e.clientX - box.left - box.width / 2
  const dy = e.clientY - box.top - box.height / 2
  const h = ((Math.atan2(dx, -dy) * 180) / Math.PI + 360) % 360
  const s = Math.min(1, Math.hypot(dx, dy) / (box.width / 2))
  // HSV → RGB com V = 1
  const f = (n: number) => {
    const k = (n + h / 60) % 6
    return 255 * (1 - s * Math.max(0, Math.min(k, 4 - k, 1)))
  }
  color.value = toHex([f(5), f(3), f(1)])
}

function down(e: PointerEvent) {
  wheel.value!.setPointerCapture(e.pointerId)
  pick(e)
}

function move(e: PointerEvent) {
  if (wheel.value!.hasPointerCapture(e.pointerId)) pick(e)
}

function setChannel(i: number, e: Event) {
  const c = rgb.value.slice()
  c[i] = Math.min(255, Math.max(0, Number((e.target as HTMLInputElement).value) || 0))
  color.value = toHex(c)
}
</script>

<template>
  <div class="rl-color">
    <div ref="wheel" class="wheel" @pointerdown="down" @pointermove="move">
      <div
        class="marker"
        :style="{
          left: `${50 + Math.sin(hs.h) * hs.s * 50}%`,
          top: `${50 - Math.cos(hs.h) * hs.s * 50}%`,
          background: color,
        }"
      />
    </div>
    <div class="swatches">
      <button
        v-for="swatch in swatches"
        :key="swatch"
        type="button"
        class="swatch"
        :class="{ active: swatch === color }"
        :style="{ background: swatch }"
        :aria-label="swatch"
        @click="color = swatch"
      />
    </div>
    <div class="rgb">
      <label v-for="(channel, i) in 'RGB'" :key="channel">
        {{ channel }}
        <input type="number" class="field" min="0" max="255" :value="rgb[i]" @change="setChannel(i, $event)">
      </label>
    </div>
  </div>
</template>

<style scoped>
.rl-color {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.wheel {
  position: relative;
  aspect-ratio: 1;
  margin: 6px;
  border-radius: 50%;
  touch-action: none;
  background:
    radial-gradient(closest-side, #fff, rgb(255 255 255 / 0)),
    conic-gradient(red, yellow, lime, cyan, blue, magenta, red);
  box-shadow: 0 0 0 5px #08090a, 0 0 0 6px #000, 0 4px 12px rgb(0 0 0 / 0.6);
}

.marker {
  position: absolute;
  width: 28px;
  height: 28px;
  translate: -50% -50%;
  border: 3px solid #fff;
  border-radius: 50%;
  box-shadow: 0 0 0 1px #000, 0 2px 6px rgb(0 0 0 / 0.6);
  pointer-events: none;
}

.swatches {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(36px, 1fr));
  gap: 6px;
}

.swatch {
  aspect-ratio: 1;
  min-height: 36px;
  border: 1px solid #000;
  border-radius: 6px;
  box-shadow: inset 0 1px 0 rgb(255 255 255 / 0.35), 0 2px 3px rgb(0 0 0 / 0.6);
}

.swatch.active {
  outline: 3px solid var(--color-select);
  outline-offset: 2px;
}

.rgb {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 6px;
}

.rgb label {
  display: flex;
  align-items: center;
  gap: 6px;
  color: var(--color-fg-muted);
  font-size: 12px;
  font-weight: 600;
}

.rgb input {
  width: 100%;
  min-width: 0;
  padding: 0 6px;
}
</style>

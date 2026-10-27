<script setup lang="ts">
// Curva de velocidade desenhada do EFX: o tempo da volta na horizontal, o quanto do desenho já foi andado na
// vertical. O traço muda os 16 pontos; ao soltar, a curva é ajustada para ir de 0 à volta inteira sem descer.
const props = defineProps<{ points: number[] }>()
const emit = defineEmits<{ change: [points: number[]] }>()

const draft = ref<number[] | null>(null)
const shown = computed(() => draft.value ?? props.points)
const line = computed(() => shown.value.map((v, i) => `${(i / 15) * 100},${100 - (v / 255) * 100}`).join(' '))
let last: number | null = null

// Ponto do traço: a coluna mais próxima recebe a altura, e as que o dedo pulou ficam na reta entre as duas
function mark(e: PointerEvent) {
  if (!draft.value) return
  // A área tem 4% de margem em volta (o viewBox vai de -4 a 104)
  const box = (e.currentTarget as SVGElement).getBoundingClientRect()
  const at = (pos: number, size: number) => Math.min(Math.max(((pos / size) * 108 - 4) / 100, 0), 1)
  const i = Math.round(at(e.clientX - box.left, box.width) * 15)
  const v = Math.round((1 - at(e.clientY - box.top, box.height)) * 255)
  const from = last ?? i
  const a = draft.value[from]!
  for (let k = Math.min(from, i); k <= Math.max(from, i); k++) draft.value[k] = from === i ? v : Math.round(a + ((v - a) * (k - from)) / (i - from))
  last = i
}

function start(e: PointerEvent) {
  ;(e.currentTarget as SVGElement).setPointerCapture(e.pointerId)
  draft.value = [...props.points]
  last = null
  mark(e)
}

function end() {
  if (!draft.value) return
  const points = [...draft.value]
  points[0] = 0
  points[15] = 255
  for (let i = 1; i < 15; i++) points[i] = Math.max(points[i]!, points[i - 1]!)
  draft.value = null
  emit('change', points)
}
</script>

<template>
  <svg
    viewBox="-4 -4 108 108"
    class="sunken aspect-square w-full max-w-40 touch-none rounded-md"
    @pointerdown="start"
    @pointermove="mark"
    @pointerup="end"
    @pointercancel="end"
  >
    <path d="M0 100L100 0" stroke="var(--color-line)" stroke-dasharray="3 3" />
    <polyline :points="line" fill="none" stroke="var(--color-position)" stroke-width="2" />
  </svg>
</template>

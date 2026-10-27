<script setup lang="ts">
import type { Efx } from '~/types/generated/Efx'

// Prévia animada do EFX: os caminhos (o geral e o de cada item com desenho próprio ou espelhado) e as
// fixtures andando neles, com a mesma conta da saída (utils/functions). O relógio fica aqui dentro: no
// editor, ele o redesenharia a cada quadro, e o redesenho apagaria o que se digita nos campos antes de
// chegar ao servidor. Com `drawing`, o traço do dedo ou do mouse vira o desenho à mão (`draw`, no espaço do
// pan e do tilt, de 0 a 255).
const props = defineProps<{ efx: Efx; drawing?: boolean }>()
const emit = defineEmits<{ draw: [points: [number, number][]] }>()
const { state } = useEngine()
const project = computed(() => state.value.project)
const now = useClock(33)

const moving = computed(() => efxFixtures(project.value, props.efx))
const colorOf = (id: string) => project.value.fixtures.find((f) => f.id === id)?.color ?? '#fff'
// Um caminho por aparência (desenho e espelho); o geral sem espelho na cor da posição. O tilt cresce para
// cima, como no palco visto de frente
const paths = computed(() => {
  const general = { fixture: '', reverse: false, startOffset: 0, invertPan: false, invertTilt: false, swapAxes: false, shape: props.efx }
  const looks = new Map<string, { points: string; color: string }>()
  for (const f of [general, ...moving.value]) {
    const key = JSON.stringify([f.shape === props.efx || f.shape, f.invertPan, f.invertTilt, f.swapAxes])
    if (looks.has(key)) continue
    const points = Array.from({ length: 129 }, (_, i) => efxMirror(f, efxPoint(f.shape, (i / 128) * Math.PI * 2)))
    const plain = f.shape === props.efx && !f.invertPan && !f.invertTilt && !f.swapAxes
    looks.set(key, { points: points.map((p) => `${p.x},${255 - p.y}`).join(' '), color: plain ? 'var(--color-position)' : colorOf(f.fixture) })
  }
  return [...looks.values()]
})
const dots = computed(() => efxPositions(props.efx, moving.value, now.value))

// O traço, no espaço do desenho (o viewBox vai de -10 a 265)
const stroke = ref<[number, number][] | null>(null)
function mark(e: PointerEvent) {
  if (!stroke.value) return
  const box = (e.currentTarget as SVGElement).getBoundingClientRect()
  const at = (pos: number, size: number) => Math.round(Math.min(Math.max((pos / size) * 275 - 10, 0), 255))
  const p: [number, number] = [at(e.clientX - box.left, box.width), 255 - at(e.clientY - box.top, box.height)]
  const last = stroke.value.at(-1)
  if (!last || Math.hypot(p[0] - last[0], p[1] - last[1]) >= 4) stroke.value.push(p)
}
function start(e: PointerEvent) {
  if (!props.drawing) return
  ;(e.currentTarget as SVGElement).setPointerCapture(e.pointerId)
  stroke.value = []
  mark(e)
}
function end() {
  if (stroke.value && stroke.value.length > 2) emit('draw', stroke.value)
  stroke.value = null
}
</script>

<template>
  <svg
    viewBox="-10 -10 275 275"
    class="sunken aspect-square w-full max-w-xs rounded-md"
    :class="{ 'touch-none cursor-crosshair': drawing }"
    @pointerdown="start"
    @pointermove="mark"
    @pointerup="end"
    @pointercancel="end"
  >
    <path d="M127.5 -10V265M-10 127.5H265" stroke="var(--color-line)" stroke-dasharray="4 4" />
    <polyline v-if="stroke" :points="stroke.map(([x, y]) => `${x},${255 - y}`).join(' ')" fill="none" stroke="var(--color-position)" stroke-width="3" />
    <template v-else>
      <polyline v-for="(p, i) in paths" :key="i" :points="p.points" fill="none" :stroke="p.color" stroke-width="2" opacity="0.7" />
      <circle
        v-for="(p, i) in dots"
        :key="i"
        :cx="p.x"
        :cy="255 - p.y"
        r="7"
        :fill="colorOf(moving[i]!.fixture)"
        stroke="#000"
        stroke-width="2"
      />
    </template>
  </svg>
</template>

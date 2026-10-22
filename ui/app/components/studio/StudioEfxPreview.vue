<script setup lang="ts">
import type { Efx } from '~/types/generated/Efx'

// Prévia animada do EFX: o desenho e as fixtures andando nele, com a mesma conta da saída
// (utils/functions). O relógio fica aqui dentro: no editor, ele o redesenharia a cada quadro, e o
// redesenho apagaria o que se digita nos campos antes de chegar ao servidor.
const props = defineProps<{ efx: Efx }>()
const { state } = useEngine()
const project = computed(() => state.value.project)
const now = useClock(33)

// O tilt cresce para cima, como no palco visto de frente
const path = computed(() =>
  Array.from({ length: 129 }, (_, i) => efxPoint(props.efx, (i / 128) * Math.PI * 2)).map((p) => `${p.x},${255 - p.y}`).join(' '),
)
const moving = computed(() => efxFixtures(project.value, props.efx))
const dots = computed(() => efxPositions(props.efx, moving.value, now.value))
const colorOf = (id: string) => project.value.fixtures.find((f) => f.id === id)?.color ?? '#fff'
</script>

<template>
  <svg viewBox="-10 -10 275 275" class="sunken aspect-square w-full max-w-xs rounded-md">
    <path d="M127.5 -10V265M-10 127.5H265" stroke="var(--color-line)" stroke-dasharray="4 4" />
    <polyline :points="path" fill="none" stroke="var(--color-position)" stroke-width="2" opacity="0.7" />
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
  </svg>
</template>

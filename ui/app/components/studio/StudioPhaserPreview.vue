<script setup lang="ts">
import type { Function } from '~/types/generated/Function'

// Prévia animada do Phaser: uma linha por camada, com uma barra para cada fixture do grupo (a altura é o
// valor da onda agora), com as mesmas contas da saída (utils/functions). O relógio fica aqui dentro: no
// editor, ele o redesenharia a cada quadro, e o redesenho apagaria o que se digita nos campos antes de
// chegar ao servidor.
const props = defineProps<{ fn: Extract<Function, { type: 'phaser' }> }>()
const { t } = useI18n()
const { state } = useEngine()
const project = computed(() => state.value.project)
const now = useClock(33)

const rows = computed(() => props.fn.layers.map((layer) => ({ layer, n: phaserCount(project.value, props.fn.group, layer.attribute) })))
const turns = computed(() => now.value / phaserCycleMs(props.fn, state.value.master.bpm))

// A barra da fixture `i`: do chão ao valor; no relativo, do zero (127, a linha do meio) ao deslocamento
function bar(layer: (typeof props.fn.layers)[number], i: number) {
  const v = phaserValue(layer, props.fn.spread, i, turns.value)
  if (!layer.relative) return { y: 40 - (v / 255) * 40, height: (v / 255) * 40 }
  const height = (Math.abs(v - 127) / 127) * 20
  return { y: v >= 127 ? 20 - height : 20, height }
}
</script>

<template>
  <div class="sunken flex flex-col gap-2 rounded-md p-2">
    <p v-if="!fn.group" class="text-sm text-fg-muted">{{ t('phaser.noGroup') }}</p>
    <div v-for="(row, k) in rows" :key="k" class="grid grid-cols-[7.5rem_1fr] items-center gap-2">
      <span class="flex items-center gap-1 truncate text-xs text-fg-muted">
        <component :is="attributeLook[row.layer.attribute].icon" :size="14" class="shrink-0" :style="{ color: attributeLook[row.layer.attribute].color }" />
        {{ t(`attribute.${row.layer.attribute}`) }}<template v-if="row.layer.relative"> ±</template>
      </span>
      <svg v-if="row.n" :viewBox="`0 0 ${row.n * 10} 40`" preserveAspectRatio="none" class="h-10 w-full">
        <line v-if="row.layer.relative" x1="0" :x2="row.n * 10" y1="20" y2="20" stroke="var(--color-line)" stroke-dasharray="3 3" />
        <rect
          v-for="i in row.n"
          :key="i"
          :x="(i - 1) * 10 + 1"
          :y="bar(row.layer, i - 1).y"
          width="8"
          :height="bar(row.layer, i - 1).height"
          :fill="attributeLook[row.layer.attribute].color"
        />
      </svg>
      <span v-else class="text-xs text-fg-muted">{{ t('phaser.noFixtures') }}</span>
    </div>
  </div>
</template>

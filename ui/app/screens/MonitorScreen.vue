<script setup lang="ts">
import { Clock, Cpu, MemoryStick, Timer, Users } from 'lucide-vue-next'
import type { Fixture } from '~/types/generated/Fixture'

// Monitoramento (E12): os universos mandando sinal (valores de saída ao vivo) e os dados do app e da
// máquina. O tick do motor tem limite de 20 ms (A1).
const { t } = useI18n()
const { format } = useValueFormat()
const { state, output } = useEngine()
const project = computed(() => state.value.project)
const universe = ref(project.value.universes[0]?.id ?? '')
const stats = computed(() => state.value.stats)

const tiles = computed(() => {
  const s = stats.value.uptimeS
  const uptime = `${Math.floor(s / 3600)}:${String(Math.floor(s / 60) % 60).padStart(2, '0')}:${String(s % 60).padStart(2, '0')}`
  return [
    { icon: Cpu, label: t('monitor.cpu'), value: `${stats.value.cpuPercent.toFixed(0)}%`, level: stats.value.cpuPercent / 100 },
    { icon: MemoryStick, label: t('monitor.memory'), value: `${stats.value.memoryMb} MB` },
    { icon: Timer, label: t('monitor.tick'), value: `${stats.value.tickMs.toFixed(2)} ms`, level: stats.value.tickMs / 20 },
    { icon: Clock, label: t('monitor.uptime'), value: uptime },
    { icon: Users, label: t('monitor.clients'), value: String(state.value.remote.clients.length) },
  ]
})

// Fixture dona de cada canal do universo, para colorir a grade
const owners = computed(() => {
  const owner: (Fixture | undefined)[] = []
  for (const f of project.value.fixtures.filter((x) => x.universe === universe.value)) {
    for (let c = f.address; c < f.address + channelCount(project.value, f) && c <= 512; c++) owner[c] = f
  }
  return owner
})
</script>

<template>
  <div class="flex h-full flex-col gap-3 overflow-auto p-2">
    <div class="grid grid-cols-1 gap-3 @5xl:grid-cols-2">
      <section class="panel">
        <h2 class="heading">{{ t('monitor.app') }}</h2>
        <div class="grid grid-cols-2 gap-2 @xl:grid-cols-3">
          <div v-for="tile in tiles" :key="tile.label" class="sunken grid gap-1.5 rounded-md p-3">
            <span class="flex items-center gap-1.5 text-xs text-fg-muted">
              <component :is="tile.icon" :size="14" />
              {{ tile.label }}
            </span>
            <span class="font-lcd text-xl text-lcd">{{ tile.value }}</span>
            <span v-if="tile.level !== undefined" class="h-1.5 overflow-hidden rounded-full bg-surface-2">
              <span class="block h-full rounded-full bg-on transition-[width]" :style="{ width: `${Math.min(100, tile.level * 100)}%` }" />
            </span>
          </div>
        </div>
      </section>

      <section class="panel">
        <h2 class="heading">{{ t('monitor.outputs') }}</h2>
        <div v-for="u in project.universes" :key="u.id" class="sunken flex flex-wrap items-center gap-3 rounded-md p-3">
          <div class="min-w-0 flex-1 basis-40">
            <p class="truncate font-semibold">{{ u.name }}</p>
            <p class="truncate text-xs text-fg-muted">{{ state.interfaces.find((i) => i.id === u.output)?.name ?? t('io.none') }}</p>
          </div>
          <template v-for="i in state.interfaces.filter((x) => x.id === u.output)" :key="i.id">
            <span class="rl-led" :style="{ '--led': i.connected ? 'var(--color-on)' : 'var(--color-danger)' }" />
            <RlLcd>{{ i.fps }} fps</RlLcd>
            <RlLcd>{{ i.jitterMs.toFixed(1) }} ms</RlLcd>
            <RlLcd :color="i.errors ? 'var(--color-danger)' : undefined">{{ t('monitor.errors', { n: i.errors }) }}</RlLcd>
          </template>
        </div>
      </section>
    </div>

    <RlTabs v-model="universe" class="min-h-0" :tabs="project.universes.map((u) => ({ value: u.id, label: u.name }))">
      <section class="panel @container">
        <h2 class="heading">{{ t('monitor.values') }}</h2>
        <div class="grid-512">
          <span
            v-for="c in 512"
            :key="c"
            class="cell"
            :style="{ '--tag': owners[c]?.color ?? '#ffffff', '--v': (output[universe]?.[c - 1] ?? 0) / 255 }"
            :title="owners[c] ? `${c} · ${owners[c]!.name}` : String(c)"
          >
            <small>{{ c }}</small>
            {{ format((output[universe]?.[c - 1] ?? 0) / 255) }}
          </span>
        </div>
      </section>
    </RlTabs>
  </div>
</template>

<style scoped>
.grid-512 {
  display: grid;
  grid-template-columns: repeat(16, minmax(0, 1fr));
  gap: 2px;
}

@container (min-width: 760px) {
  .grid-512 {
    grid-template-columns: repeat(32, minmax(0, 1fr));
  }
}

/* O fundo acende na cor da fixture conforme o valor */
.cell {
  display: grid;
  min-height: 32px;
  place-content: center;
  overflow: hidden;
  border-radius: 2px;
  background: color-mix(in srgb, var(--tag) calc(var(--v) * 75%), #08090a);
  color: var(--color-fg);
  font-family: var(--font-lcd);
  font-size: 10px;
  line-height: 1.1;
  text-shadow: 0 0 2px #000, 0 0 3px #000;
  text-align: center;
}

.cell small {
  color: var(--color-fg-dim);
  font-size: 8px;
}
</style>

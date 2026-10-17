<script setup lang="ts">
import type { TestPattern } from '~/types/generated/TestPattern'

// Teste da saída DMX (F3): padrões gerados pelo motor em todos os canais, as saídas com quadros por
// segundo e atraso, e os primeiros canais de cada universo como saem do motor.
const { t } = useI18n()
const { state, output, send } = useEngine()
const pattern = computed({
  get: () => state.value.testPattern ?? 'off',
  set: (value: TestPattern | 'off') => send({ op: 'setTestPattern', pattern: value === 'off' ? null : value }),
})
const patterns = computed(() => (['off', 'ramp', 'chase'] as const).map((value) => ({ value, label: t(`dev.dmx.${value}`) })))
</script>

<template>
  <div class="flex h-dvh flex-col">
    <header class="brushed flex items-center gap-2 px-3 py-1.5">
      <RlButton size="sm" @press="navigateTo('/console')">{{ t('dev.back') }}</RlButton>
      <h1 class="heading flex items-center gap-1">{{ t('dev.dmx.title') }} <RlInfo :text="t('dev.dmx.info')" /></h1>
    </header>
    <div class="flex flex-col gap-3 overflow-auto p-2">
      <RlSegmented v-model="pattern" :options="patterns" />

      <section class="panel">
        <h2 class="heading">{{ t('dev.dmx.outputs') }}</h2>
        <div v-for="i in state.interfaces" :key="i.id" class="flex flex-wrap items-center gap-2">
          <span class="rl-led" :style="{ '--led': i.connected ? 'var(--color-on)' : 'var(--color-danger)' }" />
          <span class="min-w-40 font-semibold">{{ i.name }}</span>
          <RlLcd>{{ i.fps.toFixed(0) }} fps</RlLcd>
          <RlLcd>{{ i.jitterMs.toFixed(2) }} ms</RlLcd>
          <RlLcd :color="i.errors ? 'var(--color-danger)' : undefined">{{ i.errors }} erros</RlLcd>
        </div>
      </section>

      <section v-for="u in state.project.universes" :key="u.id" class="panel">
        <h2 class="heading">{{ t('dev.dmx.universe', { name: u.name }) }}</h2>
        <div class="grid grid-cols-[repeat(auto-fill,minmax(48px,1fr))] gap-1">
          <RlLcd v-for="c in 32" :key="c" class="text-xs!">
            <span class="text-fg-dim">{{ c }}</span><br>{{ output[u.id]?.[c - 1] ?? 0 }}
          </RlLcd>
        </div>
      </section>
    </div>
  </div>
</template>

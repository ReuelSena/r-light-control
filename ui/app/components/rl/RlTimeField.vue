<script setup lang="ts">
// Campo de tempo: mostra segundos (ou batidas, com `beats`) e guarda milissegundos (ou milésimos de batida).
// Com `infinite`, o botão ∞ liga e desliga o tempo infinito (o hold que espera o Go).
const props = defineProps<{ beats?: boolean; infinite?: boolean }>()
const ms = defineModel<number>({ required: true })
const { t } = useI18n()
const endless = computed(() => props.infinite && ms.value === INFINITE)
</script>

<template>
  <span class="rl-time field">
    <span v-if="endless" class="flex-1 text-xl leading-none">∞</span>
    <template v-else>
      <input
        type="number"
        min="0"
        step="0.1"
        :value="ms / 1000"
        @change="ms = Math.max(0, Math.round(Number(($event.target as HTMLInputElement).value) * 1000))"
      >
      <span class="unit">{{ beats ? 'bt' : 's' }}</span>
    </template>
    <button v-if="infinite" type="button" class="unit" :class="{ on: endless }" :title="t('studio.infinite')" :aria-label="t('studio.infinite')" @click="ms = endless ? 0 : INFINITE">∞</button>
  </span>
</template>

<style scoped>
.rl-time {
  display: flex;
  align-items: center;
  gap: 4px;
  padding-right: 8px;
}

.rl-time:focus-within {
  border-color: var(--color-select);
}

input {
  width: 100%;
  min-width: 3ch;
  background: transparent;
  outline: none;
}

.unit {
  color: var(--color-fg-dim);
  font-size: 12px;
}

button.unit {
  padding: 0 2px;
  font-size: 15px;
}

button.on {
  color: var(--color-select);
}
</style>

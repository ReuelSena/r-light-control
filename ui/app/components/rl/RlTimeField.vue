<script setup lang="ts">
// Campo de tempo: mostra segundos (ou batidas, com `beats`) e guarda milissegundos (ou milésimos de batida).
defineProps<{ beats?: boolean }>()
const ms = defineModel<number>({ required: true })
</script>

<template>
  <span class="rl-time field">
    <input
      type="number"
      min="0"
      step="0.1"
      :value="ms / 1000"
      @change="ms = Math.max(0, Math.round(Number(($event.target as HTMLInputElement).value) * 1000))"
    >
    <span class="unit">{{ beats ? 'bt' : 's' }}</span>
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
</style>

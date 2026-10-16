<script setup lang="ts">
// Teclado numérico para digitar um valor exato, em % ou DMX conforme o formato em uso (E10).
// Emite o valor normalizado (0–1).
const emit = defineEmits<{ confirm: [value: number] }>()
const { t } = useI18n()
const { mode, max } = useValueFormat()
const entry = ref('')
const keys = ['7', '8', '9', '4', '5', '6', '1', '2', '3', 'C', '0', '⌫']

function press(key: string) {
  if (key === 'C') entry.value = ''
  else if (key === '⌫') entry.value = entry.value.slice(0, -1)
  else entry.value = String(Math.min(max.value, Number(entry.value + key)))
}
</script>

<template>
  <div class="rl-numpad">
    <RlLcd class="display">{{ entry || '0' }}{{ mode === 'percent' ? '%' : '' }}</RlLcd>
    <div class="keys">
      <RlButton v-for="key in keys" :key="key" size="lg" @press="press(key)">{{ key }}</RlButton>
    </div>
    <RlButton size="lg" class="ok" @press="emit('confirm', Number(entry || 0) / max)">{{ t('rl.ok') }}</RlButton>
  </div>
</template>

<style scoped>
.rl-numpad {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.display {
  font-size: 28px;
  text-align: right;
}

.keys {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 6px;
}

.ok {
  color: var(--color-on);
}
</style>

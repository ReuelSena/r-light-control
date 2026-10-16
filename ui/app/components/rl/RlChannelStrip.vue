<script setup lang="ts">
// Channel strip no estilo Ui24R: display, botões FLASH e SEL, fader (com FINO) e etiqueta.
// Tocar no display pede a edição do valor exato (evento `edit`). Com `plain`, sem os botões; o slot
// `label` troca o texto da etiqueta (ex.: ícone + nome).
defineProps<{ label: string; color?: string; meter?: number; cap?: 'silver' | 'red'; plain?: boolean }>()
const value = defineModel<number>({ required: true })
const selected = defineModel<boolean>('selected', { default: false })
const flash = defineModel<boolean>('flash', { default: false })
defineEmits<{ edit: [] }>()
const { t } = useI18n()
const { format } = useValueFormat()
</script>

<template>
  <div class="rl-strip" :class="{ selected }">
    <RlLcd class="lcd" @click="$emit('edit')">{{ format(value) }}</RlLcd>
    <div v-if="!plain" class="buttons">
      <RlButton v-model="flash" mode="flash" size="sm" color="var(--color-flash)">{{ t('rl.flash') }}</RlButton>
      <RlButton v-model="selected" mode="toggle" size="sm" color="var(--color-select)">{{ t('rl.select') }}</RlButton>
    </div>
    <RlFader v-model="value" class="fader" :meter="meter" :cap="cap" />
    <div class="label" :style="{ '--tag': color ?? 'var(--color-line)' }" :title="label">
      <slot name="label">{{ label }}</slot>
    </div>
  </div>
</template>

<style scoped>
.rl-strip {
  display: flex;
  flex-direction: column;
  gap: 6px;
  flex-shrink: 0;
  width: 88px;
  padding: 6px 5px 0;
  background: linear-gradient(#1b1d21, #141619);
  border-right: 1px solid #000;
  box-shadow: inset 1px 0 0 rgb(255 255 255 / 0.04);
}

.lcd {
  cursor: pointer;
}

.buttons {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 4px;
}

.fader {
  flex: 1;
}

.label {
  margin: 0 -5px;
  padding: 8px 4px;
  overflow: hidden;
  background: #0b0c0e;
  border-top: 3px solid var(--tag);
  font-size: 11px;
  font-weight: 600;
  text-align: center;
  text-overflow: ellipsis;
  text-transform: uppercase;
  white-space: nowrap;
}

.selected .label {
  background: var(--color-select);
  color: #111;
}
</style>

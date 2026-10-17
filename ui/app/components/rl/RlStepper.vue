<script setup lang="ts">
import { Minus, Plus } from 'lucide-vue-next'

// Valor numa escala fixa, com − e + (ex.: zoom). Toque duplo no display volta ao padrão.
const props = withDefaults(defineProps<{ steps: number[]; default?: number; format?: (v: number) => string }>(), {
  default: 1,
  format: (v: number) => `${Math.round(v * 100)}%`,
})
const value = defineModel<number>({ required: true })
const { t } = useI18n()

const go = (delta: number) => (value.value = stepValue(props.steps, value.value, delta))
</script>

<template>
  <div class="flex items-center gap-1">
    <RlButton size="sm" :aria-label="t('rl.less')" @press="go(-1)"><Minus :size="16" /></RlButton>
    <RlLcd class="min-w-16 text-center" @dblclick="value = props.default">{{ format(value) }}</RlLcd>
    <RlButton size="sm" :aria-label="t('rl.more')" @press="go(1)"><Plus :size="16" /></RlButton>
  </div>
</template>

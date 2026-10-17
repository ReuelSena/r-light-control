<script setup lang="ts">
// Campos de posição (m) e rotação (graus) de uma fixture ou de um elemento de palco.
defineProps<{ value: Placement; title: string }>()
const emit = defineEmits<{ change: [patch: Partial<Placement>] }>()
const { t } = useI18n()

function set(key: keyof Placement, e: Event) {
  const value = Number((e.target as HTMLInputElement).value)
  if (!Number.isNaN(value)) emit('change', { [key]: value })
}
</script>

<template>
  <h3 class="heading">{{ title }}</h3>
  <div class="grid grid-cols-3 gap-2">
    <label v-for="key in ['x', 'y', 'z'] as const" :key="key" class="field-label">
      {{ t(`view3d.axis.${key}`) }}
      <input class="field w-full" type="number" step="0.1" :value="value[key]" @change="set(key, $event)">
    </label>
  </div>
  <h3 class="heading">{{ t('view3d.rotation') }}</h3>
  <div class="grid grid-cols-3 gap-2">
    <label v-for="key in ['rotationX', 'rotationY', 'rotationZ'] as const" :key="key" class="field-label">
      {{ t(`view3d.axis.${key}`) }}
      <input class="field w-full" type="number" step="5" :value="value[key]" @change="set(key, $event)">
    </label>
  </div>
</template>

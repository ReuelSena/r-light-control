<script setup lang="ts" generic="T extends string">
import { ToggleGroupItem, ToggleGroupRoot } from 'reka-ui'

// Escolha de uma entre poucas opções, no mesmo trilho das abas (RlTabs). Sempre há uma escolhida.
// `size="sm"` deixa o trilho da altura de um botão pequeno (barras sobre a 3D View, por exemplo).
defineProps<{ options: { value: T; label: string }[]; size?: 'sm' | 'md' }>()
const value = defineModel<T>({ required: true })
</script>

<template>
  <ToggleGroupRoot
    type="single"
    :model-value="value"
    class="rl-segmented sunken w-full @3xl:w-fit"
    :class="{ sm: size === 'sm' }"
    @update:model-value="(v) => v && (value = v as T)"
  >
    <ToggleGroupItem v-for="o in options" :key="o.value" :value="o.value" class="rl-segment">{{ o.label }}</ToggleGroupItem>
  </ToggleGroupRoot>
</template>

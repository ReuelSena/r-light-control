<script setup lang="ts" generic="T extends string">
import { ToggleGroupItem, ToggleGroupRoot } from 'reka-ui'

// Escolha de uma entre poucas opções, no mesmo trilho das abas (RlTabs). Sempre há uma escolhida.
defineProps<{ options: { value: T; label: string }[] }>()
const value = defineModel<T>({ required: true })
</script>

<template>
  <ToggleGroupRoot
    type="single"
    :model-value="value"
    class="rl-segmented sunken w-full @3xl:w-fit"
    @update:model-value="(v) => v && (value = v as T)"
  >
    <ToggleGroupItem v-for="o in options" :key="o.value" :value="o.value" class="rl-segment">{{ o.label }}</ToggleGroupItem>
  </ToggleGroupRoot>
</template>

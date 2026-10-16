<script setup lang="ts">
import { TabsContent, TabsList, TabsRoot, TabsTrigger } from 'reka-ui'

// Abas no estilo dos botões de visão da Ui24R (a aba ativa acende em laranja).
// O conteúdo de cada aba vem no slot com o nome do `value`.
defineProps<{ tabs: { value: string; label: string }[] }>()
const active = defineModel<string>({ required: true })
</script>

<template>
  <TabsRoot v-model="active" class="flex min-h-0 flex-col gap-2">
    <TabsList class="flex flex-wrap gap-1">
      <TabsTrigger v-for="tab in tabs" :key="tab.value" :value="tab.value" class="rl-tab raised">
        {{ tab.label }}
      </TabsTrigger>
    </TabsList>
    <TabsContent v-for="tab in tabs" :key="tab.value" :value="tab.value" class="min-h-0 flex-1 overflow-auto">
      <slot :name="tab.value" />
    </TabsContent>
  </TabsRoot>
</template>

<style scoped>
.rl-tab {
  min-height: 40px;
  padding: 0 14px;
  border-radius: 6px;
  color: var(--color-fg-muted);
  font-size: 12px;
  font-weight: 600;
  letter-spacing: 0.05em;
  text-transform: uppercase;
}

.rl-tab[data-state='active'] {
  background: linear-gradient(color-mix(in srgb, var(--color-select), white 25%), var(--color-select));
  color: #111;
  box-shadow: 0 0 12px color-mix(in srgb, var(--color-select) 50%, transparent), inset 0 1px 0 rgb(255 255 255 / 0.4);
}
</style>

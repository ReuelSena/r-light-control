<script setup lang="ts">
import { TabsContent, TabsList, TabsRoot, TabsTrigger } from 'reka-ui'

// Abas agrupadas num trilho afundado, como um seletor físico: a aba ativa é a tecla acesa em laranja.
// Ocupam a largura toda em espaços estreitos e só o necessário (abas iguais) em espaços largos.
// O conteúdo de cada aba vem no slot com o nome do `value`; sem ele, vale o slot padrão (o mesmo
// conteúdo para todas as abas, ex.: um universo por aba). O slot `actions` fica ao lado do trilho
// (ou embaixo, quando não cabe).
defineProps<{ tabs: { value: string; label: string }[] }>()
const active = defineModel<string>({ required: true })
</script>

<template>
  <TabsRoot v-model="active" class="flex min-h-0 flex-col gap-2">
    <div class="flex flex-wrap items-center gap-2">
      <TabsList class="rl-segmented sunken min-w-0 flex-1 basis-64 @3xl:flex-none @3xl:basis-auto">
        <TabsTrigger v-for="tab in tabs" :key="tab.value" :value="tab.value" class="rl-segment">
          {{ tab.label }}
        </TabsTrigger>
      </TabsList>
      <slot name="actions" />
    </div>
    <TabsContent v-for="tab in tabs" :key="tab.value" :value="tab.value" class="min-h-0 flex-1 overflow-auto">
      <slot :name="tab.value">
        <slot />
      </slot>
    </TabsContent>
  </TabsRoot>
</template>

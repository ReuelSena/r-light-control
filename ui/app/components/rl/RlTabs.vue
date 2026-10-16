<script setup lang="ts">
import { TabsContent, TabsList, TabsRoot, TabsTrigger } from 'reka-ui'

// Abas agrupadas num trilho afundado, como um seletor físico: a aba ativa é a tecla acesa em laranja.
// Ocupam a largura toda em espaços estreitos e só o necessário (abas iguais) em espaços largos.
// O conteúdo de cada aba vem no slot com o nome do `value`; sem ele, vale o slot padrão (o mesmo
// conteúdo para todas as abas, ex.: um universo por aba). O slot `actions` fica ao lado do trilho.
defineProps<{ tabs: { value: string; label: string }[] }>()
const active = defineModel<string>({ required: true })
</script>

<template>
  <TabsRoot v-model="active" class="flex min-h-0 flex-col gap-2">
    <div class="flex items-center gap-2">
      <TabsList class="rl-tabs sunken min-w-0 flex-1 @3xl:flex-none">
        <TabsTrigger v-for="tab in tabs" :key="tab.value" :value="tab.value" class="rl-tab">
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

<style scoped>
.rl-tabs {
  display: grid;
  flex-shrink: 0;
  grid-auto-columns: 1fr;
  grid-auto-flow: column;
  padding: 3px;
  border-radius: 9px;
}

.rl-tab {
  position: relative;
  min-height: 40px;
  padding: 2px 16px;
  border-radius: 6px;
  color: var(--color-fg-muted);
  font-size: 12px;
  font-weight: 600;
  letter-spacing: 0.05em;
  line-height: 1.1;
  text-transform: uppercase;
}

/* Divisória entre abas apagadas; some ao lado da acesa */
.rl-tab + .rl-tab::before {
  content: '';
  position: absolute;
  top: 10px;
  bottom: 10px;
  left: 0;
  width: 1px;
  background: var(--color-line);
}

.rl-tab[data-state='active']::before,
.rl-tab[data-state='active'] + .rl-tab::before {
  display: none;
}

.rl-tab:active {
  background: rgb(255 255 255 / 0.04);
}

.rl-tab:focus-visible {
  outline: 2px solid var(--color-select);
  outline-offset: -2px;
}

.rl-tab[data-state='active'] {
  background: linear-gradient(color-mix(in srgb, var(--color-select), white 25%), var(--color-select));
  color: #111;
  box-shadow: 0 0 12px color-mix(in srgb, var(--color-select) 50%, transparent), inset 0 1px 0 rgb(255 255 255 / 0.4), 0 1px 2px rgb(0 0 0 / 0.8);
}
</style>

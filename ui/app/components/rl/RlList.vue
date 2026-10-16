<script setup lang="ts">
// Lista de itens selecionáveis (bibliotecas, funções), com linhas altas para toque.
defineProps<{ items: { id: string; label: string; detail?: string }[] }>()
const selected = defineModel<string>()
</script>

<template>
  <ul class="rl-list sunken" role="listbox">
    <li
      v-for="item in items"
      :key="item.id"
      role="option"
      :aria-selected="item.id === selected"
      :class="{ active: item.id === selected }"
      @click="selected = item.id"
    >
      <span class="truncate">{{ item.label }}</span>
      <small v-if="item.detail">{{ item.detail }}</small>
    </li>
  </ul>
</template>

<style scoped>
.rl-list {
  overflow: auto;
  border-radius: 6px;
}

li {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
  min-height: 48px;
  padding: 0 12px;
  border-bottom: 1px solid #000;
  box-shadow: inset 0 1px 0 rgb(255 255 255 / 0.03);
  cursor: pointer;
}

li.active {
  background: color-mix(in srgb, var(--color-select) 22%, transparent);
  box-shadow: inset 3px 0 0 var(--color-select);
}

small {
  flex-shrink: 0;
  color: var(--color-fg-muted);
  font-size: 12px;
}
</style>

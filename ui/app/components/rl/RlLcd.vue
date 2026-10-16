<script setup lang="ts">
// Display LCD (texto ciano sobre fundo afundado), como nos canais da Ui24R.
defineProps<{ color?: string }>()
</script>

<template>
  <div class="rl-lcd sunken" :style="{ color }">
    <slot />
  </div>
</template>

<style scoped>
.rl-lcd {
  flex-shrink: 0;
  padding: 3px 6px;
  border-radius: 4px;
  color: var(--color-lcd);
  font-family: var(--font-lcd);
  font-size: 15px;
  line-height: 1.3;
  text-align: center;
  white-space: nowrap;
  overflow: hidden;
  text-shadow: 0 0 6px color-mix(in srgb, currentColor 55%, transparent);
}
</style>

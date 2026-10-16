<script setup lang="ts">
// Escolha de cor numa paleta curta (etiqueta de fixture, cor de widget).
defineProps<{ colors: string[] }>()
const color = defineModel<string>({ required: true })
</script>

<template>
  <div class="rl-swatches" role="radiogroup">
    <button
      v-for="swatch in colors"
      :key="swatch"
      type="button"
      role="radio"
      :aria-checked="swatch === color"
      :aria-label="swatch"
      :class="{ active: swatch === color }"
      :style="{ background: swatch }"
      @click="color = swatch"
    />
  </div>
</template>

<style scoped>
.rl-swatches {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}

button {
  width: 36px;
  height: 36px;
  border: 1px solid #000;
  border-radius: 6px;
  box-shadow: inset 0 1px 0 rgb(255 255 255 / 0.35), 0 2px 3px rgb(0 0 0 / 0.6);
}

.active {
  outline: 3px solid var(--color-select);
  outline-offset: 2px;
}
</style>

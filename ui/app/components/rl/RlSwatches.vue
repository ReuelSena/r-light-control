<script setup lang="ts">
// Escolha de cor numa paleta curta (etiqueta de fixture, cor de widget). Com `custom`, o último botão
// (arco-íris) pede uma cor fora da paleta: emite `custom` (quem usa abre o seletor de cor) e, com uma cor de
// fora da paleta escolhida, mostra essa cor.
const props = defineProps<{ colors: string[]; custom?: boolean }>()
const emit = defineEmits<{ custom: [] }>()
const color = defineModel<string>({ required: true })
const { t } = useI18n()
const isCustom = computed(() => props.custom && !props.colors.includes(color.value))
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
    <button
      v-if="custom"
      type="button"
      role="radio"
      class="custom"
      :aria-checked="isCustom"
      :aria-label="t('rl.customColor')"
      :class="{ active: isCustom }"
      :style="isCustom ? { background: color } : undefined"
      @click="emit('custom')"
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

.custom {
  background: conic-gradient(#f00, #ff0, #0f0, #0ff, #00f, #f0f, #f00);
}

.active {
  outline: 3px solid var(--color-select);
  outline-offset: 2px;
}
</style>

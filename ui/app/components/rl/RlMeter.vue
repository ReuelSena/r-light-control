<script setup lang="ts">
// Medidor vertical em segmentos de LED; `level` de 0 a 1.
withDefaults(defineProps<{ level: number; color?: string }>(), { color: 'var(--color-intensity)' })
</script>

<template>
  <div class="rl-meter sunken">
    <div class="leds">
      <div class="fill" :style="{ transform: `scaleY(${level})`, background: color }" />
    </div>
  </div>
</template>

<style scoped>
.rl-meter {
  position: relative;
  width: 8px;
  border-radius: 2px;
}

/* A máscara fica no contêiner fixo, para os segmentos não esticarem com o nível */
.leds {
  position: absolute;
  inset: 2px 1px;
  background: rgb(255 255 255 / 0.05);
  mask: repeating-linear-gradient(to top, #000 0 3px, transparent 3px 4px);
}

.fill {
  position: absolute;
  inset: 0;
  transform-origin: bottom;
  will-change: transform;
}
</style>

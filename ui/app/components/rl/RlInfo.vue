<script setup lang="ts">
import { Info } from 'lucide-vue-next'
import { PopoverArrow, PopoverContent, PopoverPortal, PopoverRoot, PopoverTrigger } from 'reka-ui'

// Ícone "i" ao lado do título de uma seção, página ou diálogo: tocar abre a explicação num balão.
// Explicações não ficam escritas na tela (E6); quando necessárias, vêm aqui. Quebras de linha valem.
defineProps<{ text: string }>()
const { t } = useI18n()
</script>

<template>
  <PopoverRoot>
    <PopoverTrigger class="rl-info" :aria-label="t('rl.info')">
      <Info :size="16" />
    </PopoverTrigger>
    <PopoverPortal>
      <PopoverContent class="rl-info-box" side="bottom" align="start" :side-offset="4" :collision-padding="12">
        {{ text }}
        <PopoverArrow class="fill-surface-3" />
      </PopoverContent>
    </PopoverPortal>
  </PopoverRoot>
</template>

<style scoped>
/* Ícone discreto; a área de toque é maior que ele, sem empurrar o título */
.rl-info {
  display: inline-grid;
  flex-shrink: 0;
  width: 32px;
  height: 32px;
  margin: -8px -6px;
  place-items: center;
  border-radius: 50%;
  color: var(--color-fg-muted);
}

.rl-info:hover,
.rl-info[data-state='open'] {
  color: var(--color-select);
}
</style>

<!-- Sem scoped: a Reka envolve o balão num elemento próprio e o atributo do scoped não chega nele -->
<style>
.rl-info-box {
  z-index: 60;
  max-width: min(20rem, calc(100vw - 24px));
  padding: 10px 12px;
  border: 1px solid #000;
  border-radius: 8px;
  background: var(--color-surface-3);
  box-shadow: 0 8px 24px rgb(0 0 0 / 0.6), inset 0 1px 0 rgb(255 255 255 / 0.08);
  color: var(--color-fg);
  font-size: 13px;
  font-weight: 400;
  letter-spacing: normal;
  line-height: 1.4;
  text-transform: none;
  white-space: pre-line;
}
</style>

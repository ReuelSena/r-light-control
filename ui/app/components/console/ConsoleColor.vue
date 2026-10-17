<script setup lang="ts">
import { PopoverContent, PopoverPortal, PopoverRoot, PopoverTrigger } from 'reka-ui'

// Cor de um elemento do console: o botão mostra a cor e abre a paleta (cada cor viva sobre a versão
// escura dela, e os cinzas). Com `none`, a paleta oferece "sem cor" (valor null).
defineProps<{ label: string; none?: boolean }>()
const color = defineModel<string | null>({ required: true })
const { t } = useI18n()
const open = ref(false)

function pick(value: string | null) {
  color.value = value
  open.value = false
}
</script>

<template>
  <div class="field-label">
    {{ label }}
    <PopoverRoot v-model:open="open">
      <PopoverTrigger class="trigger raised">
        <span class="chip" :class="{ empty: !color }" :style="{ background: color ?? undefined }" />
        <span class="font-lcd text-sm text-fg">{{ color ?? t('vc.noColor') }}</span>
      </PopoverTrigger>
      <PopoverPortal>
        <PopoverContent class="console-palette" side="bottom" align="start" :side-offset="4" :collision-padding="12">
          <div class="flex flex-wrap gap-1">
            <div v-for="pair in widgetColors.pairs" :key="pair[0]" class="flex flex-col gap-1">
              <button
                v-for="c in pair"
                :key="c"
                type="button"
                class="swatch"
                :class="{ active: c === color }"
                :style="{ background: c }"
                :aria-label="c"
                @click="pick(c)"
              />
            </div>
          </div>
          <div class="flex flex-wrap gap-1">
            <button
              v-for="c in widgetColors.grays"
              :key="c"
              type="button"
              class="swatch"
              :class="{ active: c === color }"
              :style="{ background: c }"
              :aria-label="c"
              @click="pick(c)"
            />
          </div>
          <RlButton v-if="none" size="sm" :model-value="!color" highlight="led" @press="pick(null)">{{ t('vc.noColor') }}</RlButton>
        </PopoverContent>
      </PopoverPortal>
    </PopoverRoot>
  </div>
</template>

<style scoped>
.trigger {
  display: flex;
  min-height: 44px;
  align-items: center;
  gap: 10px;
  padding: 0 10px;
  border-radius: 6px;
}

.chip {
  width: 24px;
  height: 24px;
  border: 1px solid #000;
  border-radius: 4px;
}

/* Sem cor: risco vermelho na diagonal */
.empty {
  background: linear-gradient(to top right, transparent calc(50% - 1px), var(--color-danger) 50%, transparent calc(50% + 1px)), #08090a;
}
</style>

<!-- Sem scoped: a Reka põe a paleta num elemento próprio, fora do componente -->
<style>
.console-palette {
  z-index: 60;
  display: flex;
  flex-direction: column;
  gap: 8px;
  max-width: calc(100vw - 24px);
  padding: 8px;
  border: 1px solid #000;
  border-radius: 8px;
  background: var(--color-surface-3);
  box-shadow: 0 8px 24px rgb(0 0 0 / 0.6), inset 0 1px 0 rgb(255 255 255 / 0.08);
}

.console-palette .swatch {
  width: 40px;
  height: 40px;
  border: 1px solid #000;
  border-radius: 6px;
  box-shadow: inset 0 1px 0 rgb(255 255 255 / 0.25);
}

.console-palette .swatch.active {
  outline: 2px solid var(--color-select);
  outline-offset: 1px;
}
</style>

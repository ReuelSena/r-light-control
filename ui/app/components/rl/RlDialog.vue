<script setup lang="ts">
import { DialogClose, DialogContent, DialogOverlay, DialogPortal, DialogRoot, DialogTitle } from 'reka-ui'
import { X } from 'lucide-vue-next'

// Diálogo modal. Com `sheet` vira painel: sobe de baixo no celular e entra pela direita em telas maiores.
// `info` põe o "i" com a explicação ao lado do título.
defineProps<{ title: string; sheet?: boolean; info?: string }>()
const open = defineModel<boolean>('open', { default: false })
const { t } = useI18n()
</script>

<template>
  <DialogRoot v-model:open="open">
    <DialogPortal>
      <DialogOverlay class="fixed inset-0 z-40 bg-black/60" />
      <DialogContent class="rl-dialog" :class="{ sheet }" :aria-describedby="undefined">
        <header class="brushed flex items-center justify-between gap-2 py-1.5 pr-1.5 pl-3">
          <span class="flex items-center gap-2">
            <DialogTitle class="text-sm font-semibold tracking-wide uppercase">{{ title }}</DialogTitle>
            <RlInfo v-if="info" :text="info" />
          </span>
          <DialogClose as-child>
            <RlButton size="sm" :aria-label="t('rl.close')"><X :size="18" /></RlButton>
          </DialogClose>
        </header>
        <div class="body">
          <slot />
        </div>
      </DialogContent>
    </DialogPortal>
  </DialogRoot>
</template>

<style scoped>
.rl-dialog {
  position: fixed;
  z-index: 50;
  left: 50%;
  top: 50%;
  translate: -50% -50%;
  display: flex;
  flex-direction: column;
  width: min(92vw, 400px);
  max-height: 90dvh;
  overflow: hidden;
  background: var(--color-surface-1);
  border: 1px solid #000;
  border-radius: 10px;
  box-shadow: 0 20px 50px rgb(0 0 0 / 0.7);
}

.body {
  padding: 12px;
  overflow: auto;
}

/* Celular: painel que sobe de baixo */
.sheet {
  left: 0;
  right: 0;
  top: auto;
  bottom: 0;
  translate: none;
  width: auto;
  max-height: 85dvh;
  border-radius: 12px 12px 0 0;
}

/* Telas maiores: painel lateral à direita */
@media (min-width: 640px) {
  .sheet {
    left: auto;
    top: 0;
    width: 340px;
    max-height: none;
    border-radius: 0;
  }
}
</style>

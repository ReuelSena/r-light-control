<script setup lang="ts">
// Tela que chega numa rodada seguinte do protótipo (F2).
// `screen` é texto (e não ScreenId) porque o registro de telas usa este componente.
defineProps<{ screen: string; round: number }>()
const { t } = useI18n()
</script>

<template>
  <div class="flex h-full flex-col items-center justify-center gap-3 p-6 text-center">
    <component :is="screens[screen as ScreenId].icon" :size="56" class="text-fg-dim" />
    <h1 class="text-xl font-bold tracking-wide uppercase">{{ t(`screens.${screen}`) }}</h1>
    <p class="text-fg-muted">{{ t('placeholder.round', { round }) }}</p>
  </div>
</template>

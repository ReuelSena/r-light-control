<script setup lang="ts">
// Teste de desempenho e multitoque (F1): 64 faders animando a 50 Hz.
// Desligue "Animar" para testar vários dedos ao mesmo tempo.
const { t } = useI18n()
const values = ref(Array.from({ length: 64 }, () => 0))
const animate = ref(true)
const fps = ref(0)
const touches = ref(0)
let timer: ReturnType<typeof setInterval> | undefined
let raf = 0

onMounted(() => {
  timer = setInterval(() => {
    if (!animate.value) return
    const time = performance.now() / 1000
    values.value = values.value.map((_, i) => (Math.sin(time * 2 + i * 0.3) + 1) / 2)
  }, 20)

  let frames = 0
  let last = performance.now()
  const count = (now: number) => {
    frames++
    if (now - last >= 1000) {
      fps.value = Math.round((frames * 1000) / (now - last))
      frames = 0
      last = now
    }
    raf = requestAnimationFrame(count)
  }
  raf = requestAnimationFrame(count)
})

onUnmounted(() => {
  clearInterval(timer)
  cancelAnimationFrame(raf)
})
</script>

<template>
  <div
    class="flex h-dvh flex-col"
    @pointerdown.capture="touches++"
    @pointerup.capture="touches--"
    @pointercancel.capture="touches--"
  >
    <header class="brushed flex items-center gap-2 px-3 py-1.5">
      <RlButton size="sm" @press="navigateTo('/dev/showcase')">{{ t('dev.back') }}</RlButton>
      <RlLcd>{{ t('dev.fps', { n: fps }) }}</RlLcd>
      <RlLcd>{{ t('dev.touches', { n: touches }) }}</RlLcd>
      <RlButton v-model="animate" class="ml-auto" mode="toggle" color="var(--color-on)">{{ t('dev.animate') }}</RlButton>
    </header>
    <div class="grid flex-1 grid-cols-[repeat(auto-fill,minmax(64px,1fr))] content-start gap-1 overflow-auto p-2">
      <RlFader
        v-for="(value, i) in values"
        :key="i"
        class="h-64"
        :model-value="value"
        @update:model-value="values[i] = $event"
      />
    </div>
  </div>
</template>

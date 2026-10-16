<script setup lang="ts">
// Janela externa (E13): uma tela sozinha, para levar a outro monitor. A cada segundo avisa a
// janela principal que continua aberta; quando o painel é trazido de volta, para de avisar.
const route = useRoute()
const { t } = useI18n()
const screen = computed<ScreenId>(() => (isScreen(route.params.screen) ? route.params.screen : 'console'))
const returned = ref(false)
useHead({ title: () => `${t(`screens.${screen.value}`)} · R Light Control` })

let timer: ReturnType<typeof setInterval> | undefined
let channel: BroadcastChannel | undefined

onMounted(() => {
  channel = new BroadcastChannel('rlc-windows')
  const beat = () => channel?.postMessage({ alive: screen.value })
  beat()
  timer = setInterval(beat, 1000)
  channel.onmessage = ({ data }) => {
    if (data.close !== screen.value) return
    clearInterval(timer)
    returned.value = true
    window.close()
  }
})

onUnmounted(() => {
  clearInterval(timer)
  channel?.close()
})
</script>

<template>
  <div class="flex h-dvh flex-col">
    <header class="brushed flex items-center gap-2 px-3 py-2 text-sm font-bold tracking-wide uppercase">
      <component :is="screens[screen].icon" :size="18" />
      {{ t(`screens.${screen}`) }}
    </header>
    <main class="@container min-h-0 flex-1 overflow-auto">
      <p v-if="returned" class="p-6 text-center text-fg-muted">{{ t('dock.returned') }}</p>
      <component :is="screens[screen].component" v-else />
    </main>
  </div>
</template>

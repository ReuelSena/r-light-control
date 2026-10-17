<script setup lang="ts">
import { Ellipsis, LayoutPanelLeft, Maximize, Minimize, OctagonX, Settings } from 'lucide-vue-next'

// Moldura do app: menus (E12) e, conforme a preferência deste aparelho, uma tela por vez
// ou o modo modular com painéis encaixáveis (E13).
const route = useRoute()
const { t } = useI18n()
const prefs = usePrefs()
const { state, send } = useEngine()
const wide = useMediaQuery('(min-width: 1024px)')

if (!isScreen(route.params.screen)) navigateTo('/console', { replace: true })
const screen = computed<ScreenId>(() => (isScreen(route.params.screen) ? route.params.screen : 'console'))
useHead({ title: () => `${t(`screens.${screen.value}`)} · R Light Control` })

const modular = computed(() => prefs.value.modular && wide.value)
const moreOpen = ref(false)
// Configurações num painel lateral sobre a tela atual (não é uma tela navegável)
const settingsOpen = ref(false)

// Tela cheia no navegador do celular e do tablet, como nos apps das mesas digitais (o iPhone não permite)
const canFullscreen = document.fullscreenEnabled && matchMedia('(pointer: coarse)').matches
const fullscreen = ref(!!document.fullscreenElement)
const onFullscreen = () => (fullscreen.value = !!document.fullscreenElement)
onMounted(() => document.addEventListener('fullscreenchange', onFullscreen))
onBeforeUnmount(() => document.removeEventListener('fullscreenchange', onFullscreen))
function toggleFullscreen() {
  if (document.fullscreenElement) document.exitFullscreen().catch(() => {})
  else document.documentElement.requestFullscreen({ navigationUI: 'hide' }).catch(() => {})
}

// Pânico: um toque para as criações; o segundo toque logo em seguida também zera a mesa simples.
// O LED fica aceso enquanto houver algo para parar.
const busy = computed(() => state.value.running.length + state.value.desk.levels.length + state.value.desk.groups.length > 0)
let lastStop = 0
function stop() {
  const double = Date.now() - lastStop < 400
  lastStop = Date.now()
  send({ op: double ? 'panic' : 'stopAllFunctions' })
}

// Verde quando todo universo com saída tem a interface conectada
const usbOk = computed(() =>
  state.value.project.universes.every((u) => !u.output || state.value.interfaces.find((i) => i.id === u.output)?.connected),
)

function go(id: ScreenId) {
  moreOpen.value = false
  navigateTo(`/${id}`)
}
</script>

<template>
  <RlAppShell>
    <template #toolbar>
      <!-- Nome e texto do USB só em telas largas, para a barra caber em 1280 px -->
      <span class="brand max-2xl:hidden!">R·LIGHT</span>
      <AppProjectMenu />
    </template>

    <template #nav>
      <RlButton v-for="id in navScreens" :key="id" :model-value="screen === id" @press="go(id)">
        <component :is="screens[id].icon" :size="16" />
        {{ t(`screens.${id}`) }}
      </RlButton>
    </template>

    <template #toolbar-end>
      <RlButton
        size="sm"
        highlight="led"
        color="var(--color-danger)"
        :model-value="busy"
        :title="t('shell.stop')"
        :aria-label="t('shell.stop')"
        @press="stop"
      >
        <OctagonX :size="18" />
      </RlButton>
      <RlLcd class="max-2xl:hidden">{{ state.master.bpm }} BPM</RlLcd>
      <span class="flex items-center gap-1.5 text-xs font-bold" :title="usbOk ? t('shell.usbOk') : t('shell.usbFail')">
        <span class="rl-led" :style="{ '--led': usbOk ? 'var(--color-on)' : 'var(--color-danger)' }" />
        <span class="max-2xl:hidden">USB</span>
      </span>
      <RlButton
        v-if="wide"
        v-model="prefs.modular"
        mode="toggle"
        highlight="led"
        size="sm"
        :title="t('shell.modular')"
        :aria-label="t('shell.modular')"
      >
        <LayoutPanelLeft :size="18" />
      </RlButton>
      <RlButton
        v-if="canFullscreen"
        size="sm"
        highlight="led"
        :model-value="fullscreen"
        :aria-label="t(fullscreen ? 'shell.exitFullscreen' : 'shell.fullscreen')"
        @press="toggleFullscreen"
      >
        <component :is="fullscreen ? Minimize : Maximize" :size="18" />
      </RlButton>
      <RlButton size="sm" :model-value="settingsOpen" :aria-label="t('screens.settings')" :title="t('screens.settings')" @press="settingsOpen = true">
        <Settings :size="18" />
      </RlButton>
    </template>

    <template #nav-bottom>
      <RlButton v-for="id in phoneScreens" :key="id" :model-value="screen === id" @press="go(id)">
        <component :is="screens[id].icon" :size="20" />
        {{ t(`screens.${id}`) }}
      </RlButton>
      <RlButton :model-value="moreScreens.includes(screen)" @press="moreOpen = true">
        <Ellipsis :size="20" />
        {{ t('shell.more') }}
      </RlButton>
    </template>

    <AppDock v-if="modular" />
    <!-- @container: as telas se adaptam à largura do espaço que ocupam, não à da janela -->
    <div v-else class="@container h-full">
      <component :is="screens[screen].component" />
    </div>

    <RlDialog v-model:open="moreOpen" sheet :title="t('shell.more')">
      <div class="grid gap-2">
        <RlButton v-for="id in moreScreens" :key="id" size="lg" :model-value="screen === id" @press="go(id)">
          <component :is="screens[id].icon" :size="20" />
          {{ t(`screens.${id}`) }}
        </RlButton>
      </div>
    </RlDialog>

    <AppSettings v-model:open="settingsOpen" />
  </RlAppShell>
</template>

<style scoped>
.brand {
  font-weight: 800;
  letter-spacing: 0.14em;
  text-shadow: 0 1px 0 #000;
  white-space: nowrap;
}
</style>

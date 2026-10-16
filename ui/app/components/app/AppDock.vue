<script setup lang="ts">
import { DockviewVue, themeDark, type DockviewApi, type DockviewReadyEvent } from 'dockview-vue'
import 'dockview-vue/dist/styles/dockview.css'
import { ExternalLink } from 'lucide-vue-next'

// Modo modular (E13): cada tela é um painel que vira aba, se encaixa, redimensiona e flutua.
// O layout fica salvo neste aparelho. A rota acompanha o painel ativo, então os menus
// continuam acesos no lugar certo. "Abrir em janela" leva o painel para uma janela externa.
const LAYOUT = 'rlc.layout'
const route = useRoute()
const router = useRouter()
const { t } = useI18n()
let api: DockviewApi | undefined

function open(id: ScreenId) {
  const panel = api?.getPanel(id)
  if (panel) panel.api.setActive()
  else api?.addPanel({ id, component: 'screen', title: t(`screens.${id}`), params: { screen: id } })
}

function onReady(event: DockviewReadyEvent) {
  api = event.api
  try {
    api.fromJSON(JSON.parse(localStorage.getItem(LAYOUT) ?? ''))
  } catch {
    // Sem layout salvo: Fixtures e Patch lado a lado
    open('fixtures')
    api.addPanel({ id: 'patch', component: 'screen', title: t('screens.patch'), params: { screen: 'patch' }, position: { referencePanel: 'fixtures', direction: 'right' } })
  }
  if (isScreen(route.params.screen)) open(route.params.screen)
  api.onDidLayoutChange(() => {
    try {
      localStorage.setItem(LAYOUT, JSON.stringify(api!.toJSON()))
    } catch {
      // Sem armazenamento: o layout vale só nesta sessão
    }
  })
  api.onDidActivePanelChange(({ panel }) => {
    if (panel) router.replace(`/${panel.id}`)
  })
}

watch(() => route.params.screen, (id) => isScreen(id) && open(id))

// ---------- Janelas externas ----------
// Cada janela externa avisa a cada segundo; sem aviso por 3 s, o painel volta para o dock.
const external = ref(new Map<ScreenId, number>())
let channel: BroadcastChannel | undefined
let timer: ReturnType<typeof setInterval> | undefined

function popout(id: ScreenId) {
  window.open(`/janela/${id}`, `rlc-${id}`, 'popup,width=1100,height=750')
  external.value.set(id, Date.now())
}

function bringBack(id: ScreenId) {
  external.value.delete(id)
  channel?.postMessage({ close: id })
}

onMounted(() => {
  channel = new BroadcastChannel('rlc-windows')
  channel.onmessage = ({ data }) => {
    const id: unknown = data.alive
    if (isScreen(id)) external.value.set(id, Date.now())
  }
  timer = setInterval(() => {
    for (const [id, seen] of external.value) {
      if (Date.now() - seen > 3000) external.value.delete(id)
    }
  }, 1000)
})

onUnmounted(() => {
  clearInterval(timer)
  channel?.close()
})
</script>

<template>
  <DockviewVue class="rl-dock h-full" :theme="themeDark" @ready="onReady">
    <template #panel-screen="{ params }">
      <div v-if="external.has(params.screen)" class="flex h-full flex-col items-center justify-center gap-3 p-6 text-center">
        <ExternalLink :size="40" class="text-fg-dim" />
        <p class="text-fg-muted">{{ t('dock.external') }}</p>
        <RlButton @press="bringBack(params.screen)">{{ t('dock.bringBack') }}</RlButton>
      </div>
      <div v-else class="@container h-full overflow-auto">
        <component :is="screens[params.screen as ScreenId].component" />
      </div>
    </template>
    <template #rightHeaderActions="{ activePanel }">
      <button
        v-if="activePanel && isScreen(activePanel.id)"
        type="button"
        class="popout"
        :title="t('dock.popout')"
        :aria-label="t('dock.popout')"
        @click="popout(activePanel.id)"
      >
        <ExternalLink :size="15" />
      </button>
    </template>
  </DockviewVue>
</template>

<style>
/* Tema do dockview no padrão do design system */
.rl-dock .dockview-theme-dark {
  --dv-group-view-background-color: var(--color-surface-0);
  --dv-tabs-and-actions-container-background-color: #0b0c0e;
  --dv-tabs-and-actions-container-height: 36px;
  --dv-tab-font-size: 12px;
  --dv-activegroup-visiblepanel-tab-background-color: var(--color-surface-2);
  --dv-activegroup-visiblepanel-tab-color: var(--color-fg);
  --dv-activegroup-hiddenpanel-tab-background-color: transparent;
  --dv-activegroup-hiddenpanel-tab-color: var(--color-fg-muted);
  --dv-inactivegroup-visiblepanel-tab-background-color: var(--color-surface-1);
  --dv-inactivegroup-visiblepanel-tab-color: var(--color-fg-muted);
  --dv-inactivegroup-hiddenpanel-tab-background-color: transparent;
  --dv-inactivegroup-hiddenpanel-tab-color: var(--color-fg-dim);
  --dv-tab-divider-color: #000;
  --dv-separator-border: #000;
  --dv-sash-color: #000;
  --dv-active-sash-color: var(--color-select);
  --dv-drag-over-background-color: color-mix(in srgb, var(--color-select) 25%, transparent);
  --dv-drag-over-border-color: var(--color-select);
  --dv-paneview-active-outline-color: var(--color-select);
  --dv-floating-box-shadow: 0 20px 50px rgb(0 0 0 / 0.7);
}

.rl-dock .dv-tab {
  font-weight: 600;
  letter-spacing: 0.05em;
  text-transform: uppercase;
}

/* Aba ativa do grupo em foco ganha o filete laranja da seleção */
.rl-dock .dv-active-group .dv-tab.dv-active-tab {
  box-shadow: inset 0 -2px 0 var(--color-select);
}

.rl-dock .popout {
  display: grid;
  place-items: center;
  width: 32px;
  height: 100%;
  color: var(--color-fg-muted);
}

.rl-dock .popout:hover {
  color: var(--color-fg);
}
</style>

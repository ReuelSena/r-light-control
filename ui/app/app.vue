<script setup lang="ts">
type Tauri = { invoke: (cmd: string, args: object) => Promise<unknown> }
const prefs = usePrefs()
const route = useRoute()
const { t } = useI18n()
const { connected, error } = useEngine()

// Zoom geral (⚙), sempre um zoom de verdade, que mantém medidas, toques, arrastos e balões coerentes
// (o `zoom` do CSS desalinha os balões):
// - no app desktop, o zoom nativo da janela;
// - no celular e no tablet, a largura do viewport: o navegador encaixa a página na tela na escala escolhida;
// - no navegador do computador, vale o zoom do próprio navegador.
const tauri = (window as { __TAURI_INTERNALS__?: Tauri }).__TAURI_INTERNALS__
if (!tauri && matchMedia('(pointer: coarse)').matches) {
  const original = document.querySelector<HTMLMetaElement>('meta[name=viewport]')!.content
  const landscape = useMediaQuery('(orientation: landscape)')
  // No celular (tela menor que 640 px), o 100% é 75% da escala do navegador: na escala dele, a interface fica grande demais
  const base = Math.min(screen.width, screen.height) < 640 ? 0.75 : 1
  // Pelo useHead, para o Nuxt não repor o viewport original quando atualiza o <head>
  useHead({
    meta: [{
      name: 'viewport',
      content: computed(() => {
        const zoom = Math.round(prefs.value.uiZoom * base * 1000) / 1000
        // A largura da tela na orientação atual (no iPhone, `screen` fica sempre em retrato)
        const device = (landscape.value ? Math.max : Math.min)(screen.width, screen.height)
        return zoom === 1
          ? original
          : `width=${Math.round(device / zoom)}, initial-scale=${zoom}, minimum-scale=${zoom}, maximum-scale=${zoom}, user-scalable=no, viewport-fit=cover`
      }),
    }],
  })
}

onMounted(() => {
  // Toque longo não abre menu de contexto (exceto em campos de texto).
  document.addEventListener('contextmenu', (e) => {
    if (!(e.target as HTMLElement).closest('input, textarea')) e.preventDefault()
  })
  if (tauri) watchEffect(() => tauri.invoke('plugin:webview|set_webview_zoom', { value: prefs.value.uiZoom }).catch(() => {}))

  // Atalhos padrão: Ctrl + − e Ctrl + + (fileira de cima ou teclado numérico) mudam o zoom geral; com o
  // console na frente (tela aberta, painel ativo do modular ou janela externa dele), o zoom do console.
  window.addEventListener('keydown', (e) => {
    if (!e.ctrlKey || e.altKey) return
    const delta = e.key === '-' || e.code === 'NumpadSubtract' ? -1 : ['+', '='].includes(e.key) || e.code === 'NumpadAdd' ? 1 : 0
    // No navegador, fora do console, fica o zoom do próprio navegador
    const target = route.params.screen === 'console' ? 'consoleZoom' : tauri ? 'uiZoom' : undefined
    if (!delta || !target) return
    e.preventDefault()
    prefs.value[target] = stepValue(zoomSteps, prefs.value[target], delta)
  })
})
</script>

<template>
  <NuxtPage />
  <!-- Sem conexão, as telas ficam cobertas (sem toque) até o servidor voltar -->
  <div v-if="!connected" class="fixed inset-0 z-[60] grid place-items-center bg-black/70 p-4">
    <p class="panel text-center text-flash">{{ t('connection.lost') }}</p>
  </div>
  <p
    v-if="error"
    class="panel fixed inset-x-0 bottom-24 z-[60] mx-auto w-fit max-w-[90vw] border-danger! text-sm text-fg"
    role="alert"
  >
    {{ error }}
  </p>
</template>

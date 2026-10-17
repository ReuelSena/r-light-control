<script setup lang="ts">
type Tauri = { invoke: (cmd: string, args: object) => Promise<unknown> }
const prefs = usePrefs()
const route = useRoute()
const landscape = useMediaQuery('(orientation: landscape)')

onMounted(() => {
  // Toque longo não abre menu de contexto (exceto em campos de texto).
  document.addEventListener('contextmenu', (e) => {
    if (!(e.target as HTMLElement).closest('input, textarea')) e.preventDefault()
  })
  // Zoom geral (⚙), sempre um zoom de verdade, que mantém medidas, toques, arrastos e balões coerentes
  // (o `zoom` do CSS desalinha os balões):
  // - no app desktop, o zoom nativo da janela;
  // - no celular e no tablet, a largura do viewport: o navegador encaixa a página na tela na escala escolhida;
  // - no navegador do computador, vale o zoom do próprio navegador.
  const tauri = (window as { __TAURI_INTERNALS__?: Tauri }).__TAURI_INTERNALS__
  if (tauri) watchEffect(() => tauri.invoke('plugin:webview|set_webview_zoom', { value: prefs.value.uiZoom }).catch(() => {}))
  else if (matchMedia('(pointer: coarse)').matches) {
    const meta = document.querySelector<HTMLMetaElement>('meta[name=viewport]')!
    const original = meta.content
    watchEffect(() => {
      const zoom = prefs.value.uiZoom
      // A largura da tela na orientação atual (no iPhone, `screen` fica sempre em retrato)
      const device = (landscape.value ? Math.max : Math.min)(screen.width, screen.height)
      meta.content = zoom === 1
        ? original
        : `width=${Math.round(device / zoom)}, initial-scale=${zoom}, minimum-scale=${zoom}, maximum-scale=${zoom}, user-scalable=no, viewport-fit=cover`
    })
  }

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
</template>

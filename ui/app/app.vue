<script setup lang="ts">
type Tauri = { invoke: (cmd: string, args: object) => Promise<unknown> }
const prefs = usePrefs()
const route = useRoute()

onMounted(() => {
  // Toque longo não abre menu de contexto (exceto em campos de texto).
  document.addEventListener('contextmenu', (e) => {
    if (!(e.target as HTMLElement).closest('input, textarea')) e.preventDefault()
  })
  // Zoom geral (⚙): no app desktop, o zoom nativo da janela, que mantém medidas, arrastos e balões
  // coerentes (o `zoom` do CSS desalinha os balões). No navegador, vale o zoom do próprio navegador.
  const tauri = (window as { __TAURI_INTERNALS__?: Tauri }).__TAURI_INTERNALS__
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
</template>

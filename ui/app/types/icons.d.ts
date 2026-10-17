// Caminho interno do lucide com todos os ícones (não tem tipos próprios). Usado pela biblioteca de
// ícones dos botões do console (useIconLibrary).
declare module 'lucide-vue-next/dist/esm/icons/index.js' {
  import type { Component } from 'vue'

  const icons: Record<string, Component>
  export = icons
}

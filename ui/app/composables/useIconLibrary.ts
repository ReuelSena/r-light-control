import type { Component } from 'vue'

// Biblioteca completa de ícones (lucide, ~1.700) para os botões do console. Fica num pedaço separado
// do app e só é baixada quando algum botão usa ícone ou a escolha de ícone é aberta.
const library = shallowRef<Record<string, Component>>()
let loading: Promise<unknown> | undefined

export function useIconLibrary() {
  const load = () =>
    (loading ??= import('lucide-vue-next/dist/esm/icons/index.js').then((m) => (library.value = m as unknown as Record<string, Component>)))
  /** Componente do ícone pelo nome (ex.: "Camera"); vazio enquanto a biblioteca carrega. */
  const icon = (name: string | null) => {
    if (!name) return undefined
    load()
    return library.value?.[name]
  }
  return { library, load, icon }
}

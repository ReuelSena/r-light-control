import type { Component } from 'vue'
import { Activity, Box, Cable, Clapperboard, Lamp, SlidersHorizontal, SlidersVertical, Usb } from 'lucide-vue-next'

// Telas do app (E12). A mesma tela aparece como página (modo simples), painel (modo modular)
// ou janela externa (E13). As Configurações não são tela: ficam num painel lateral aberto pelo ⚙.

export const screens = {
  console: { icon: SlidersVertical, component: defineAsyncComponent(() => import('~/screens/ConsoleScreen.vue')) },
  fixtures: { icon: Lamp, component: defineAsyncComponent(() => import('~/screens/FixturesScreen.vue')) },
  studio: { icon: Clapperboard, component: defineAsyncComponent(() => import('~/screens/StudioScreen.vue')) },
  patch: { icon: Cable, component: defineAsyncComponent(() => import('~/screens/PatchScreen.vue')) },
  view3d: { icon: Box, component: defineAsyncComponent(() => import('~/screens/View3dScreen.vue')) },
  desk: { icon: SlidersHorizontal, component: defineAsyncComponent(() => import('~/screens/DeskScreen.vue')) },
  monitor: { icon: Activity, component: defineAsyncComponent(() => import('~/screens/MonitorScreen.vue')) },
  io: { icon: Usb, component: defineAsyncComponent(() => import('~/screens/InOutScreen.vue')) },
} satisfies Record<string, { icon: Component; component: Component }>

export type ScreenId = keyof typeof screens

export const isScreen = (id: unknown): id is ScreenId => typeof id === 'string' && id in screens

/** Barra superior (o ⚙ fica à parte). */
export const navScreens: ScreenId[] = ['console', 'fixtures', 'studio', 'patch', 'view3d', 'desk', 'monitor', 'io']
/** Celular: quatro na barra inferior e o resto em "Mais". */
export const phoneScreens: ScreenId[] = ['console', 'studio', 'desk', 'fixtures']
export const moreScreens: ScreenId[] = ['patch', 'view3d', 'monitor', 'io']

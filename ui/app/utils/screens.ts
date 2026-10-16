import type { Component } from 'vue'
import { Activity, Box, Cable, Clapperboard, Lamp, Settings, SlidersHorizontal, SlidersVertical, Usb } from 'lucide-vue-next'
import PlaceholderScreen from '~/screens/PlaceholderScreen.vue'

// Telas do app (E12). A mesma tela aparece como página (modo simples), painel (modo modular)
// ou janela externa (E13).

/** Tela que chega numa rodada seguinte do protótipo (F2). */
const later = (screen: string, round: number) => () => h(PlaceholderScreen, { screen, round })

export const screens = {
  console: { icon: SlidersVertical, component: later('console', 3) },
  fixtures: { icon: Lamp, component: defineAsyncComponent(() => import('~/screens/FixturesScreen.vue')) },
  studio: { icon: Clapperboard, component: later('studio', 2) },
  patch: { icon: Cable, component: defineAsyncComponent(() => import('~/screens/PatchScreen.vue')) },
  view3d: { icon: Box, component: later('view3d', 3) },
  desk: { icon: SlidersHorizontal, component: later('desk', 2) },
  monitor: { icon: Activity, component: later('monitor', 2) },
  io: { icon: Usb, component: defineAsyncComponent(() => import('~/screens/InOutScreen.vue')) },
  settings: { icon: Settings, component: defineAsyncComponent(() => import('~/screens/SettingsScreen.vue')) },
} satisfies Record<string, { icon: Component; component: Component }>

export type ScreenId = keyof typeof screens

export const isScreen = (id: unknown): id is ScreenId => typeof id === 'string' && id in screens

/** Barra superior (o ⚙ fica à parte). */
export const navScreens: ScreenId[] = ['console', 'fixtures', 'studio', 'patch', 'view3d', 'desk', 'monitor', 'io']
/** Celular: quatro na barra inferior e o resto em "Mais". */
export const phoneScreens: ScreenId[] = ['console', 'studio', 'desk', 'fixtures']
export const moreScreens: ScreenId[] = ['patch', 'view3d', 'monitor', 'io', 'settings']

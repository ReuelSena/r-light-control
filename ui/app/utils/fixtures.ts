import type { Component } from 'vue'
import { Aperture, CircleDot, Disc, Focus, Gauge, Move, Palette, SlidersVertical, Sparkles, Sun, Triangle, Wrench, Zap, ZoomIn } from 'lucide-vue-next'
import type { Attribute } from '~/types/generated/Attribute'
import type { ChannelGroup } from '~/types/generated/ChannelGroup'
import type { Fixture } from '~/types/generated/Fixture'
import type { Project } from '~/types/generated/Project'

/** Cores de etiqueta oferecidas para fixtures (vivas, como as cores semânticas). */
export const tagColors = ['#ff0000', '#ff8000', '#ffff00', '#00ff00', '#00ffff', '#00bfff', '#0000ff', '#8000ff', '#ff00aa', '#ffffff']

/** Ícone e cor de cada atributo. Atributos da mesma família dividem o ícone (as cores primárias são todas paleta). */
export const attributeLook: Record<Attribute, { icon: Component; color: string }> = {
  dimmer: { icon: Sun, color: 'var(--color-intensity)' },
  red: { icon: Palette, color: '#ff0000' },
  green: { icon: Palette, color: '#00ff00' },
  blue: { icon: Palette, color: '#0000ff' },
  white: { icon: Palette, color: '#ffffff' },
  amber: { icon: Palette, color: '#ffbf00' },
  uv: { icon: Palette, color: '#8000ff' },
  cyan: { icon: Palette, color: '#00ffff' },
  magenta: { icon: Palette, color: '#ff00ff' },
  yellow: { icon: Palette, color: '#ffff00' },
  colorWheel: { icon: Palette, color: 'var(--color-fg)' },
  colorMacro: { icon: Palette, color: 'var(--color-fg)' },
  pan: { icon: Move, color: 'var(--color-position)' },
  tilt: { icon: Move, color: 'var(--color-position)' },
  speed: { icon: Gauge, color: 'var(--color-speed)' },
  shutter: { icon: Zap, color: 'var(--color-beam)' },
  gobo: { icon: Disc, color: 'var(--color-beam)' },
  goboRotation: { icon: Disc, color: 'var(--color-beam)' },
  prism: { icon: Triangle, color: 'var(--color-beam)' },
  focus: { icon: Focus, color: 'var(--color-beam)' },
  zoom: { icon: ZoomIn, color: 'var(--color-beam)' },
  iris: { icon: Aperture, color: 'var(--color-beam)' },
  effect: { icon: Sparkles, color: 'var(--color-fg)' },
  maintenance: { icon: Wrench, color: 'var(--color-fg-muted)' },
  other: { icon: CircleDot, color: 'var(--color-fg-muted)' },
}

export const findDef = (project: Project, defId: string) => project.fixtureDefs.find((d) => d.id === defId)

/** Canais da fixture no modo dela, na ordem dos endereços. */
export function modeChannels(project: Project, fixture: Fixture) {
  const def = findDef(project, fixture.defId)
  return def?.modes.find((m) => m.name === fixture.mode)?.channels.map((i) => def.channels[i]!) ?? []
}

/** Ícone de um grupo de canais: o dos atributos, se todos dividem o mesmo; senão (ou vazio), um genérico. */
export function channelGroupLook(project: Project, group: ChannelGroup) {
  const looks = group.channels.map((c) => {
    const fixture = project.fixtures.find((f) => f.id === c.fixture)
    return attributeLook[(fixture && modeChannels(project, fixture)[c.channel]?.attribute) || 'other']
  })
  const first = looks[0]
  if (!first || looks.some((l) => l.icon !== first.icon)) return { icon: SlidersVertical, color: 'var(--color-fg-muted)' }
  return { icon: first.icon, color: looks.every((l) => l.color === first.color) ? first.color : 'var(--color-fg)' }
}

/** Quantos canais DMX a fixture ocupa no modo escolhido. */
export function channelCount(project: Project, fixture: Fixture) {
  return findDef(project, fixture.defId)?.modes.find((m) => m.name === fixture.mode)?.channels.length ?? 0
}

/** Fixtures que disputam algum endereço com outra no mesmo universo. */
export function conflicts(project: Project) {
  const owner = new Map<string, string>()
  const result = new Set<string>()
  for (const fixture of project.fixtures) {
    for (let c = fixture.address; c < fixture.address + channelCount(project, fixture); c++) {
      const key = `${fixture.universe}:${c}`
      const other = owner.get(key)
      if (other) {
        result.add(other)
        result.add(fixture.id)
      } else {
        owner.set(key, fixture.id)
      }
    }
  }
  return result
}

/** Primeiro endereço com `count` canais livres seguidos no universo (0 se não couber). */
export function nextFreeAddress(project: Project, universe: string, count: number) {
  const used = new Set<number>()
  for (const fixture of project.fixtures) {
    if (fixture.universe !== universe) continue
    for (let c = fixture.address; c < fixture.address + channelCount(project, fixture); c++) used.add(c)
  }
  for (let start = 1; start + count - 1 <= 512; start++) {
    let free = true
    for (let c = start; c < start + count && free; c++) free = !used.has(c)
    if (free) return start
  }
  return 0
}

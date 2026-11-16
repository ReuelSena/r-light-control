import type { BeamLook } from '~/types/generated/BeamLook'
import type { Capability } from '~/types/generated/Capability'
import type { FixtureDef } from '~/types/generated/FixtureDef'

// Aparência do feixe na 3D View (F15b): o padrão de cada definição (pelo tipo e pela lente), o ajuste gravado
// na definição (`FixtureDef.beam`, na Biblioteca) e o estado ao vivo (zoom, foco e shutter) a partir dos valores dos canais.
// Só prévia: não muda a luz.

/** O padrão da fixture: banho (PAR, dimmer) ou raio (moving head, scanner); os outros tipos não têm cone (`undefined`). */
export function defaultLook(def: FixtureDef): BeamLook | undefined {
  const { lensMinDeg: min, lensMaxDeg: max } = def.physical
  if (def.kind === 'colorChanger' || def.kind === 'dimmer') {
    return { brightness: 1, angleMin: Math.max(min || 25, 25), angleMax: Math.max(max || 40, 40), softness: 0.9, core: 0.4, reach: 6 }
  }
  // O strobo é um holofote largo
  if (def.kind === 'strobe') return { brightness: 1, angleMin: 50, angleMax: 70, softness: 0.9, core: 0.4, reach: 6 }
  if (def.kind !== 'movingHead' && def.kind !== 'scanner') return undefined
  // A definição do QLC+ nem sempre é fiel: o raio não passa de 4° a 10°
  const angleMin = Math.max(1, Math.min(min || 4, 4))
  return { brightness: 1, angleMin, angleMax: Math.max(angleMin, Math.min(max || 10, 10)), softness: 0.15, core: 1, reach: 12 }
}

/** A aparência do feixe da definição: a da biblioteca (`def.beam`) ou o padrão; `undefined` sem cone. */
export const beamLook = (def: FixtureDef | undefined) => def && (def.beam ?? defaultLook(def))

const clamp = (v: number, min = 0, max = 1) => Math.min(max, Math.max(min, v))

/**
 * Convenção do shutter: só pisca se o canal tem uma faixa de strobe (pelo nome da capability) e o valor está nela;
 * a frequência vai de 1 a 20 Hz ao longo da faixa. Sem faixa de strobe, fora dela ou com o valor 0, o shutter está aberto.
 * `now` é o relógio em segundos.
 */
const strobeName = /strobe|strobo|estrobo|pulse|flash|pisca/i

/** Ângulo (graus), suavidade da borda (0–1) e brilho do pisca (0 ou 1) do feixe, a partir dos valores dos canais. */
export function beamState(values: Partial<Record<string, number>>, look: BeamLook, shutter: Capability[] = [], now = 0) {
  // Sem canal de zoom, a média dos dois ângulos
  const angle = values.zoom === undefined ? (look.angleMin + look.angleMax) / 2 : look.angleMin + (look.angleMax - look.angleMin) * (values.zoom / 255)
  const softness = values.focus === undefined ? look.softness : clamp(look.softness + (values.focus / 255 - 0.5) * 0.6)
  // O valor 0 é o shutter aberto, mesmo quando a faixa de strobe da definição começa nele (ParLed: "Strobo 0–255")
  const strobe = !values.shutter ? undefined : shutter.find((c) => strobeName.test(c.name) && values.shutter! >= c.min && values.shutter! <= c.max)
  const hz = strobe ? 1 + (19 * (values.shutter! - Math.max(1, strobe.min))) / Math.max(1, strobe.max - Math.max(1, strobe.min)) : 0
  const flicker = hz && (now * hz) % 1 > 0.5 ? 0 : 1
  return { angle: Math.max(1, angle), softness, flicker }
}

// Prisma, gobo e rotação (R3): pelos nomes das capabilities, que variam entre fabricantes
const capAt = (caps: Capability[], value: number) => caps.find((c) => value >= c.min && value <= c.max)
const offName = /default|desligad|\boff\b|\bopen\b|abert|no function|sem |\bout\b|nenhum/i
const wheelName = /wheel|roda|scroll/i
const stopName = /stop|parad|position|posi[cç][aã]o|index/i
const ccwName = /left|esquerd|anti|counter|ccw|revers|invers/i
/** Canal (ou capability) de rotação, pelo nome. */
export const spinName = /rot|spin|gir/i

/** Facetas do prisma (0 = sem prisma) e a rotação lenta (rad/s) quando a própria capability fala em rotação. */
export function prismState(caps: Capability[], value: number) {
  const cap = capAt(caps, value)
  if (!value || !cap || offName.test(cap.name)) return { facets: 0, spin: 0 }
  // O número no nome que é uma quantidade de facetas ("Prisma 2 / 5 facetas" = 5); sem ele, 3
  const facets = cap.name.match(/\d+/g)?.map(Number).find((n) => [3, 4, 5, 6, 8].includes(n)) ?? 3
  return { facets, spin: spinName.test(cap.name) ? (ccwName.test(cap.name) ? -0.5 : 0.5) : 0 }
}

/** Velocidade de rotação (rad/s, negativa no anti-horário) pelo valor do canal; parada na faixa de posição. */
export function spinSpeed(caps: Capability[], value: number) {
  const cap = capAt(caps, value)
  if (!value || !cap || stopName.test(cap.name)) return 0
  const fraction = cap.max > cap.min ? (value - cap.min) / (cap.max - cap.min) : 1
  return (ccwName.test(cap.name) ? -1 : 1) * 2 * Math.PI * (0.1 + 0.9 * fraction)
}

/**
 * Figura do gobo (de 0 a `count - 1`) pela ordem das capabilities que são gobos; -1 = aberto (ou a roda girando).
 * A mesma figura tremendo ("Shake …") conta como ela.
 */
export function goboIndex(caps: Capability[], value: number, count: number) {
  const isGobo = (c: Capability) => !offName.test(c.name) && !wheelName.test(c.name)
  const base = (c: Capability) => c.name.replace(/shake\s*/i, '').trim().toLowerCase()
  const cap = capAt(caps, value)
  if (!cap || !isGobo(cap)) return -1
  return [...new Set(caps.filter(isGobo).map(base))].indexOf(base(cap)) % count
}

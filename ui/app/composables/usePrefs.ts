// Preferências deste aparelho (cada celular ou PC tem as suas), guardadas no navegador.
type Prefs = {
  valueFormat: 'percent' | 'dmx'
  modular: boolean
  /** Formato do console neste aparelho (VC-2); `auto` escolhe pela largura da tela. */
  consoleFormat: 'auto' | ConsoleFormat
  /** 3D View: qualidade do desenho e quanto a fumaça mostra os feixes (0–1). */
  quality: 'low' | 'medium' | 'high'
  smoke: number
  /** Zoom geral da interface (app desktop) e zoom da grade do console. */
  uiZoom: number
  consoleZoom: number
  /** Console: etiquetas com as teclas de atalho nos botões e elementos. */
  keyHints: boolean
  /** Localizar: intensidade das outras fixtures, de 0 a 1 (0 deixa só a localizada acesa). */
  locateOthers: number
}

/** Degraus de zoom, como os do navegador. */
export const zoomSteps = [0.5, 0.67, 0.75, 0.8, 0.9, 1, 1.1, 1.25, 1.5, 1.75, 2]

/** Próximo degrau de uma escala (`delta` −1 ou +1), a partir do degrau mais perto do valor. */
export function stepValue(steps: number[], value: number, delta: number) {
  const nearest = steps.reduce((best, s, i) => (Math.abs(s - value) < Math.abs(steps[best]! - value) ? i : best), 0)
  return steps[Math.min(steps.length - 1, Math.max(0, nearest + delta))]!
}

const KEY = 'rlc.prefs'
let saving = false

export function usePrefs() {
  const prefs = useState<Prefs>('prefs', () => {
    const defaults: Prefs = { valueFormat: 'percent', modular: false, consoleFormat: 'auto', quality: 'medium', smoke: 0.6, uiZoom: 1, consoleZoom: 1, keyHints: true, locateOthers: 0.2 }
    try {
      return { ...defaults, ...JSON.parse(localStorage.getItem(KEY) ?? '{}') }
    } catch {
      return defaults
    }
  })
  if (!saving) {
    saving = true
    watch(prefs, (value) => {
      try {
        localStorage.setItem(KEY, JSON.stringify(value))
      } catch {
        // Sem armazenamento (ex.: navegação privada): vale só nesta sessão.
      }
    }, { deep: true })
  }
  return prefs
}

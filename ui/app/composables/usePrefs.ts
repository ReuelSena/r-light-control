// Preferências deste aparelho (cada celular ou PC tem as suas), guardadas no navegador.
type Prefs = { valueFormat: 'percent' | 'dmx'; modular: boolean }

const KEY = 'rlc.prefs'
let saving = false

export function usePrefs() {
  const prefs = useState<Prefs>('prefs', () => {
    const defaults: Prefs = { valueFormat: 'percent', modular: false }
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

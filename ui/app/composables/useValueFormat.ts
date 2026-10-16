// Formato de exibição dos valores (E10): % por padrão, DMX (0–255) como opção deste aparelho.
export function useValueFormat() {
  const prefs = usePrefs()
  const mode = computed({
    get: () => prefs.value.valueFormat,
    set: (value) => (prefs.value.valueFormat = value),
  })
  const max = computed(() => (mode.value === 'dmx' ? 255 : 100))
  const format = (v: number) => (mode.value === 'dmx' ? String(Math.round(v * 255)) : `${Math.round(v * 100)}%`)
  return { mode, max, format }
}

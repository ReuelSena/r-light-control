// Formato de exibição dos valores (E10): % por padrão, DMX (0–255) como opção.
export function useValueFormat() {
  const mode = useState<'percent' | 'dmx'>('valueFormat', () => 'percent')
  const max = computed(() => (mode.value === 'dmx' ? 255 : 100))
  const format = (v: number) => (mode.value === 'dmx' ? String(Math.round(v * 255)) : `${Math.round(v * 100)}%`)
  return { mode, max, format }
}

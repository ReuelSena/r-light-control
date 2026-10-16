// Acompanha uma media query (ex.: '(min-width: 1024px)').
export function useMediaQuery(query: string) {
  const media = matchMedia(query)
  const matches = ref(media.matches)
  const update = () => (matches.value = media.matches)
  media.addEventListener('change', update)
  onScopeDispose(() => media.removeEventListener('change', update))
  return matches
}

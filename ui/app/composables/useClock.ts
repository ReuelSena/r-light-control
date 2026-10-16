/** Hora atual (ms), atualizada a cada `interval` ms enquanto o componente existe. Para prévias e passos ao vivo. */
export function useClock(interval = 50) {
  const now = ref(Date.now())
  const timer = setInterval(() => (now.value = Date.now()), interval)
  onScopeDispose(() => clearInterval(timer))
  return now
}

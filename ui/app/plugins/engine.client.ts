import { Refused } from '~/engine/ws'

// O app só abre com o estado do servidor em mãos (A3): até lá fica a tela "Conectando…"
// (spa-loading-template.html), e as telas nunca veem o estado vazio.
export default defineNuxtPlugin(async (nuxtApp) => {
  // Uma operação recusada interrompe quem esperava por ela; não é erro do app (o aviso já mostra o motivo)
  nuxtApp.vueApp.config.errorHandler = (error) => {
    if (!(error instanceof Refused)) console.error(error)
  }

  const { state } = useEngine()
  if (state.value) return
  await new Promise<void>((resolve) => {
    const stop = watch(state, () => {
      stop()
      resolve()
    })
  })
})

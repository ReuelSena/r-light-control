import type { Ref } from 'vue'

/**
 * "Localizar" (o highlight do QLC+): acende as fixtures escolhidas, acompanhando a escolha, até ser
 * desligado ou a tela sair, e deixa as outras com a intensidade escolhida neste aparelho. Apagado por
 * outro aparelho ou pelo pânico, o botão desliga junto.
 */
export function useLocate(ids: Ref<string[]>) {
  const { state, send } = useEngine()
  const prefs = usePrefs()
  const on = ref(false)
  const light = (fixtures: string[]) => send({ op: 'setHighlight', fixtures, others: Math.round(prefs.value.locateOthers * 255) })

  watch([() => ids.value.join(), () => prefs.value.locateOthers], () => on.value && light(ids.value))
  watch(() => state.value.highlight.fixtures.length, (n) => n || (on.value = false))
  onBeforeUnmount(() => on.value && light([]))

  return computed({
    get: () => on.value,
    set: (value) => {
      on.value = value
      light(value ? ids.value : [])
    },
  })
}

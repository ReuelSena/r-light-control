import type { Ref } from 'vue'

/**
 * "Localizar" (o highlight do QLC+): acende as fixtures escolhidas, acompanhando a escolha, até ser
 * desligado ou a tela sair. Apagado por outro aparelho ou pelo pânico, o botão desliga junto.
 */
export function useLocate(ids: Ref<string[]>) {
  const { state, send } = useEngine()
  const on = ref(false)
  const light = (fixtures: string[]) => send({ op: 'setHighlight', fixtures })

  watch(() => ids.value.join(), () => on.value && light(ids.value))
  watch(() => state.value.highlight.length, (n) => n || (on.value = false))
  onBeforeUnmount(() => on.value && light([]))

  return computed({
    get: () => on.value,
    set: (value) => {
      on.value = value
      light(value ? ids.value : [])
    },
  })
}

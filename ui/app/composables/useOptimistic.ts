import type { Ref } from 'vue'

/**
 * Valor otimista do controle sob o dedo (A2), para fader, knob e pad XY: o que a mão mexe aparece na
 * hora e o próximo movimento parte dele, sem esperar a volta do servidor. 300 ms depois do último
 * movimento, vale de novo o valor que veio do servidor.
 */
export function useOptimistic<T>(model: Ref<T>) {
  const local = shallowRef<{ value: T }>()
  let timer: ReturnType<typeof setTimeout> | undefined
  return computed({
    get: () => (local.value ? local.value.value : model.value),
    set: (value: T) => {
      local.value = { value }
      model.value = value
      clearTimeout(timer)
      timer = setTimeout(() => (local.value = undefined), 300)
    },
  })
}

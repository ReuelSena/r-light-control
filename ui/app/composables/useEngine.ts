import type { Ref, ShallowRef } from 'vue'
import type { Op } from '~/types/generated/Op'
import type { Snapshot } from '~/types/generated/Snapshot'
import { createMockClient } from '~/engine/mock'

// A única porta das telas para o motor (E2). Na F2 é o motor de mentira; na F3 vira o
// cliente WebSocket, sem mudar as telas.
export interface EngineClient {
  state: Ref<Snapshot>
  /** Valores de saída de cada universo (512 canais), por id. Do servidor, vêm em frames binários (A3). */
  output: ShallowRef<Record<string, Uint8Array>>
  send(op: Op): Promise<unknown>
}

let client: EngineClient | undefined

export function useEngine() {
  client ??= createMockClient()
  return client
}

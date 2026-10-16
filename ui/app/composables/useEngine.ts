import type { Ref } from 'vue'
import type { Op } from '~/types/generated/Op'
import type { Snapshot } from '~/types/generated/Snapshot'
import { createMockClient } from '~/engine/mock'

// A única porta das telas para o motor (E2). Na F2 é o motor de mentira; na F3 vira o
// cliente WebSocket, sem mudar as telas.
export interface EngineClient {
  state: Ref<Snapshot>
  send(op: Op): Promise<unknown>
}

let client: EngineClient | undefined

export function useEngine() {
  client ??= createMockClient()
  return client
}

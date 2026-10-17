import type { Ref, ShallowRef } from 'vue'
import type { Op } from '~/types/generated/Op'
import type { Snapshot } from '~/types/generated/Snapshot'
import { createWsClient } from '~/engine/ws'

// A única porta das telas para o motor (E2): o cliente WebSocket do servidor (A3).
export interface EngineClient {
  /** Estado completo, mantido em dia pelos avisos do servidor. */
  state: Ref<Snapshot>
  /** Valores de saída de cada universo (512 canais), por id. Do servidor, vêm em frames binários (A3). */
  output: ShallowRef<Record<string, Uint8Array>>
  /** Conectado e com o estado em dia. */
  connected: Ref<boolean>
  /** Id deste aparelho na lista dos conectados (`remote.clients`). */
  clientId: Ref<string>
  /** Motivo da última operação recusada, por alguns segundos. */
  error: Ref<string>
  /**
   * Manda uma operação; resolve com a resposta do servidor. Recusada, rejeita com `Refused`: o código
   * depois do `await` não roda (ex.: o diálogo continua aberto) e o motivo aparece no aviso do app.
   */
  send(op: Op): Promise<unknown>
}

let client: EngineClient | undefined

export function useEngine() {
  client ??= createWsClient()
  return client
}

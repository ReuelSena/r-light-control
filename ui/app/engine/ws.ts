import type { Op } from '~/types/generated/Op'
import type { Snapshot } from '~/types/generated/Snapshot'
import type { EngineClient } from '~/composables/useEngine'

// Cliente do servidor (A3, docs/protocolo.md): recebe o estado completo ao conectar, aplica cada
// aviso de mudança no pedaço do estado que ele leva e guarda os valores dos universos, que chegam em
// frames binários. A UI não repete as regras do motor: o estado só muda pelo que o servidor manda.
// Caindo a conexão, tenta de novo a cada segundo e recebe o estado completo ao voltar.

/** Operação recusada (ou sem conexão). O motivo já aparece no aviso do app (`error`). */
export class Refused extends Error {}

type ServerMessage =
  | { t: 'hello'; data: { clientId: string } }
  | { t: 'snapshot'; data: Snapshot }
  | { t: 'evt'; topic: string; data: unknown }
  | { t: 'ack'; id: number; data: unknown }
  | { t: 'err'; id: number | null; data: string }

export function createWsClient(): EngineClient {
  const { serverPort } = useRuntimeConfig().public
  // Em dev a página vem do servidor do Nuxt (porta 3000), mas o WebSocket é sempre o do Rust
  const url = import.meta.dev ? `ws://${location.hostname}:${serverPort}/ws` : `ws://${location.host}/ws`
  // Vazio até o primeiro estado: o app só mostra as telas depois dele (app.vue)
  const state = ref() as Ref<Snapshot>
  const connected = ref(false)
  const clientId = ref('')
  const error = ref('')
  const output = shallowRef<Record<string, Uint8Array>>({})
  const pending = new Map<number, { resolve: (data: unknown) => void; reject: (e: Error) => void }>()
  const decoder = new TextDecoder()
  let nextId = 1
  let socket: WebSocket

  // Quadros do motor sombra (o preparado), só enquanto assinado (VC-6)
  const previewOutput = shallowRef<Record<string, Uint8Array>>({})
  let previewOn = false

  // Os quadros de um mesmo envio entram juntos, uma vez por quadro de tela (tipo 1 = real, 2 = preview)
  const batches: Record<number, Record<string, Uint8Array> | undefined> = {}
  function frame(bytes: Uint8Array) {
    const kind = bytes[0]!
    if (kind !== 1 && kind !== 2) return
    const target = kind === 1 ? output : previewOutput
    const end = 2 + bytes[1]!
    let batch = batches[kind]
    if (!batch) {
      batch = batches[kind] = {}
      requestAnimationFrame(() => {
        target.value = { ...target.value, ...batches[kind] }
        batches[kind] = undefined
      })
    }
    batch[decoder.decode(bytes.subarray(2, end))] = bytes.slice(end, end + 512)
  }

  function subscribePreview(on: boolean) {
    previewOn = on
    if (!on) previewOutput.value = {}
    if (socket.readyState === WebSocket.OPEN) socket.send(JSON.stringify({ t: 'previewSub', on }))
  }

  function receive(message: ServerMessage) {
    switch (message.t) {
      case 'hello':
        clientId.value = message.data.clientId
        break
      case 'snapshot':
        state.value = message.data
        connected.value = true
        break
      // O tópico é o caminho do pedaço no estado (ex.: "project.fixtures")
      case 'evt': {
        const keys = message.topic.split('.')
        const parent = keys.slice(0, -1).reduce<Record<string, unknown>>((o, k) => o[k] as Record<string, unknown>, state.value as unknown as Record<string, unknown>)
        parent[keys.at(-1)!] = message.data
        break
      }
      case 'ack':
        pending.get(message.id)?.resolve(message.data)
        pending.delete(message.id)
        break
      case 'err':
        say(message.data)
        if (message.id === null) break
        pending.get(message.id)?.reject(new Refused(message.data))
        pending.delete(message.id)
    }
  }

  function say(text: string) {
    error.value = text
    setTimeout(() => error.value === text && (error.value = ''), 4000)
  }

  function connect() {
    socket = new WebSocket(url)
    socket.binaryType = 'arraybuffer'
    // Reconectou: a assinatura é por conexão, então vale de novo
    socket.onopen = () => previewOn && socket.send(JSON.stringify({ t: 'previewSub', on: true }))
    socket.onmessage = ({ data }) => (data instanceof ArrayBuffer ? frame(new Uint8Array(data)) : receive(JSON.parse(data)))
    socket.onclose = () => {
      connected.value = false
      for (const p of pending.values()) p.reject(new Refused('Sem conexão com o servidor.'))
      pending.clear()
      setTimeout(connect, 1000)
    }
  }
  connect()

  return {
    state,
    output,
    previewOutput,
    subscribePreview,
    connected,
    clientId,
    error,
    send(op: Op) {
      const reply = new Promise<unknown>((resolve, reject) => {
        if (!connected.value) return reject(new Refused('Sem conexão com o servidor.'))
        const id = nextId++
        pending.set(id, { resolve, reject })
        socket.send(JSON.stringify({ t: 'cmd', id, data: op }))
      })
      // Quem não espera a resposta não precisa tratar a recusa: o motivo já aparece no aviso do app
      reply.catch(() => {})
      return reply
    },
  }
}

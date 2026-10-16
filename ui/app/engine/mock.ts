import type { Attribute } from '~/types/generated/Attribute'
import type { ImportReport } from '~/types/generated/ImportReport'
import type { Op } from '~/types/generated/Op'
import type { Project } from '~/types/generated/Project'
import type { Snapshot } from '~/types/generated/Snapshot'
import type { EngineClient } from '~/composables/useEngine'
import { emptyProject, fakeSnapshot } from './fake'

// Motor de mentira da F2: aplica as operações no estado em memória, como o servidor fará (A2).
// Janelas do mesmo app ou navegador se sincronizam pelo BroadcastChannel; a que abre por
// último pede o estado às outras.

type Message = { op: Op } | { hello: true } | { snapshot: Snapshot }

const clone = <T>(value: T): T => JSON.parse(JSON.stringify(value))
// crypto.randomUUID não existe fora de contexto seguro (o celular acessa por http na rede local)
const newId = () => Math.random().toString(36).slice(2, 10)

export function createMockClient(): EngineClient {
  const state = ref(fakeSnapshot())
  const channel = new BroadcastChannel('rlc-mock')
  channel.onmessage = ({ data }: MessageEvent<Message>) => {
    if ('op' in data) apply(state.value, data.op)
    else if ('hello' in data) channel.postMessage({ snapshot: clone(state.value) })
    else state.value = data.snapshot
  }
  channel.postMessage({ hello: true })

  const output = shallowRef<Record<string, Uint8Array>>({})
  setInterval(() => (output.value = simulate(toRaw(state.value), Date.now())), 40)

  // Dados da máquina variando um pouco, como num computador de verdade
  const started = Date.now()
  setInterval(() => {
    const stats = state.value.stats
    stats.cpuPercent = Math.min(40, Math.max(2, stats.cpuPercent + (Math.random() - 0.5) * 4))
    stats.memoryMb = Math.round(140 + Math.random() * 6)
    stats.tickMs = 0.3 + Math.random() * 0.5
    stats.uptimeS = Math.round((Date.now() - started) / 1000)
  }, 1000)

  return {
    state,
    output,
    async send(input) {
      // Cópia: o estado nunca guarda objetos que a tela continua editando
      const op = clone(input)
      const reply = apply(state.value, op)
      channel.postMessage({ op })
      return reply
    },
  }
}

function apply(s: Snapshot, op: Op): ImportReport | undefined {
  const p = s.project
  switch (op.op) {
    case 'newProject':
      s.project = emptyProject()
      s.projectPath = null
      s.dirty = false
      s.running = []
      return
    case 'openProject':
      s.project = fakeSnapshot().project
      s.project.name = op.path.split('\\').pop()!.replace(/\.rlc$/, '')
      s.projectPath = op.path
      s.dirty = false
      return
    case 'saveProject':
      s.projectPath = op.path ?? s.projectPath ?? `C:\\Shows\\${p.name}.rlc`
      s.recentProjects = [s.projectPath, ...s.recentProjects.filter((path) => path !== s.projectPath)].slice(0, 8)
      s.dirty = false
      return
    case 'addFixtures': {
      const def = s.library.find((d) => d.id === op.defId) ?? findDef(p, op.defId)
      if (!def) throw new Error(`definição ${op.defId} não encontrada`)
      if (!findDef(p, def.id)) p.fixtureDefs.push(clone(def))
      const size = def.modes.find((m) => m.name === op.mode)?.channels.length ?? 1
      for (let i = 0; i < op.count; i++) {
        const address = op.address ? op.address + i * size : nextFreeAddress(p, op.universe, size) || 1
        const name = op.count > 1 ? `${op.name} ${i + 1}` : op.name
        p.fixtures.push({ id: newId(), defId: def.id, mode: op.mode, name, color: op.color, universe: op.universe, address })
      }
      break
    }
    case 'updateFixture':
      upsert(p.fixtures, op.fixture)
      break
    case 'removeFixtures':
      p.fixtures = p.fixtures.filter((f) => !op.ids.includes(f.id))
      for (const group of p.fixtureGroups) group.cells = group.cells.filter((c) => !op.ids.includes(c.fixture))
      for (const group of p.channelGroups) group.channels = group.channels.filter((c) => !op.ids.includes(c.fixture))
      for (const f of p.functions) {
        if (f.type === 'scene') f.values = f.values.filter((v) => !op.ids.includes(v.fixture))
        if (f.type === 'sequence') for (const step of f.steps) step.values = step.values.filter((v) => !op.ids.includes(v.fixture))
        if (f.type === 'efx') f.fixtures = f.fixtures.filter((x) => !op.ids.includes(x.fixture))
      }
      break
    case 'autoAddress': {
      let address = op.start
      for (const id of op.ids) {
        const fixture = p.fixtures.find((f) => f.id === id)
        if (!fixture) continue
        fixture.universe = op.universe
        fixture.address = address
        address += channelCount(p, fixture)
      }
      break
    }
    case 'saveFixtureGroup':
      upsert(p.fixtureGroups, op.group)
      break
    case 'removeFixtureGroup':
      p.fixtureGroups = p.fixtureGroups.filter((g) => g.id !== op.id)
      break
    case 'saveChannelGroup':
      upsert(p.channelGroups, op.group)
      break
    case 'removeChannelGroup':
      p.channelGroups = p.channelGroups.filter((g) => g.id !== op.id)
      break
    case 'saveFunction':
      upsert(p.functions, op.function)
      break
    case 'removeFunction':
      p.functions = p.functions.filter((f) => f.id !== op.id)
      for (const f of p.functions) {
        if (f.type === 'chaser') f.steps = f.steps.filter((step) => step.function !== op.id)
        if (f.type === 'collection') f.functions = f.functions.filter((id) => id !== op.id)
      }
      s.running = s.running.filter((id) => id !== op.id)
      break
    case 'saveFolder':
      upsert(p.folders, op.folder)
      break
    case 'removeFolder':
      p.folders = p.folders.filter((d) => d.id !== op.id)
      for (const f of p.functions) if (f.folder === op.id) f.folder = null
      break
    case 'setFunctionRunning':
      s.running = s.running.filter((id) => id !== op.id).concat(op.running ? [op.id] : [])
      return
    case 'stopAllFunctions':
      s.running = []
      return
    case 'panic':
      s.running = []
      s.desk = { levels: [], groups: [] }
      return
    case 'setDeskLevel':
      setDeskLevel(s, op.universe, op.channel, op.value)
      return
    // O grupo grava o próprio nível e o valor nos canais dele, que os faders de canal mostram
    case 'setDeskGroupLevel':
      s.desk.groups = s.desk.groups.filter((g) => g.group !== op.group).concat({ group: op.group, value: op.value })
      for (const [universe, channel] of groupAddresses(p, op.group)) setDeskLevel(s, universe, channel, op.value)
      return
    // Zerar os grupos também solta os canais deles
    case 'resetDesk': {
      if (op.universe) {
        s.desk.levels = s.desk.levels.filter((l) => l.universe !== op.universe)
        return
      }
      const released = new Set(s.desk.groups.flatMap((g) => groupAddresses(p, g.group).map(([u, c]) => `${u}:${c}`)))
      s.desk.levels = s.desk.levels.filter((l) => !released.has(`${l.universe}:${l.channel}`))
      s.desk.groups = []
      return
    }
    case 'saveFixtureDef':
      upsert(s.library, op.def)
      if (!findDef(p, op.def.id)) return
      upsert(p.fixtureDefs, clone(op.def))
      break
    case 'importQxf': {
      // A conversão de verdade é da F4; aqui só simulamos o resultado
      const def = { ...clone(s.library[0]!), id: `importado-${newId()}`, manufacturer: 'Importado do QLC+', model: op.fileName.replace(/\.qxf$/i, '') }
      s.library.push(def)
      return {
        defId: def.id,
        channels: def.channels.length,
        modes: def.modes.length,
        warnings: [
          'Capability "Strobe aleatório" sem equivalente: guardada como texto.',
          'Canal "Função especial" sem grupo conhecido: marcado como "outro".',
        ],
      }
    }
    case 'addUniverse':
      p.universes.push({ id: newId(), name: `Universo ${p.universes.length + 1}`, output: null })
      break
    case 'updateUniverse':
      upsert(p.universes, op.universe)
      break
    case 'removeUniverse':
      p.universes = p.universes.filter((u) => u.id !== op.id)
      p.fixtures = p.fixtures.filter((f) => f.universe !== op.id)
      break
    case 'setRemoteAccess':
      s.remote.enabled = op.enabled
      s.remote.pin = op.pin
      return
  }
  s.dirty = true
}

function setDeskLevel(s: Snapshot, universe: string, channel: number, value: number) {
  s.desk.levels = s.desk.levels.filter((l) => l.universe !== universe || l.channel !== channel)
  s.desk.levels.push({ universe, channel, value })
}

/** Endereços [universo, canal DMX] dos canais de um grupo de canais. */
function groupAddresses(p: Project, groupId: string) {
  return (p.channelGroups.find((g) => g.id === groupId)?.channels ?? []).flatMap((c) => {
    const f = p.fixtures.find((x) => x.id === c.fixture)
    return f ? [[f.universe, f.address + c.channel] as [string, number]] : []
  })
}

/** Substitui o item de mesmo id ou o acrescenta ao fim. */
function upsert<T extends { id: string }>(list: T[], item: T) {
  const i = list.findIndex((x) => x.id === item.id)
  if (i >= 0) list[i] = item
  else list.push(item)
}

/**
 * Saída simulada dos universos: as criações rodando somam pelo maior valor, e a mesa simples
 * fica por cima (o último a mexer vence). É uma aproximação; o motor de verdade chega na F3.
 */
function simulate(s: Snapshot, now: number) {
  const p = s.project
  const out = Object.fromEntries(p.universes.map((u) => [u.id, new Uint8Array(512)]))
  const fixtures = new Map(p.fixtures.map((f) => [f.id, f]))
  const set = (fixtureId: string, channel: number, value: number) => {
    const f = fixtures.get(fixtureId)
    const frame = f && out[f.universe]
    const i = f ? f.address - 1 + channel : 512
    if (frame && i < 512) frame[i] = Math.max(frame[i]!, Math.round(value))
  }
  // Canais com o atributo pedido, na fixture inteira ou só numa cabeça (pixel de barra de LED)
  const setAttribute = (fixtureId: string, attribute: Attribute, value: number, head?: number) => {
    const f = fixtures.get(fixtureId)
    if (!f) return
    const heads = findDef(p, f.defId)?.modes.find((m) => m.name === f.mode)?.heads
    modeChannels(p, f).forEach((c, i) => {
      if (c.attribute === attribute && !c.fine && (head === undefined || !heads?.length || heads[head]?.includes(i))) set(fixtureId, i, value)
    })
  }
  const run = (id: string, depth: number): void => {
    const fn = p.functions.find((f) => f.id === id)
    if (!fn || depth > 4) return
    switch (fn.type) {
      case 'scene':
        for (const v of fn.values) set(v.fixture, v.channel, v.value)
        break
      case 'chaser': {
        const step = fn.steps[chaserStep(fn, now)]
        if (step) run(step.function, depth + 1)
        break
      }
      case 'sequence':
        for (const v of fn.steps[chaserStep(fn, now)]?.values ?? []) set(v.fixture, v.channel, v.value)
        break
      case 'collection':
        for (const child of fn.functions) run(child, depth + 1)
        break
      case 'efx':
        efxPositions(fn, now).forEach((pos, i) => {
          setAttribute(fn.fixtures[i]!.fixture, 'pan', pos.x)
          setAttribute(fn.fixtures[i]!.fixture, 'tilt', pos.y)
        })
        break
      case 'rgbMatrix': {
        const group = p.fixtureGroups.find((g) => g.id === fn.group)
        const step = Math.floor(now / Math.max(fn.timing.holdMs, 50))
        for (const cell of group?.cells ?? []) {
          const [r, g, b] = matrixColor(fn, cell.x, cell.y, group!.width, step)
          setAttribute(cell.fixture, 'red', r!, cell.head)
          setAttribute(cell.fixture, 'green', g!, cell.head)
          setAttribute(cell.fixture, 'blue', b!, cell.head)
        }
      }
    }
  }
  for (const id of s.running) run(id, 0)
  // Mesa por cima (os grupos já gravaram nos canais deles)
  for (const l of s.desk.levels) if (out[l.universe]) out[l.universe]![l.channel - 1] = l.value
  return out
}

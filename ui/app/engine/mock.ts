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
      s.widgetValues = []
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
      for (const w of p.console.widgets) if (w.type === 'button' || w.type === 'xyPad') w.fixtures = w.fixtures.filter((id) => !op.ids.includes(id))
      p.stage.fixtures = p.stage.fixtures.filter((x) => !op.ids.includes(x.fixture))
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
      for (const w of p.console.widgets) if (w.type === 'button') w.groups = w.groups.filter((id) => id !== op.id)
      break
    case 'saveChannelGroup':
      upsert(p.channelGroups, op.group)
      break
    case 'removeChannelGroup':
      p.channelGroups = p.channelGroups.filter((g) => g.id !== op.id)
      for (const w of p.console.widgets) if (w.type === 'slider') w.channelGroups = w.channelGroups.filter((id) => id !== op.id)
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
      for (const w of p.console.widgets) {
        if ((w.type === 'button' || w.type === 'slider') && w.function === op.id) w.function = null
        if (w.type === 'cueList' && w.chaser === op.id) w.chaser = null
        if (w.type === 'animation' && w.matrix === op.id) w.matrix = null
        if (w.type === 'speedDial') w.functions = w.functions.filter((id) => id !== op.id)
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
    // Parar também baixa os sliders de playback e solta os passos escolhidos nas cue lists
    case 'stopAllFunctions':
    case 'panic': {
      s.running = []
      const playback = new Set(p.console.widgets.filter((w) => (w.type === 'slider' && w.mode === 'playback') || w.type === 'cueList').map((w) => w.id))
      s.widgetValues = s.widgetValues.filter((v) => !playback.has(v.widget))
      if (op.op === 'panic') s.desk = { levels: [], groups: [] }
      return
    }
    case 'saveConsolePage':
      upsert(p.console.pages, op.page)
      break
    case 'removeConsolePage':
      p.console.pages = p.console.pages.filter((page) => page.id !== op.id)
      p.console.widgets = p.console.widgets.filter((w) => w.page !== op.id)
      break
    case 'saveWidgets':
      for (const w of op.widgets) upsert(p.console.widgets, w)
      break
    case 'removeWidgets':
      p.console.widgets = p.console.widgets.filter((w) => !op.ids.includes(w.id) && !op.ids.includes(w.frame ?? ''))
      s.widgetValues = s.widgetValues.filter((v) => p.console.widgets.some((w) => w.id === v.widget))
      break
    case 'setWidgetValue':
      s.widgetValues = s.widgetValues.filter((v) => v.widget !== op.widget).concat(op.value.length ? [{ widget: op.widget, value: op.value }] : [])
      return
    case 'setGrandMaster':
      s.master.grandMaster = op.value
      return
    case 'setBlackout':
      s.master.blackout = op.on
      return
    case 'setBpm':
      s.master.bpm = op.bpm
      return
    case 'saveStage':
      p.stage = op.stage
      break
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
 * Saída simulada dos universos. É uma aproximação; o motor de verdade chega na F3.
 * - As criações rodando somam pelo maior valor; playback, animation e submaster do console
 *   escalam a intensidade delas, e os sliders de nível somam nos canais dos grupos.
 * - O XY pad e a mesa simples ficam por cima (o último a mexer vence).
 * - Por fim, o blackout de fixtures, o Grand Master e o blackout geral agem nos canais de intensidade.
 */
function simulate(s: Snapshot, now: number) {
  const p = s.project
  const out = Object.fromEntries(p.universes.map((u) => [u.id, new Uint8Array(512)]))
  const fixtures = new Map(p.fixtures.map((f) => [f.id, f]))
  const channels = new Map(p.fixtures.map((f) => [f.id, modeChannels(p, f)]))
  const index = (fixtureId: string, channel: number) => {
    const f = fixtures.get(fixtureId)
    const i = f ? f.address - 1 + channel : 512
    return f && out[f.universe] && i < 512 ? { frame: out[f.universe]!, i } : undefined
  }
  // `k` escala só os canais de intensidade (dimmer e cores), como a intensidade de uma função no QLC+
  const set = (fixtureId: string, channel: number, value: number, k = 1) => {
    const at = index(fixtureId, channel)
    const scaled = intensity.has(channels.get(fixtureId)?.[channel]?.attribute ?? 'other') ? value * k : value
    if (at) at.frame[at.i] = Math.max(at.frame[at.i]!, Math.round(scaled))
  }
  // Canais com o atributo pedido, na fixture inteira ou só numa cabeça (pixel de barra de LED)
  const each = (fixtureId: string, attributes: Set<Attribute>, fn: (i: number) => void, head?: number) => {
    const f = fixtures.get(fixtureId)
    const heads = f && findDef(p, f.defId)?.modes.find((m) => m.name === f.mode)?.heads
    channels.get(fixtureId)?.forEach((c, i) => {
      if (attributes.has(c.attribute) && !c.fine && (head === undefined || !heads?.length || heads[head]?.includes(i))) fn(i)
    })
  }
  const assign = (fixtureId: string, attributes: Set<Attribute>, value: number) =>
    each(fixtureId, attributes, (i) => {
      const at = index(fixtureId, i)
      if (at) at.frame[at.i] = value
    })

  const widgets = p.console.widgets
  const values = new Map(s.widgetValues.map((v) => [v.widget, v.value]))
  const level = (id: string, fallback: number) => (values.get(id)?.[0] ?? fallback) / 255
  // Intensidade de cada criação vinda do console, e o passo escolhido nas cue lists
  const factor = new Map<string, number>()
  const runs = new Set(s.running)
  const cueStep = new Map<string, number>()
  for (const w of widgets) {
    if (w.type === 'slider' && w.mode === 'playback' && w.function && level(w.id, 0) > 0) {
      runs.add(w.function)
      factor.set(w.function, level(w.id, 0))
    }
    if (w.type === 'animation' && w.matrix) factor.set(w.matrix, level(w.id, 255))
    if (w.type === 'cueList' && w.chaser && values.has(w.id)) cueStep.set(w.chaser, values.get(w.id)![0]!)
    // O submaster vale para os botões da mesma moldura (ou da página, fora de moldura)
    if (w.type === 'slider' && w.mode === 'submaster') {
      for (const b of widgets) {
        if (b.type === 'button' && b.function && b.page === w.page && b.frame === w.frame) factor.set(b.function, (factor.get(b.function) ?? 1) * level(w.id, 255))
      }
    }
  }

  const run = (id: string, depth: number, k: number): void => {
    const fn = p.functions.find((f) => f.id === id)
    if (!fn || depth > 4) return
    k *= factor.get(id) ?? 1
    switch (fn.type) {
      case 'scene':
        for (const v of fn.values) set(v.fixture, v.channel, v.value, k)
        break
      case 'chaser': {
        const step = fn.steps[cueStep.get(fn.id) ?? chaserStep(fn, now)]
        if (step) run(step.function, depth + 1, k)
        break
      }
      case 'sequence':
        for (const v of fn.steps[cueStep.get(fn.id) ?? chaserStep(fn, now)]?.values ?? []) set(v.fixture, v.channel, v.value, k)
        break
      case 'collection':
        for (const child of fn.functions) run(child, depth + 1, k)
        break
      case 'efx':
        efxPositions(fn, now).forEach((pos, i) => {
          each(fn.fixtures[i]!.fixture, new Set(['pan']), (c) => set(fn.fixtures[i]!.fixture, c, pos.x))
          each(fn.fixtures[i]!.fixture, new Set(['tilt']), (c) => set(fn.fixtures[i]!.fixture, c, pos.y))
        })
        break
      case 'rgbMatrix': {
        const group = p.fixtureGroups.find((g) => g.id === fn.group)
        const step = Math.floor(now / Math.max(fn.timing.holdMs, 50))
        for (const cell of group?.cells ?? []) {
          const rgb = matrixColor(fn, cell.x, cell.y, group!.width, step)
          ;(['red', 'green', 'blue'] as const).forEach((attribute, n) =>
            each(cell.fixture, new Set([attribute]), (c) => set(cell.fixture, c, rgb[n]!, k), cell.head))
        }
      }
    }
  }
  for (const id of runs) run(id, 0, 1)

  for (const w of widgets) {
    if (w.type === 'slider' && w.mode === 'level') {
      for (const group of p.channelGroups.filter((g) => w.channelGroups.includes(g.id))) {
        for (const c of group.channels) set(c.fixture, c.channel, level(w.id, 0) * 255)
      }
    }
    const xy = w.type === 'xyPad' && values.get(w.id)
    if (w.type === 'xyPad' && xy) {
      for (const id of w.fixtures) {
        assign(id, new Set(['pan']), xy[0]!)
        assign(id, new Set(['tilt']), xy[1]!)
      }
    }
  }
  // Mesa por cima (os grupos já gravaram nos canais deles)
  for (const l of s.desk.levels) if (out[l.universe]) out[l.universe]![l.channel - 1] = l.value

  for (const w of widgets) {
    if (w.type !== 'button' || w.action !== 'blackoutFixtures' || !values.has(w.id)) continue
    const ids = [...w.fixtures, ...p.fixtureGroups.filter((g) => w.groups.includes(g.id)).flatMap((g) => g.cells.map((c) => c.fixture))]
    for (const id of ids) assign(id, intensity, 0)
  }
  const gm = s.master.blackout ? 0 : s.master.grandMaster / 255
  if (gm < 1) {
    for (const f of p.fixtures) {
      each(f.id, intensity, (i) => {
        const at = index(f.id, i)
        if (at) at.frame[at.i] = Math.round(at.frame[at.i]! * gm)
      })
    }
  }
  return out
}

/** Canais de intensidade: o Grand Master e os blackouts agem neles. */
const intensity = new Set<Attribute>(['dimmer', 'red', 'green', 'blue', 'white', 'amber', 'uv'])

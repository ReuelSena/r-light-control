import type { ImportReport } from '~/types/generated/ImportReport'
import type { Op } from '~/types/generated/Op'
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

  return {
    state,
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

/** Substitui o item de mesmo id ou o acrescenta ao fim. */
function upsert<T extends { id: string }>(list: T[], item: T) {
  const i = list.findIndex((x) => x.id === item.id)
  if (i >= 0) list[i] = item
  else list.push(item)
}

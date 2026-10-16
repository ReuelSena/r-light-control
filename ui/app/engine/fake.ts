import type { Attribute } from '~/types/generated/Attribute'
import type { Capability } from '~/types/generated/Capability'
import type { Channel } from '~/types/generated/Channel'
import type { Fixture } from '~/types/generated/Fixture'
import type { FixtureDef } from '~/types/generated/FixtureDef'
import type { Physical } from '~/types/generated/Physical'
import type { Project } from '~/types/generated/Project'
import type { Snapshot } from '~/types/generated/Snapshot'

// Dados fictícios do protótipo (F2). Saem quando o motor real chegar (F3).

const cap = (name: string, min = 0, max = 255, color: string | null = null): Capability => ({ min, max, name, color })
const ch = (name: string, attribute: Attribute, capabilities = [cap(name)], fine = false): Channel => ({ name, attribute, fine, capabilities })
const physical = (w: number, h: number, d: number, kg: number, watts: number, lens = [25, 25], pan = 0, tilt = 0): Physical => ({
  widthMm: w, heightMm: h, depthMm: d, weightKg: kg, powerW: watts, lensMinDeg: lens[0]!, lensMaxDeg: lens[1]!, panMaxDeg: pan, tiltMaxDeg: tilt,
})
const range = (n: number) => Array.from({ length: n }, (_, i) => i)

const strobe = [cap('Aberto', 0, 9), cap('Strobe lento → rápido', 10, 255)]

const defs: FixtureDef[] = [
  {
    id: 'generico-par-rgbw', manufacturer: 'Genérico', model: 'PAR LED RGBW', kind: 'colorChanger',
    channels: [
      ch('Dimmer', 'dimmer'), ch('Vermelho', 'red'), ch('Verde', 'green'), ch('Azul', 'blue'), ch('Branco', 'white'),
      ch('Strobe', 'shutter', strobe),
      ch('Macro de cor', 'colorMacro', [
        cap('Desligado', 0, 9), cap('Vermelho', 10, 39, '#ff0000'), cap('Verde', 40, 69, '#00ff00'),
        cap('Azul', 70, 99, '#0000ff'), cap('Branco', 100, 129, '#ffffff'), cap('Troca automática', 130, 255),
      ]),
    ],
    modes: [{ name: '7 canais', channels: range(7), heads: [] }, { name: '4 canais (RGBW)', channels: [1, 2, 3, 4], heads: [] }],
    physical: physical(240, 300, 200, 2.5, 60),
  },
  {
    id: 'generico-par-rgb', manufacturer: 'Genérico', model: 'PAR LED RGB', kind: 'colorChanger',
    channels: [ch('Vermelho', 'red'), ch('Verde', 'green'), ch('Azul', 'blue')],
    modes: [{ name: '3 canais', channels: range(3), heads: [] }],
    physical: physical(200, 250, 180, 1.8, 36),
  },
  {
    id: 'generico-moving-spot', manufacturer: 'Genérico', model: 'Moving Head Spot 150', kind: 'movingHead',
    channels: [
      ch('Pan', 'pan'), ch('Pan fino', 'pan', undefined, true), ch('Tilt', 'tilt'), ch('Tilt fino', 'tilt', undefined, true),
      ch('Velocidade P/T', 'speed', [cap('Rápido → lento')]), ch('Dimmer', 'dimmer'),
      ch('Shutter', 'shutter', [cap('Fechado', 0, 7), cap('Aberto', 8, 15), cap('Strobe lento → rápido', 16, 255)]),
      ch('Roda de cores', 'colorWheel', [
        cap('Branco', 0, 9, '#ffffff'), cap('Vermelho', 10, 19, '#ff0000'), cap('Laranja', 20, 29, '#ff8000'),
        cap('Amarelo', 30, 39, '#ffff00'), cap('Verde', 40, 49, '#00ff00'), cap('Azul', 50, 59, '#0000ff'),
        cap('Magenta', 60, 69, '#ff00aa'), cap('Rotação', 128, 255),
      ]),
      ch('Gobo', 'gobo', [cap('Aberto', 0, 9), ...range(7).map((i) => cap(`Gobo ${i + 1}`, 10 + i * 10, 19 + i * 10)), cap('Shake', 128, 255)]),
      ch('Rotação do gobo', 'goboRotation'),
      ch('Prisma', 'prism', [cap('Desligado', 0, 127), cap('Prisma 3 faces', 128, 255)]),
      ch('Foco', 'focus'), ch('Zoom', 'zoom'),
      ch('Reset', 'maintenance', [cap('Nada', 0, 199), cap('Reset', 200, 255)]),
    ],
    modes: [{ name: '14 canais', channels: range(14), heads: [] }, { name: '9 canais', channels: [0, 2, 4, 5, 6, 7, 8, 11, 12], heads: [] }],
    physical: physical(300, 450, 250, 12, 150, [12, 30], 540, 270),
  },
  {
    id: 'generico-barra-8px', manufacturer: 'Genérico', model: 'Barra LED 8 pixels RGB', kind: 'ledBar',
    channels: range(8).flatMap((i) => [ch(`Vermelho ${i + 1}`, 'red'), ch(`Verde ${i + 1}`, 'green'), ch(`Azul ${i + 1}`, 'blue')]),
    modes: [
      { name: '24 canais (8 pixels)', channels: range(24), heads: range(8).map((i) => [i * 3, i * 3 + 1, i * 3 + 2]) },
      { name: '3 canais (barra inteira)', channels: range(3), heads: [[0, 1, 2]] },
    ],
    physical: physical(1000, 80, 80, 3, 80),
  },
  {
    id: 'generico-strobe', manufacturer: 'Genérico', model: 'Strobe LED', kind: 'strobe',
    channels: [ch('Dimmer', 'dimmer'), ch('Velocidade', 'shutter', [cap('Lento → rápido')])],
    modes: [{ name: '2 canais', channels: range(2), heads: [] }],
    physical: physical(400, 200, 150, 3, 200),
  },
  {
    id: 'generico-hazer', manufacturer: 'Genérico', model: 'Máquina de haze', kind: 'hazer',
    channels: [ch('Saída', 'effect', [cap('Desligado', 0, 9), cap('Pouco → muito', 10, 255)]), ch('Ventilador', 'speed')],
    modes: [{ name: '2 canais', channels: range(2), heads: [] }],
    physical: physical(450, 300, 250, 9, 700),
  },
]

const fixture = (id: string, defId: string, mode: string, name: string, color: string, universe: string, address: number): Fixture =>
  ({ id, defId, mode, name, color, universe, address })

const fixtures: Fixture[] = [
  ...range(6).map((i) => fixture(`par${i + 1}`, 'generico-par-rgbw', '7 canais', `PAR ${i + 1}`, '#ff8000', 'u1', 1 + i * 7)),
  // De propósito em cima do PAR 6, para a tela de patch mostrar um conflito
  fixture('par7', 'generico-par-rgb', '3 canais', 'PAR 7', '#ff00aa', 'u1', 41),
  ...range(4).map((i) => fixture(`mh${i + 1}`, 'generico-moving-spot', '14 canais', `MH ${i + 1}`, '#00bfff', 'u1', 101 + i * 14)),
  fixture('strobe', 'generico-strobe', '2 canais', 'Strobe', '#ffffff', 'u1', 200),
  fixture('hazer', 'generico-hazer', '2 canais', 'Haze', '#00ff00', 'u1', 210),
  fixture('barra1', 'generico-barra-8px', '24 canais (8 pixels)', 'Barra 1', '#8000ff', 'u2', 1),
  fixture('barra2', 'generico-barra-8px', '24 canais (8 pixels)', 'Barra 2', '#8000ff', 'u2', 25),
]

const clone = <T>(value: T): T => JSON.parse(JSON.stringify(value))
// Os canais `channels` (posição no modo) de cada fixture de `ids`, para os grupos de canais
const pick = (ids: string[], channels: number[]) => ids.flatMap((fixture) => channels.map((channel) => ({ fixture, channel })))
const pars = range(6).map((i) => `par${i + 1}`)

export function emptyProject(): Project {
  return {
    name: 'Novo projeto', universes: [{ id: 'u1', name: 'Universo 1', output: null }],
    fixtureDefs: [], fixtures: [], fixtureGroups: [], channelGroups: [],
  }
}

export function fakeSnapshot(): Snapshot {
  return {
    project: {
      name: 'Show de exemplo',
      universes: [{ id: 'u1', name: 'Palco', output: 'i1' }, { id: 'u2', name: 'Fundo', output: 'i2' }],
      fixtureDefs: clone(defs),
      fixtures: clone(fixtures),
      fixtureGroups: [
        { id: 'g1', name: 'PARs frente', width: 6, height: 1, cells: pars.map((fixture, x) => ({ x, y: 0, fixture, head: 0 })) },
        {
          id: 'g2', name: 'Barras LED', width: 8, height: 2,
          cells: [1, 2].flatMap((bar) => range(8).map((i) => ({ x: i, y: bar - 1, fixture: `barra${bar}`, head: i }))),
        },
      ],
      channelGroups: [
        { id: 'cg1', name: 'Dimmer PARs', channels: pick(pars, [0]) },
        { id: 'cg2', name: 'Vermelho geral', channels: [...pick(pars, [1]), ...pick(['par7'], [0])] },
        { id: 'cg3', name: 'RGB PARs', channels: pick(pars, [1, 2, 3]) },
        { id: 'cg4', name: 'Pan/Tilt MHs', channels: pick(['mh1', 'mh2', 'mh3', 'mh4'], [0, 2]) },
        { id: 'cg5', name: 'Strobe e haze', channels: pick(['strobe', 'hazer'], [0]) },
      ],
    },
    projectPath: 'C:\\Shows\\Show de exemplo.rlc',
    dirty: false,
    recentProjects: ['C:\\Shows\\Show de exemplo.rlc', 'C:\\Shows\\Culto de domingo.rlc', 'C:\\Shows\\Formatura 2026.rlc'],
    library: clone(defs),
    interfaces: [
      { id: 'i1', kind: 'enttecPro', name: 'ENTTEC DMX USB PRO', serial: 'EN217642', connected: true, fps: 44, jitterMs: 0.2, errors: 0 },
      { id: 'i2', kind: 'openDmx', name: 'Open DMX (FTDI)', serial: 'A10KX7Q2', connected: true, fps: 30, jitterMs: 0.6, errors: 0 },
      { id: 'i3', kind: 'dummy', name: 'Saída virtual', serial: '—', connected: true, fps: 44, jitterMs: 0, errors: 0 },
    ],
    midiDevices: [
      { id: 'm1', name: 'Controlador MIDI USB', output: true, connected: true },
      { id: 'm2', name: 'loopMIDI Port', output: false, connected: true },
    ],
    remote: {
      enabled: true,
      pin: null,
      lanUrl: 'http://192.168.0.6:8090',
      clients: [
        { id: 'c1', name: 'Este computador', address: '127.0.0.1', thisDevice: true },
        { id: 'c2', name: 'Celular', address: '192.168.0.23', thisDevice: false },
      ],
    },
  }
}

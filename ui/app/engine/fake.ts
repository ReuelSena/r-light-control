import type { Attribute } from '~/types/generated/Attribute'
import type { Capability } from '~/types/generated/Capability'
import type { Channel } from '~/types/generated/Channel'
import type { Fixture } from '~/types/generated/Fixture'
import type { FixtureDef } from '~/types/generated/FixtureDef'
import type { Function } from '~/types/generated/Function'
import type { GroupLevel } from '~/types/generated/GroupLevel'
import type { Physical } from '~/types/generated/Physical'
import type { Project } from '~/types/generated/Project'
import type { SceneValue } from '~/types/generated/SceneValue'
import type { Snapshot } from '~/types/generated/Snapshot'
import type { StageElementKind } from '~/types/generated/StageElementKind'
import type { Rect } from '~/types/generated/Rect'
import type { VcButton } from '~/types/generated/VcButton'
import type { VcWidget } from '~/types/generated/VcWidget'
import type { WidgetInput } from '~/types/generated/WidgetInput'

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
const mhs = ['mh1', 'mh2', 'mh3', 'mh4']

// Valores de cena: { canal: valor } aplicado a cada fixture de `ids`
const vals = (ids: string[], values: Record<number, number>): SceneValue[] =>
  ids.flatMap((fixture) => Object.entries(values).map(([channel, value]) => ({ fixture, channel: Number(channel), value })))
const timing = (fadeInMs: number, holdMs: number, fadeOutMs = 0) => ({ fadeInMs, holdMs, fadeOutMs })
const scene = (id: string, name: string, values: SceneValue[], channelGroups: GroupLevel[] = [], folder: string | null = null): Function =>
  ({ id, name, folder, type: 'scene', values, channelGroups, fadeInMs: 1000, fadeOutMs: 1000 })

const functions: Function[] = [
  scene('s1', 'Frente âmbar', vals(pars, { 0: 255, 1: 255, 2: 140 }), [{ group: 'cg1', value: 255 }], 'd1'),
  scene('s2', 'Frente azul', vals(pars, { 0: 255, 3: 255 }), [{ group: 'cg1', value: 255 }], 'd1'),
  scene('s3', 'Frente vermelha', vals(pars, { 0: 255, 1: 255 }), [{ group: 'cg1', value: 255 }], 'd1'),
  scene('s4', 'MHs no centro', vals(mhs, { 0: 128, 2: 128, 5: 255, 6: 10, 12: 100 })),
  scene('s5', 'Haze leve', vals(['hazer'], { 0: 80, 1: 120 })),
  {
    id: 'c1', name: 'Troca de cores', folder: null, type: 'chaser', scene: null, runOrder: 'loop', direction: 'forward', beats: false,
    fadeInMode: 'perStep', holdMode: 'perStep', fadeOutMode: 'default', common: timing(500, 2000),
    steps: ['s1', 's2', 's3'].map((fn) => ({ function: fn, timing: timing(1000, 2000), note: '', values: [] })),
  },
  {
    id: 'q1', name: 'Subida da frente', folder: null, type: 'sequence', scene: 's1', runOrder: 'pingPong', direction: 'forward', beats: false,
    fadeInMode: 'common', holdMode: 'common', fadeOutMode: 'common', common: timing(800, 400),
    steps: [0.25, 0.5, 1].map((k) => ({ function: 's1', timing: timing(800, 400), note: `${k * 100}%`, values: vals(pars, { 0: Math.round(255 * k), 1: 255, 2: 140 }) })),
  },
  { id: 'k1', name: 'Abertura', folder: null, type: 'collection', functions: ['c1', 's4', 'e1', 's5'] },
  {
    id: 'e1', name: 'Círculo nos MHs', folder: null, type: 'efx', algorithm: 'circle', width: 64, height: 64, xOffset: 128, yOffset: 128, rotation: 0,
    xFrequency: 2, yFrequency: 3, xPhase: 90, yPhase: 0, relative: false, propagation: 'asymmetric',
    fixtures: mhs.map((fixture) => ({ fixture, reverse: false, startOffset: 0 })), runOrder: 'loop', direction: 'forward', timing: timing(0, 4000),
  },
  {
    id: 'r1', name: 'Degradê nas barras', folder: null, type: 'rgbMatrix', group: 'g2', algorithm: 'gradient', colors: ['#ff0000', '#0000ff'],
    runOrder: 'loop', direction: 'forward', timing: timing(0, 200),
  },
  {
    id: 'r2', name: 'Perseguição nos PARs', folder: null, type: 'rgbMatrix', group: 'g1', algorithm: 'chase', colors: ['#ffbf00', '#000000'],
    runOrder: 'loop', direction: 'forward', timing: timing(0, 250),
  },
]

// Console de exemplo, montado no formato desktop; tablet e celular saem dele (copyLayout)
const rect = (x: number, y: number, w: number, h: number): Rect => ({ x, y, w, h })
const key = (key: string, control: WidgetInput['control'] = 'main'): WidgetInput => ({ control, midi: null, key })
const midi = (message: 'note' | 'controlChange', number: number): WidgetInput =>
  ({ control: 'main', midi: { device: 'm1', channel: 1, message, number, feedback: true }, key: null })
const widget = (id: string, page: string, caption: string, color: string, desktop: Rect, kind: object, frame: string | null = null, inputs: WidgetInput[] = []) =>
  ({ id, page, frame, caption, color, layouts: { phone: null, tablet: null, desktop }, inputs, ...kind }) as VcWidget
const button = (fn: string | null, extra: Partial<VcButton> = {}) =>
  ({ type: 'button', action: 'function', function: fn, fixtures: [], groups: [], mode: 'toggle', highlight: 'face', icon: null, ...extra })

function consoleWidgets(): VcWidget[] {
  const widgets = [
    widget('fr1', 'p1', 'Frente', '#ff8000', rect(0, 0, 16, 13), { type: 'frame', solo: true }),
    widget('b1', 'p1', 'Âmbar', '#ffbf00', rect(0, 1, 4, 4), button('s1'), 'fr1', [key('1')]),
    widget('b2', 'p1', 'Azul', '#0080ff', rect(4, 1, 4, 4), button('s2'), 'fr1', [key('2')]),
    widget('b3', 'p1', 'Vermelha', '#ff0000', rect(8, 1, 4, 4), button('s3'), 'fr1', [key('3')]),
    widget('b4', 'p1', 'Apagar PARs', '#ff0000', rect(0, 5, 4, 4), button(null, { action: 'blackoutFixtures', groups: ['g1'], highlight: 'led', icon: 'LightbulbOff' }), 'fr1'),
    widget('sl1', 'p1', 'Sub frente', '#ff8000', rect(12, 1, 4, 12), { type: 'slider', mode: 'submaster', channelGroups: [], function: null, knob: false }, 'fr1'),
    widget('b6', 'p1', 'Troca de cores', '#ff00aa', rect(16, 0, 4, 4), button('c1'), null, [midi('note', 36)]),
    widget('b7', 'p1', 'Círculo', '#00bfff', rect(20, 0, 4, 4), button('e1')),
    widget('b8', 'p1', 'Degradê', '#8000ff', rect(24, 0, 4, 4), button('r1')),
    widget('b9', 'p1', 'Perseguição', '#ffbf00', rect(28, 0, 4, 4), button('r2')),
    widget('b10', 'p1', 'Abertura', '#ff8000', rect(16, 4, 4, 4), button('k1', { highlight: 'led' })),
    widget('b11', 'p1', 'Haze', '#00ff00', rect(20, 4, 4, 4), button('s5', { mode: 'flash' }), null, [key('H')]),
    widget('b12', 'p1', 'Parar tudo', '#ff0000', rect(24, 4, 4, 4), button(null, { action: 'stopAll', icon: 'OctagonX' })),
    widget('sl2', 'p1', 'Dimmer PARs', '#ffbf00', rect(32, 0, 4, 12), { type: 'slider', mode: 'level', channelGroups: ['cg1'], function: null, knob: false }, null, [midi('controlChange', 21)]),
    widget('sl3', 'p1', 'MHs centro', '#00bfff', rect(36, 0, 4, 12), { type: 'slider', mode: 'playback', channelGroups: [], function: 's4', knob: false }),
    widget('xy1', 'p1', 'Posição MHs', '#00bfff', rect(40, 0, 8, 12), { type: 'xyPad', fixtures: mhs }),
    widget('sd1', 'p1', 'Fades da frente', '#ffff00', rect(16, 8, 12, 8), { type: 'speedDial', functions: ['s1', 's2', 's3'], fadeIn: true, fadeOut: true, hold: false }),
    widget('cl1', 'p1', 'Troca de cores', '#ff00aa', rect(0, 13, 16, 12), { type: 'cueList', chaser: 'c1' }, null, [key('ArrowLeft', 'previous'), key('ArrowRight', 'next')]),
    widget('ck1', 'p1', 'Relógio', '#00ffff', rect(28, 12, 8, 4), { type: 'clock', mode: 'clock', countdownS: 0 }),
    widget('lb1', 'p1', 'Palco', '#ff8000', rect(40, 12, 8, 3), { type: 'label', background: '#402000', border: '#ff8000' }),
    widget('an1', 'p2', 'Barras', '#8000ff', rect(0, 0, 12, 12), { type: 'animation', matrix: 'r1' }),
    widget('an2', 'p2', 'PARs', '#ffbf00', rect(12, 0, 12, 12), { type: 'animation', matrix: 'r2' }),
    widget('cl2', 'p2', 'Subida da frente', '#ff8000', rect(24, 0, 16, 12), { type: 'cueList', chaser: 'q1' }),
    widget('ck2', 'p2', 'Cronômetro', '#00ffff', rect(40, 0, 8, 4), { type: 'clock', mode: 'stopwatch', countdownS: 0 }),
  ]
  for (const page of ['p1', 'p2']) {
    copyLayout(widgets.filter((w) => w.page === page), 'desktop', 'tablet')
    copyLayout(widgets.filter((w) => w.page === page), 'desktop', 'phone')
  }
  return widgets
}

// Posições no palco: PARs na vara da frente mirando o palco, moving heads na vara do fundo
const at = (fixture: string, x: number, y: number, z: number, rotationX = 0) => ({ fixture, x, y, z, rotationX, rotationY: 0, rotationZ: 0 })
const positions = [
  ...pars.map((id, i) => at(id, -3.75 + i * 1.5, 4.5, 2.5, 35)),
  at('par7', 0, 0.2, -3.6, 165),
  ...mhs.map((id, i) => at(id, -3 + i * 2, 5, -2)),
  at('strobe', 0, 5, 0.5),
  at('hazer', 4, 0, -3.5),
  at('barra1', -2.5, 0.1, -3.8),
  at('barra2', 2.5, 0.1, -3.8),
]

// Elementos de palco do exemplo
const element = (id: string, kind: StageElementKind, x: number, y: number, z: number, rotationY = 0) => ({ id, kind, x, y, z, rotationX: 0, rotationY, rotationZ: 0 })
const elements = [
  ...[-3, 0, 3].flatMap((x, i) => [element(`t${i + 1}`, 'truss300', x, 4.5, 2.5), element(`t${i + 4}`, 'truss300', x, 5, -2)]),
  element('sub1', 'subwoofer', -4.2, 0, 3.4),
  element('sub2', 'subwoofer', 4.2, 0, 3.4),
  element('cx1', 'speaker', -4.2, 0.6, 3.4),
  element('cx2', 'speaker', 4.2, 0.6, 3.4),
  element('pr1', 'riser', -1, 0, -0.8),
  element('pr2', 'riser', 1, 0, -0.8),
  element('bt1', 'drums', 0, 0.4, -0.9),
  element('ps1', 'person', 0, 0, 1.5),
  element('pd1', 'micStand', 0, 0, 1.85),
  element('rt1', 'monitor', 0, 0, 2.6),
]

export function emptyProject(): Project {
  return {
    name: 'Novo projeto', universes: [{ id: 'u1', name: 'Universo 1', output: null }],
    fixtureDefs: [], fixtures: [], fixtureGroups: [], channelGroups: [], functions: [], folders: [],
    console: { pages: [{ id: 'p1', name: 'Página 1' }], widgets: [] },
    stage: { width: 10, depth: 8, height: 6, fixtures: [], elements: [] },
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
      functions: clone(functions),
      folders: [{ id: 'd1', name: 'Frente', kind: 'scene' }],
      console: { pages: [{ id: 'p1', name: 'Show' }, { id: 'p2', name: 'Efeitos' }], widgets: consoleWidgets() },
      stage: { width: 10, depth: 8, height: 6, fixtures: clone(positions), elements: clone(elements) },
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
    // Duas criações já rodando, para o Monitoramento mostrar sinal
    running: ['s1', 'r1'],
    desk: { levels: [], groups: [] },
    master: { grandMaster: 255, blackout: false, bpm: 120 },
    widgetValues: [],
    stats: { cpuPercent: 8, memoryMb: 142, tickMs: 0.4, uptimeS: 0 },
  }
}

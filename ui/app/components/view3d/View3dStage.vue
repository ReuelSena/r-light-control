<script setup lang="ts">
import { TresCanvas } from '@tresjs/core'
import {
  AdditiveBlending, Box3, BoxGeometry, CanvasTexture, Color, CylinderGeometry, DoubleSide, GridHelper, Group, MOUSE, Mesh, MeshBasicMaterial,
  MeshStandardMaterial, PerspectiveCamera, Plane, PlaneGeometry, Quaternion, Raycaster, ShaderMaterial, TOUCH, Vector2, Vector3, type Object3D,
} from 'three'
import { OrbitControls } from 'three/examples/jsm/controls/OrbitControls.js'
import { GLTFLoader } from 'three/examples/jsm/loaders/GLTFLoader.js'
import type { Capability } from '~/types/generated/Capability'
import type { Fixture } from '~/types/generated/Fixture'
import type { FixtureKind } from '~/types/generated/FixtureKind'

// Palco em 3D (E12): as fixtures acendem com a saída dos universos (simulada na F2), com feixes
// aditivos como no teste da F1. Vistas 3D, topo, frente e lateral. Câmera: com o mouse, o botão do meio
// gira (3D) ou desloca, o direito desloca e a roda aproxima; no toque, um dedo gira ou desloca e a pinça
// aproxima. Os elementos de palco (truss, caixas…) complementam a prévia.
// O botão esquerdo (ou o toque) só escolhe, e só o que é da aba aberta (`layer`): fixtures ou elementos.
// Com "Editar posição" (`editing`), arrastar o escolhido o move em qualquer vista (no toque, depois de
// escolher). `source="preview"` desenha o palco com os quadros do modo Preparar (sem edição). O feixe (F15b) segue o zoom, o foco, o shutter, o prisma (leque girando) e o gobo (na mancha do chão).
const props = defineProps<{ view: 'perspective' | 'top' | 'front' | 'side'; editing: boolean; layer?: 'fixtures' | 'elements'; source?: 'live' | 'preview'; hidden?: { fixtures: boolean; elements: boolean; ids: string[] } }>()
const selected = defineModel<string[]>('selected', { default: () => [] })
const { state, output, previewOutput, send } = useEngine()
const prefs = usePrefs()
const project = computed(() => state.value.project)
const stage = computed(() => project.value.stage)
const elementsById = computed(() => new Map(stage.value.elements.map((e) => [e.id, e])))
const FACETS = 8 // feixes montados por fixture: 1 sem prisma, até 8 com ele
const FAN = (5 * Math.PI) / 180 // abertura de cada feixe do prisma em relação ao eixo

const wrapper = ref<HTMLElement>()
const camera = new PerspectiveCamera(50, 1, 0.1, 200)
let controls: OrbitControls | undefined

// ---------- Montagem da cena ----------
// Canal (posição no modo) e as capabilities dele, para prisma e gobo
type Channel = { index: number; caps: Capability[] }
type Look = {
  fixture: Fixture; group: Group; body: MeshStandardMaterial; pan?: Object3D; tilt?: Object3D; shutter: Capability[]; lens?: MeshBasicMaterial; glass?: Mesh; pixels: MeshBasicMaterial[]
  // Feixe: o leque gira no eixo (prisma); cada cone tem a sua mancha. Sem prisma, só o primeiro aparece
  beam?: ShaderMaterial; fan?: Group; cones: Mesh[]; spots: Mesh[]; spotMat?: ShaderMaterial; goboAngle: number; radius: number // raio da lente (m)
  prism?: Channel; prismSpin?: Channel; gobo?: Channel; goboSpin?: Channel
  mirror?: boolean // scanner com o modelo: o espelho inclina a metade do tilt, e o feixe, o tilt inteiro
}
type Piece = { id: string; group: Group; materials: MeshStandardMaterial[] }
const root = shallowRef(new Group())
let looks: Look[] = []
let pieces: Piece[] = []
let pickables: Record<'fixtures' | 'elements', Object3D[]> = { fixtures: [], elements: [] }

// Feixe: tronco de cone que sai da face da lente (raio `uLens`) e abre `uHalf` (tangente do meio ângulo) por metro.
// A geometria é um cilindro de raio 1 e altura 1 (de y = 0 a -1); o vertex shader põe o raio de cada ponto, e o
// comprimento visível vem da escala Y do cone (cada feixe do leque termina no chão num ponto diferente). Assim o
// raio da lente não cresce com o comprimento, sem remontar a geometria quando o ângulo ou o alcance mudam.
function beamMaterial() {
  return new ShaderMaterial({
    transparent: true,
    depthWrite: false,
    blending: AdditiveBlending,
    side: DoubleSide,
    uniforms: { uColor: { value: new Color(0, 0, 0) }, uSoft: { value: 0.5 }, uCore: { value: 1 }, uLens: { value: 0.05 }, uHalf: { value: 0.1 }, uReach: { value: 10 } },
    vertexShader: /* glsl */ `
      uniform float uLens;
      uniform float uHalf;
      uniform float uReach;
      varying float vAlong; // metros desde a lente, em fração do alcance
      varying float vFacing;
      void main() {
        float sx = length(modelMatrix[0].xyz);
        float sy = length(modelMatrix[1].xyz); // comprimento visível (m)
        float t = -position.y;
        vec2 n = normalize(position.xz);
        float r = (uLens + t * sy * uHalf) / sx;
        vAlong = t * sy / uReach;
        // Normal do tronco (gradiente da superfície no espaço local)
        vec3 g = vec3(sx * n.x, sy * uHalf, sx * n.y);
        vec4 mv = modelViewMatrix * vec4(n.x * r, position.y, n.y * r, 1.0);
        vFacing = abs(dot(normalize(normalMatrix * g), normalize(-mv.xyz)));
        gl_Position = projectionMatrix * mv;
      }`,
    fragmentShader: /* glsl */ `
      uniform vec3 uColor;
      uniform float uSoft;
      uniform float uCore;
      varying float vAlong;
      varying float vFacing;
      void main() {
        // Distância ao eixo na tela, de 0 (centro) a 1 (silhueta)
        float rho = sqrt(max(0.0, 1.0 - vFacing * vFacing));
        // Halo fraco e largo: cheio até perto da borda (uSoft = 0) ou em degradê até o centro (1); núcleo fino e forte
        float halo = mix(1.0 - smoothstep(0.75, 1.0, rho), pow(1.0 - rho, 1.5), uSoft);
        float core = exp(-rho * rho * 30.0);
        // Quase constante no alcance: queda leve e um fade curto no fim
        float along = (1.0 - 0.3 * vAlong) * (1.0 - smoothstep(0.85, 1.0, vAlong));
        // Saturação suave por canal: a cor continua a dela (o azul não vira branco)
        gl_FragColor = vec4(1.0 - exp(-uColor * (0.25 * halo + 0.2 * uCore * core) * along), 1.0);
        #include <colorspace_fragment>
      }`,
  })
}

// Gobos gerados em canvas (sem arquivos), brancos sobre preto: pontos, estrela, raios, círculos e linhas
let gobos: CanvasTexture[] = []
function goboTextures() {
  const circle = (g: CanvasRenderingContext2D, x: number, y: number, r: number, fill = true) => {
    g.beginPath()
    g.arc(x, y, r, 0, 2 * Math.PI)
    if (fill) g.fill()
    else g.stroke()
  }
  const draws: ((g: CanvasRenderingContext2D) => void)[] = [
    (g) => {
      for (let x = -3; x <= 3; x++) for (let y = -3; y <= 3; y++) if (x * x + y * y <= 9) circle(g, 64 + x * 17, 64 + y * 17, 6)
    },
    (g) => {
      g.beginPath()
      for (let i = 0; i < 10; i++) g.lineTo(64 + (i % 2 ? 24 : 62) * Math.cos((i * Math.PI) / 5), 64 + (i % 2 ? 24 : 62) * Math.sin((i * Math.PI) / 5))
      g.fill()
    },
    (g) => {
      for (let i = 0; i < 12; i++) {
        g.beginPath()
        g.moveTo(64, 64)
        g.arc(64, 64, 64, (i * Math.PI) / 6, (i * Math.PI) / 6 + Math.PI / 12)
        g.fill()
      }
    },
    (g) => {
      g.lineWidth = 8
      for (const r of [14, 32, 52]) circle(g, 64, 64, r, false)
    },
    (g) => {
      for (let y = 6; y < 128; y += 20) g.fillRect(0, y, 128, 9)
    },
  ]
  return draws.map((draw) => {
    const canvas = document.createElement('canvas')
    canvas.width = canvas.height = 128
    const g = canvas.getContext('2d')!
    g.fillStyle = g.strokeStyle = '#fff'
    draw(g)
    return new CanvasTexture(canvas)
  })
}

// Mancha do feixe no chão: disco com gradiente radial, de borda dura (0) a macia (1), com o gobo como máscara
function spotMaterial() {
  return new ShaderMaterial({
    transparent: true,
    depthWrite: false,
    blending: AdditiveBlending,
    uniforms: { uColor: { value: new Color(0, 0, 0) }, uSoft: { value: 0.5 }, uGobo: { value: gobos[0] }, uHasGobo: { value: 0 }, uGoboAngle: { value: 0 } },
    vertexShader: /* glsl */ `
      varying vec2 vP;
      void main() {
        vP = uv * 2.0 - 1.0;
        gl_Position = projectionMatrix * modelViewMatrix * vec4(position, 1.0);
      }`,
    fragmentShader: /* glsl */ `
      uniform vec3 uColor;
      uniform float uSoft;
      uniform sampler2D uGobo;
      uniform float uHasGobo;
      uniform float uGoboAngle;
      varying vec2 vP;
      void main() {
        float d = length(vP);
        float hard = 1.0 - smoothstep(0.85, 1.0, d);
        float soft = pow(max(1.0 - d, 0.0), 1.5);
        float c = cos(uGoboAngle);
        float s = sin(uGoboAngle);
        float mask = mix(1.0, texture2D(uGobo, mat2(c, -s, s, c) * vP * 0.5 + 0.5).r, uHasGobo);
        gl_FragColor = vec4(1.0 - exp(-uColor * mix(hard, soft, uSoft) * mask), 1.0); // saturação suave, como no feixe
        #include <colorspace_fragment>
      }`,
  })
}

// Modelos glTF do QLC+ (`public/models/`, convenções no README de lá): carregados uma vez por arquivo e clonados por
// fixture. Enquanto carregam (ou se falharem), vale o corpo procedural
const modelFiles: Partial<Record<FixtureKind, string>> = { movingHead: 'moving_head', scanner: 'scanner', colorChanger: 'par', dimmer: 'par', strobe: 'strobe' }
const models = new Map<string, Promise<{ scene: Object3D; size: Vector3 }>>()
const modelsURL = `${useRuntimeConfig().app.baseURL}models/`
function loadModel(name: string) {
  if (!models.has(name)) {
    models.set(name, new GLTFLoader().loadAsync(`${modelsURL}${name}.glb`).then(({ scene }) => ({ scene, size: new Box3().setFromObject(scene).getSize(new Vector3()) })))
  }
  return models.get(name)!
}

// Troca o corpo procedural (`shell`) pelo modelo, levando o feixe e a lente para o nó `lens`
function swapModel(look: Look, shell: Group, { scene, size }: { scene: Object3D; size: Vector3 }) {
  if (!looks.includes(look)) return // a cena foi remontada antes de o modelo chegar
  const model = scene.clone(true)
  // Geometria comum a todos os clones (`shared`: o dispose não a apaga); o material é o da fixture (realce da escolha)
  model.traverse((o) => {
    if ((o as Mesh).isMesh) (o as Mesh).material = look.body
    o.userData = { id: look.fixture.id, shared: true }
  })
  // Tamanho físico da fixture, como o QLC+: escala por igual pelo menor fator dos três eixos
  const p = findDef(project.value, look.fixture.defId)?.physical
  const k = p && p.widthMm > 0 && p.heightMm > 0 && p.depthMm > 0 ? Math.min(p.widthMm / size.x, p.heightMm / size.y, p.depthMm / size.z) / 1000 : 1
  const wrap = new Group() // a raiz do modelo guarda a translação (e a rotação) que o pendura: escala-se por fora
  wrap.scale.setScalar(k)
  wrap.add(model)
  const lens = model.getObjectByName('lens')!
  if (look.pan) {
    look.pan = model.getObjectByName('arm')
    look.tilt = model.getObjectByName('head')
    look.mirror = findDef(project.value, look.fixture.defId)?.kind === 'scanner'
  }
  if (look.fan) {
    look.fan.position.set(0, 0, 0)
    look.fan.scale.setScalar(1 / k) // o alcance do cone é em metros do palco
    lens.add(look.fan)
  }
  if (look.glass) {
    look.glass.position.set(0, 0, 0)
    lens.add(look.glass)
  }
  look.group.remove(shell)
  shell.traverse((o) => (o as Mesh).geometry?.dispose()) // o material `body` continua no modelo
  look.group.add(wrap)
}

function build() {
  const segments = { low: 12, medium: 24, high: 48 }[prefs.value.quality]
  if (!gobos.length) gobos = goboTextures() // uma vez, para todas as fixtures
  const scene = new Group()
  const { width, depth } = stage.value
  const floor = new Mesh(new PlaneGeometry(width, depth), new MeshStandardMaterial({ color: '#202328', roughness: 0.45, metalness: 0.25 }))
  floor.rotation.x = -Math.PI / 2
  const grid = new GridHelper(Math.max(width, depth), Math.max(width, depth), '#3a3f45', '#25282c')
  grid.position.y = 0.002
  scene.add(floor, grid)

  looks = []
  pieces = []
  pickables = { fixtures: [], elements: [] }
  project.value.fixtures.forEach((fixture) => {
    const def = findDef(project.value, fixture.defId)
    const kind = def?.kind ?? 'other'
    const group = new Group()
    const shell = new Group() // o corpo procedural, trocado pelo modelo quando ele chega
    group.add(shell)
    const body = new MeshStandardMaterial({ color: '#50555c', metalness: 0.3, roughness: 0.6 })
    const channels = modeChannels(project.value, fixture)
    const channel = (pick: (c: (typeof channels)[number]) => boolean): Channel | undefined => {
      const index = channels.findIndex((c) => !c.fine && pick(c))
      return index < 0 ? undefined : { index, caps: channels[index]!.capabilities }
    }
    const look: Look = {
      fixture, group, body, pixels: [], cones: [], spots: [], goboAngle: 0, radius: 0,
      shutter: channels.find((c) => c.attribute === 'shutter')?.capabilities ?? [],
      // A rotação do prisma pode vir num segundo canal "prism" (o Beam 7R tem "Prism Rotation")
      prism: channel((c) => c.attribute === 'prism' && !spinName.test(c.name)),
      prismSpin: channel((c) => c.attribute === 'prism' && spinName.test(c.name)),
      gobo: channel((c) => c.attribute === 'gobo'),
      goboSpin: channel((c) => c.attribute === 'goboRotation'),
    }
    const box = (w: number, h: number, d: number, y = 0) => {
      const mesh = new Mesh(new BoxGeometry(w, h, d), body)
      mesh.position.y = y
      return mesh
    }
    // Feixe saindo da face da lente, apontando para baixo (-Y), e a lente que brilha
    const addBeam = (parent: Object3D, y: number, radius: number) => {
      look.radius = radius
      look.beam = beamMaterial()
      look.spotMat = spotMaterial()
      // A geometria nasce uma vez, para os 8 cones do leque; o ângulo e o alcance ao vivo vão no shader (onLoop)
      const cone = new CylinderGeometry(1, 1, 1, segments, 1, true).translate(0, -0.5, 0)
      const disc = new PlaneGeometry(2, 2).rotateX(-Math.PI / 2)
      look.fan = new Group()
      look.fan.position.y = y
      for (let k = 0; k < FACETS; k++) {
        const mesh = new Mesh(cone, look.beam)
        mesh.rotation.order = 'YXZ' // inclina no X e depois gira em volta do eixo
        mesh.raycast = () => {} // o feixe não é tocável
        const spot = new Mesh(disc, look.spotMat)
        spot.raycast = () => {}
        look.fan.add(mesh)
        look.cones.push(mesh)
        look.spots.push(spot)
        scene.add(spot)
      }
      look.lens = new MeshBasicMaterial({ color: '#000' })
      look.glass = new Mesh(new CylinderGeometry(radius, radius, 0.01, segments), look.lens)
      look.glass.position.y = y
      parent.add(look.fan, look.glass)
    }
    if (kind === 'movingHead' || kind === 'scanner') {
      look.pan = new Group()
      look.tilt = new Group()
      look.pan.position.y = -0.12
      look.tilt.position.y = -0.3
      look.pan.add(box(0.3, 0.04, 0.06, -0.12), look.tilt)
      look.tilt.add(box(0.22, 0.28, 0.22))
      addBeam(look.tilt, -0.15, 0.05)
      shell.add(box(0.35, 0.12, 0.35, -0.06), look.pan)
    } else if (kind === 'colorChanger' || kind === 'dimmer') {
      shell.add(new Mesh(new CylinderGeometry(0.12, 0.1, 0.25, segments).translate(0, -0.125, 0), body))
      addBeam(shell, -0.26, 0.1) // o PAR sai da face inteira da lente
    } else if (kind === 'ledBar') {
      const length = (def?.physical.widthMm ?? 1000) / 1000
      const heads = Math.max(1, modeHeads(fixture).length)
      group.add(box(length, 0.08, 0.08))
      for (let i = 0; i < heads; i++) {
        const material = new MeshBasicMaterial({ color: '#000' })
        const pixel = new Mesh(new BoxGeometry((length / heads) * 0.8, 0.05, 0.01), material)
        pixel.position.set(-length / 2 + (length / heads) * (i + 0.5), 0, 0.045)
        look.pixels.push(material)
        group.add(pixel)
      }
    } else if (kind === 'strobe') {
      // O strobo também tem feixe (um banho largo); a lente é retangular
      addBeam(shell, -0.105, 0.14)
      look.glass!.geometry = new BoxGeometry(0.36, 0.01, 0.14)
      shell.add(box(0.4, 0.2, 0.15))
    } else {
      group.add(box(0.45, 0.3, 0.25, 0.15))
    }
    const file = modelFiles[kind]
    if (file) loadModel(file).then((model) => swapModel(look, shell, model), () => {}) // sem o modelo, fica o procedural
    group.traverse((o) => {
      if ((o as Mesh).isMesh) o.userData.id = fixture.id
    })
    group.userData.id = fixture.id
    pickables.fixtures.push(group)
    looks.push(look)
    scene.add(group)
  })

  stage.value.elements.forEach((element) => {
    const { group, materials } = buildElement(element.kind, segments)
    group.traverse((o) => (o.userData.id = element.id))
    materials.forEach((m) => (m.emissiveIntensity = 0.35))
    pieces.push({ id: element.id, group, materials })
    pickables.elements.push(group)
    scene.add(group)
  })
  return scene
}

const modeHeads = (f: Fixture) => findDef(project.value, f.defId)?.modes.find((m) => m.name === f.mode)?.heads ?? []

function dispose(o: Object3D) {
  o.traverse((x) => {
    const mesh = x as Mesh
    if (!x.userData.shared) mesh.geometry?.dispose()
    ;(Array.isArray(mesh.material) ? mesh.material : [mesh.material]).forEach((m) => m?.dispose())
  })
}

// Remonta quando mudam as fixtures, o tamanho do palco ou a qualidade; posições valem a cada quadro
const shape = computed(() => JSON.stringify([
  project.value.fixtures.map((f) => [f.id, f.defId, f.mode]),
  stage.value.elements.map((e) => [e.id, e.kind]),
  stage.value.width, stage.value.depth, prefs.value.quality]))
watch(shape, () => {
  dispose(root.value)
  root.value = build()
}, { immediate: true })
onBeforeUnmount(() => {
  dispose(root.value)
  controls?.dispose()
})

// ---------- Cor, intensidade e movimento a partir da saída ----------
const attributeValues = (f: Fixture, frame: Uint8Array | undefined, channels: number[] = []) => {
  const values: Partial<Record<string, number>> = {}
  modeChannels(project.value, f).forEach((c, i) => {
    if (c.fine || (channels.length && !channels.includes(i)) || values[c.attribute] !== undefined) return
    values[c.attribute] = frame?.[f.address - 1 + i] ?? 0
  })
  return values
}

function colorOf(f: Fixture, v: Partial<Record<string, number>>) {
  const c = new Color(1, 1, 1)
  if (v.red !== undefined || v.green !== undefined || v.blue !== undefined) {
    const w = (v.white ?? 0) / 255
    const a = (v.amber ?? 0) / 255
    c.setRGB(Math.min(1, (v.red ?? 0) / 255 + w + a), Math.min(1, (v.green ?? 0) / 255 + w + a * 0.5), Math.min(1, (v.blue ?? 0) / 255 + w))
  } else if (v.colorWheel !== undefined) {
    const wheel = modeChannels(project.value, f).find((ch) => ch.attribute === 'colorWheel')
    c.set(wheel?.capabilities.find((cap) => v.colorWheel! >= cap.min && v.colorWheel! <= cap.max)?.color ?? '#ffffff')
  }
  // Sem dimmer, o brilho vem das próprias cores
  return { color: c, intensity: v.dimmer !== undefined ? v.dimmer / 255 : Math.max(c.r, c.g, c.b) }
}

// Visibilidade só para arrumar o palco: tudo (por tipo) ou uma peça escondida some da tela e não se escolhe
const shown = (layer: 'fixtures' | 'elements', id: string) => !props.hidden?.[layer] && !props.hidden?.ids.includes(id)
const pickable = (layer: 'fixtures' | 'elements') => pickables[layer].filter((o) => o.visible)

let last = performance.now() / 1000
function onLoop() {
  const frames = props.source === 'preview' ? previewOutput.value : output.value
  // As rotações (prisma e gobo) se acumulam com o tempo do quadro
  const now = performance.now() / 1000
  const dt = Math.min(0.1, now - last)
  last = now
  looks.forEach((look) => {
    const f = look.fixture
    look.group.visible = shown('fixtures', f.id)
    placeObject(look.group, drag?.positions.get(f.id) ?? stagePosition(project.value, f))
    look.body.emissive.set(selected.value.includes(f.id) ? '#ff8000' : '#000000')
    look.body.emissiveIntensity = 0.35
    const frame = frames[f.universe]
    const v = attributeValues(f, frame)
    const { color, intensity } = colorOf(f, v)
    const physical = findDef(project.value, f.defId)?.physical
    if (look.pan) look.pan.rotation.y = (((v.pan ?? 128) / 255 - 0.5) * (physical?.panMaxDeg ?? 540) * Math.PI) / 180
    const tilt = (((v.tilt ?? 128) / 255 - 0.5) * (physical?.tiltMaxDeg ?? 270) * Math.PI) / 180
    if (look.tilt) look.tilt.rotation.x = look.mirror ? tilt / 2 : tilt
    // No espelho, o feixe gira o dobro: a outra metade vai no leque (o eixo X da lente é o da cabeça)
    if (look.mirror && look.fan) look.fan.rotation.x = tilt / 2
    const profile = beamLook(findDef(project.value, f.defId))
    if (look.beam && look.fan && look.spotMat && profile) {
      const state = beamState(v, profile, look.shutter, now)
      const read = (c: Channel) => frame?.[f.address - 1 + c.index] ?? 0
      // Prisma: N feixes em leque, girando no eixo pelo canal de rotação (ou devagar, se a capability fala em rotação)
      const prism = look.prism ? prismState(look.prism.caps, read(look.prism)) : { facets: 0, spin: 0 }
      const n = Math.max(1, prism.facets)
      look.fan.rotation.y += (look.prismSpin ? spinSpeed(look.prismSpin.caps, read(look.prismSpin)) : prism.spin) * dt
      const gobo = look.gobo ? goboIndex(look.gobo.caps, read(look.gobo), gobos.length) : -1
      if (look.goboSpin) look.goboAngle += spinSpeed(look.goboSpin.caps, read(look.goboSpin)) * dt
      // A luz se divide entre os feixes do prisma; o gobo tira parte do brilho do cone
      const lit = color.clone().multiplyScalar((intensity * state.flicker * profile.brightness) / n)
      const on = Math.max(lit.r, lit.g, lit.b) > 0.01 && look.group.visible
      const half = Math.tan((state.angle * Math.PI) / 360)
      look.spotMat.uniforms.uSoft!.value = state.softness
      look.spotMat.uniforms.uHasGobo!.value = gobo < 0 ? 0 : 1
      look.spotMat.uniforms.uGobo!.value = gobos[Math.max(0, gobo)]
      look.spotMat.uniforms.uGoboAngle!.value = look.goboAngle
      look.cones.forEach((cone, k) => {
        const spot = look.spots[k]!
        cone.visible = k < n
        spot.visible = false
        if (k >= n) return
        cone.rotation.set(n > 1 ? FAN : 0, (2 * Math.PI * k) / n, 0)
        // O eixo do feixe (a lente e o sentido de -Y) cruza o chão a t metros; o cone termina ali, ou no alcance
        const origin = cone.getWorldPosition(new Vector3())
        const dir = new Vector3(0, -1, 0).applyQuaternion(cone.getWorldQuaternion(new Quaternion()))
        const t = dir.y < -0.01 ? origin.y / -dir.y : Infinity
        const length = Math.min(profile.reach, t)
        // Só o comprimento vai na escala (Y); o raio (lente + abertura) é do shader
        cone.scale.set(1, Math.max(0.01, length), 1)
        // Mancha: elipse esticada pela inclinação; a luz se espalha pela área (o banho do PAR é largo e fraco,
        // a do Beam pequena e definida) e cai um pouco com a distância
        spot.visible = on && t <= profile.reach
        if (!spot.visible) return
        const radius = look.radius + t * half
        spot.position.set(origin.x + dir.x * t, 0.01, origin.z + dir.z * t)
        spot.rotation.y = Math.atan2(dir.x, dir.z)
        spot.scale.set(radius, 1, radius / Math.max(0.25, -dir.y))
        look.spotMat!.uniforms.uColor!.value.copy(lit).multiplyScalar(Math.min(1.5, Math.max(0.15, 0.5 / radius)) * (1 - (0.5 * t) / profile.reach))
      })
      look.beam.uniforms.uSoft!.value = state.softness
      look.beam.uniforms.uCore!.value = profile.core
      look.beam.uniforms.uLens!.value = look.radius
      look.beam.uniforms.uHalf!.value = half
      look.beam.uniforms.uReach!.value = profile.reach
      look.beam.uniforms.uColor!.value.copy(lit).multiplyScalar(prefs.value.smoke * (gobo < 0 ? 1 : 0.6))
    }
    look.lens?.color.copy(color).multiplyScalar(intensity)
    modeHeads(f).forEach((head, h) => {
      const pixel = colorOf(f, attributeValues(f, frame, head))
      look.pixels[h]?.color.copy(pixel.color).multiplyScalar(pixel.intensity)
    })
  })
  pieces.forEach((piece) => {
    const p = drag?.positions.get(piece.id) ?? elementsById.value.get(piece.id)
    piece.group.visible = shown('elements', piece.id)
    if (p) placeObject(piece.group, p)
    piece.materials.forEach((m) => m.emissive.set(selected.value.includes(piece.id) ? '#ff8000' : '#000000'))
  })
  controls?.update()
}

// ---------- Câmera e vistas ----------
function setView() {
  const { width, depth, height } = stage.value
  const presets = {
    perspective: [new Vector3(0, height, depth / 2 + 6.5), new Vector3(0, 1.8, 0)],
    top: [new Vector3(0, Math.max(width, depth) * 1.4, 0.01), new Vector3(0, 0, 0)],
    front: [new Vector3(0, height / 2, depth / 2 + 11), new Vector3(0, height / 2.5, 0)],
    side: [new Vector3(width / 2 + 11, height / 2, 0), new Vector3(0, height / 2.5, 0)],
  }
  const [position, target] = presets[props.view]
  camera.position.copy(position!)
  if (!controls) return
  controls.target.copy(target!)
  // Só a vista 3D gira; nas outras, o botão do meio (ou um dedo) desloca
  const rotate = props.view === 'perspective'
  controls.enableRotate = rotate
  controls.mouseButtons.MIDDLE = rotate ? MOUSE.ROTATE : MOUSE.PAN
  controls.touches.ONE = rotate ? TOUCH.ROTATE : TOUCH.PAN
  controls.update()
}
watch(() => props.view, setView)

let resize: ResizeObserver | undefined
onMounted(() => {
  controls = new OrbitControls(camera, wrapper.value!)
  // O botão esquerdo é só da escolha; a câmera fica no do meio e no direito, e o zoom só na roda
  controls.mouseButtons = { LEFT: null, MIDDLE: MOUSE.ROTATE, RIGHT: MOUSE.PAN }
  controls.maxDistance = 60
  setView()
  resize = new ResizeObserver(([entry]) => {
    camera.aspect = entry!.contentRect.width / Math.max(1, entry!.contentRect.height)
    camera.updateProjectionMatrix()
  })
  resize.observe(wrapper.value!)
})
onBeforeUnmount(() => resize?.disconnect())

// ---------- Escolher e arrastar fixtures e elementos ----------
// Shift + clique soma ou tira da escolha; arrastar no vazio (mouse) abre uma caixa que escolhe o que ficar dentro
// (com Shift, soma à escolha). Arrastar um dos escolhidos move todos pelo mesmo deslocamento.
const raycaster = new Raycaster()
type Drag = { id: string; starts: Map<string, Placement>; positions: Map<string, Placement>; offset: Vector3; plane: Plane; moved: boolean }
let drag: Drag | undefined
// `collapse`: clique sem arrastar num dos vários escolhidos fica só com ele
// e `empty`: o toque começou no vazio (a tela pode se redimensionar até o fim do gesto, então não se confere de novo)
let downAt: { x: number; y: number; empty: boolean; collapse?: string } | undefined
const box = ref<{ x0: number; y0: number; x1: number; y1: number }>()
let boxBase: string[] = []

function ray(e: PointerEvent) {
  const rect = wrapper.value!.getBoundingClientRect()
  raycaster.setFromCamera(new Vector2(((e.clientX - rect.left) / rect.width) * 2 - 1, -((e.clientY - rect.top) / rect.height) * 2 + 1), camera)
  return raycaster
}
// O que está sob o ponteiro, entre as coisas da aba aberta
const hitId = (e: PointerEvent) => props.layer && (ray(e).intersectObjects(pickable(props.layer), true)[0]?.object.userData.id as string | undefined)

const placementOf = (id: string): Placement =>
  props.layer === 'fixtures' ? { ...stagePosition(project.value, project.value.fixtures.find((f) => f.id === id)!) } : { ...elementsById.value.get(id)! }

// Na fase de captura, antes dos controles da câmera: sobre uma fixture ou um elemento, o gesto é dele.
// Só o botão esquerdo (ou o toque) escolhe; o do meio e o direito são sempre da câmera
function down(e: PointerEvent) {
  if (e.button !== 0) return
  const rect = wrapper.value!.getBoundingClientRect()
  const id = hitId(e)
  downAt = { x: e.clientX, y: e.clientY, empty: !id }
  if (!id) {
    if (props.layer && e.pointerType === 'mouse') {
      boxBase = e.shiftKey ? selected.value : []
      box.value = { x0: e.clientX - rect.left, y0: e.clientY - rect.top, x1: e.clientX - rect.left, y1: e.clientY - rect.top }
      wrapper.value!.setPointerCapture(e.pointerId)
    }
    return
  }
  const ids = selected.value
  const wasSelected = ids.includes(id)
  if (e.shiftKey) {
    selected.value = wasSelected ? ids.filter((x) => x !== id) : [...ids, id]
    return
  }
  if (wasSelected && ids.length > 1) downAt.collapse = id
  else selected.value = [id]
  if (!props.editing || (e.pointerType !== 'mouse' && !wasSelected)) return
  const moving = wasSelected ? ids : [id]
  const starts = new Map(moving.map((m) => [m, placementOf(m)]))
  const position = starts.get(id)!
  // A fixture (ou o elemento) anda num plano de frente para a câmera: chão no topo, X e altura na frente, Z e altura na
  // lateral; na vista 3D, o plano acompanha a câmera (girando a câmera, ela vai em qualquer direção)
  const normal = { top: new Vector3(0, 1, 0), front: new Vector3(0, 0, 1), side: new Vector3(1, 0, 0), perspective: camera.getWorldDirection(new Vector3()) }[props.view]
  const plane = new Plane().setFromNormalAndCoplanarPoint(normal, new Vector3(position.x, position.y, position.z))
  const point = ray(e).ray.intersectPlane(plane, new Vector3())
  if (!point) return
  drag = { id, starts, positions: new Map(starts), offset: point.sub(new Vector3(position.x, position.y, position.z)), plane, moved: false }
  controls!.enabled = false
  wrapper.value!.setPointerCapture(e.pointerId)
}

// O que tem a origem dentro da caixa, na tela
function inBox() {
  const rect = wrapper.value!.getBoundingClientRect()
  const { x0, y0, x1, y1 } = box.value!
  const [left, right, top, bottom] = [Math.min(x0, x1), Math.max(x0, x1), Math.min(y0, y1), Math.max(y0, y1)]
  return pickable(props.layer!).filter((o) => {
    const p = o.getWorldPosition(new Vector3()).project(camera)
    const [x, y] = [((p.x + 1) / 2) * rect.width, ((1 - p.y) / 2) * rect.height]
    return p.z < 1 && x >= left && x <= right && y >= top && y <= bottom
  }).map((o) => o.userData.id as string)
}

function move(e: PointerEvent) {
  if (box.value) {
    const rect = wrapper.value!.getBoundingClientRect()
    box.value = { ...box.value, x1: e.clientX - rect.left, y1: e.clientY - rect.top }
    selected.value = [...new Set([...boxBase, ...inBox()])]
    return
  }
  if (!drag) return
  const point = ray(e).ray.intersectPlane(drag.plane, new Vector3())
  if (!point) return
  // Encaixe de 10 cm na peça que está sob o ponteiro; as outras andam o mesmo tanto. Nada desce abaixo do chão
  const snap = (v: number) => Math.round(v * 10) / 10
  const base = drag.starts.get(drag.id)!
  const delta = { x: snap(point.x - drag.offset.x) - base.x, y: snap(point.y - drag.offset.y) - base.y, z: snap(point.z - drag.offset.z) - base.z }
  delta.y = Math.max(delta.y, -Math.min(...[...drag.starts.values()].map((p) => p.y)))
  const round = (v: number) => Math.round(v * 1000) / 1000
  drag.starts.forEach((p, id) => drag!.positions.set(id, { ...p, x: round(p.x + delta.x), y: round(p.y + delta.y), z: round(p.z + delta.z) }))
  drag.moved = true
}

function up(e: PointerEvent) {
  const click = downAt && Math.hypot(e.clientX - downAt.x, e.clientY - downAt.y) < 4
  if (drag?.moved) {
    const { positions } = drag
    const { fixtures, elements } = stage.value
    send({
      op: 'saveStage',
      stage: props.layer === 'fixtures'
        ? { ...stage.value, fixtures: [...fixtures.filter((p) => !positions.has(p.fixture)), ...[...positions].map(([fixture, p]) => ({ ...p, fixture }))] }
        : { ...stage.value, elements: elements.map((x) => ({ ...x, ...positions.get(x.id) })) },
    })
  } else if (click && downAt?.collapse) {
    selected.value = [downAt.collapse]
  } else if (click && downAt?.empty && !e.shiftKey) {
    selected.value = [] // toque no vazio solta a escolha
  }
  box.value = undefined
  drag = undefined
  downAt = undefined
  if (controls) controls.enabled = true
}
</script>

<template>
  <div
    ref="wrapper"
    class="stage"
    @pointerdown.capture="down"
    @mousedown.middle.prevent
    @pointermove="move"
    @pointerup="up"
    @pointercancel="up"
  >
    <!-- mousedown.middle.prevent: o botão do meio é da câmera, sem a rolagem automática do navegador -->
    <TresCanvas :camera="camera" clear-color="#050607" :dpr="{ low: 1, medium: 1.5, high: 2 }[prefs.quality]" @loop="onLoop">
      <TresAmbientLight :intensity="1" />
      <TresDirectionalLight :position="[5, 10, 5]" :intensity="1.2" />
      <primitive :key="shape" :object="root" />
      <View3dPost :quality="prefs.quality" />
    </TresCanvas>
    <div
      v-if="box"
      class="pointer-events-none absolute border border-dashed border-select bg-select/15"
      :style="{ left: `${Math.min(box.x0, box.x1)}px`, top: `${Math.min(box.y0, box.y1)}px`, width: `${Math.abs(box.x1 - box.x0)}px`, height: `${Math.abs(box.y1 - box.y0)}px` }"
    />
  </div>
</template>

<style scoped>
.stage {
  position: relative;
  overflow: hidden;
  border: 1px solid #000;
  border-radius: 8px;
  touch-action: none;
}
</style>

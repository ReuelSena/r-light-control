<script setup lang="ts">
import { TresCanvas } from '@tresjs/core'
import {
  AdditiveBlending, BoxGeometry, Color, ConeGeometry, CylinderGeometry, DoubleSide, GridHelper, Group, MOUSE, Mesh, MeshBasicMaterial,
  MeshStandardMaterial, PerspectiveCamera, Plane, PlaneGeometry, Raycaster, ShaderMaterial, TOUCH, Vector2, Vector3, type Object3D,
} from 'three'
import { OrbitControls } from 'three/examples/jsm/controls/OrbitControls.js'
import type { Fixture } from '~/types/generated/Fixture'

// Palco em 3D (E12): as fixtures acendem com a saída dos universos (simulada na F2), com feixes
// aditivos como no teste da F1. Vistas 3D, topo, frente e lateral. Câmera: com o mouse, o botão do meio
// gira (3D) ou desloca, o direito desloca e a roda aproxima; no toque, um dedo gira ou desloca e a pinça
// aproxima. Os elementos de palco (truss, caixas…) complementam a prévia.
// O botão esquerdo (ou o toque) só escolhe, e só o que é da aba aberta (`layer`): fixtures ou elementos.
// Com "Editar posição" (`editing`), arrastar o escolhido o move em qualquer vista (no toque, depois de
// escolher). Modelos de verdade, gobos e zoom ficam para a F15.
const props = defineProps<{ view: 'perspective' | 'top' | 'front' | 'side'; editing: boolean; layer?: 'fixtures' | 'elements' }>()
const selected = defineModel<string>('selected')
const { state, output, send } = useEngine()
const prefs = usePrefs()
const project = computed(() => state.value.project)
const stage = computed(() => project.value.stage)
const elementsById = computed(() => new Map(stage.value.elements.map((e) => [e.id, e])))
const BEAM = 8 // comprimento do feixe (m)

const wrapper = ref<HTMLElement>()
const camera = new PerspectiveCamera(50, 1, 0.1, 200)
let controls: OrbitControls | undefined

// ---------- Montagem da cena ----------
type Look = { fixture: Fixture; group: Group; body: MeshStandardMaterial; pan?: Object3D; tilt?: Object3D; beam?: ShaderMaterial; lens?: MeshBasicMaterial; pixels: MeshBasicMaterial[] }
type Piece = { id: string; group: Group; materials: MeshStandardMaterial[] }
const root = shallowRef(new Group())
let looks: Look[] = []
let pieces: Piece[] = []
let pickables: Record<'fixtures' | 'elements', Object3D[]> = { fixtures: [], elements: [] }

function beamMaterial() {
  return new ShaderMaterial({
    transparent: true,
    depthWrite: false,
    blending: AdditiveBlending,
    side: DoubleSide,
    uniforms: { uLength: { value: BEAM }, uColor: { value: new Color(0, 0, 0) } },
    vertexShader: /* glsl */ `
      uniform float uLength;
      varying float vAlong;
      varying float vFacing;
      void main() {
        vAlong = -position.y / uLength;
        vec4 mv = modelViewMatrix * vec4(position, 1.0);
        vFacing = abs(dot(normalize(normalMatrix * normal), normalize(-mv.xyz)));
        gl_Position = projectionMatrix * mv;
      }`,
    fragmentShader: /* glsl */ `
      uniform vec3 uColor;
      varying float vAlong;
      varying float vFacing;
      void main() {
        gl_FragColor = vec4(uColor * pow(1.0 - vAlong, 2.0) * pow(vFacing, 1.5), 1.0);
      }`,
  })
}

function build() {
  const segments = { low: 12, medium: 24, high: 48 }[prefs.value.quality]
  const scene = new Group()
  const { width, depth } = stage.value
  const floor = new Mesh(new PlaneGeometry(width, depth), new MeshStandardMaterial({ color: '#202328' }))
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
    const body = new MeshStandardMaterial({ color: '#50555c', metalness: 0.3, roughness: 0.6 })
    const look: Look = { fixture, group, body, pixels: [] }
    const box = (w: number, h: number, d: number, y = 0) => {
      const mesh = new Mesh(new BoxGeometry(w, h, d), body)
      mesh.position.y = y
      return mesh
    }
    // Feixe com a ponta na lente, apontando para baixo (-Y), e a lente que brilha
    const addBeam = (parent: Object3D, y: number) => {
      const lens = physicalLens(fixture)
      look.beam = beamMaterial()
      const cone = new Mesh(new ConeGeometry(BEAM * Math.tan((lens * Math.PI) / 360), BEAM, segments, 1, true).translate(0, -BEAM / 2, 0), look.beam)
      cone.position.y = y
      cone.raycast = () => {} // o feixe não é tocável
      look.lens = new MeshBasicMaterial({ color: '#000' })
      const glass = new Mesh(new CylinderGeometry(0.09, 0.09, 0.01, segments), look.lens)
      glass.position.y = y
      parent.add(cone, glass)
    }
    if (kind === 'movingHead' || kind === 'scanner') {
      look.pan = new Group()
      look.tilt = new Group()
      look.pan.position.y = -0.12
      look.tilt.position.y = -0.3
      look.pan.add(box(0.3, 0.04, 0.06, -0.12), look.tilt)
      look.tilt.add(box(0.22, 0.28, 0.22))
      addBeam(look.tilt, -0.15)
      group.add(box(0.35, 0.12, 0.35, -0.06), look.pan)
    } else if (kind === 'colorChanger' || kind === 'dimmer') {
      group.add(new Mesh(new CylinderGeometry(0.12, 0.1, 0.25, segments).translate(0, -0.125, 0), body))
      addBeam(group, -0.26)
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
      look.lens = new MeshBasicMaterial({ color: '#000' })
      const glass = new Mesh(new BoxGeometry(0.36, 0.01, 0.14), look.lens)
      glass.position.y = -0.105
      group.add(box(0.4, 0.2, 0.15), glass)
    } else {
      group.add(box(0.45, 0.3, 0.25, 0.15))
    }
    group.traverse((o) => {
      if ((o as Mesh).isMesh) o.userData.id = fixture.id
    })
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

const physicalLens = (f: Fixture) => {
  const p = findDef(project.value, f.defId)?.physical
  return p ? (p.lensMinDeg + p.lensMaxDeg) / 2 || 20 : 20
}
const modeHeads = (f: Fixture) => findDef(project.value, f.defId)?.modes.find((m) => m.name === f.mode)?.heads ?? []

function dispose(o: Object3D) {
  o.traverse((x) => {
    const mesh = x as Mesh
    mesh.geometry?.dispose()
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

function onLoop() {
  const frames = output.value
    looks.forEach((look) => {
    const f = look.fixture
    placeObject(look.group, drag?.id === f.id ? drag.position : stagePosition(project.value, f))
    look.body.emissive.set(selected.value === f.id ? '#ff8000' : '#000000')
    look.body.emissiveIntensity = 0.35
    const frame = frames[f.universe]
    const v = attributeValues(f, frame)
    const { color, intensity } = colorOf(f, v)
    const physical = findDef(project.value, f.defId)?.physical
    if (look.pan) look.pan.rotation.y = (((v.pan ?? 128) / 255 - 0.5) * (physical?.panMaxDeg ?? 540) * Math.PI) / 180
    if (look.tilt) look.tilt.rotation.x = (((v.tilt ?? 128) / 255 - 0.5) * (physical?.tiltMaxDeg ?? 270) * Math.PI) / 180
    look.beam?.uniforms.uColor!.value.copy(color).multiplyScalar(intensity * prefs.value.smoke)
    look.lens?.color.copy(color).multiplyScalar(intensity)
    modeHeads(f).forEach((head, h) => {
      const pixel = colorOf(f, attributeValues(f, frame, head))
      look.pixels[h]?.color.copy(pixel.color).multiplyScalar(pixel.intensity)
    })
  })
  pieces.forEach((piece) => {
    const p = drag?.id === piece.id ? drag.position : elementsById.value.get(piece.id)
    if (p) placeObject(piece.group, p)
    piece.materials.forEach((m) => m.emissive.set(selected.value === piece.id ? '#ff8000' : '#000000'))
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
const raycaster = new Raycaster()
let drag: { id: string; position: Placement; offset: Vector3; plane: Plane; moved: boolean } | undefined
let downAt: { x: number; y: number } | undefined

function ray(e: PointerEvent) {
  const box = wrapper.value!.getBoundingClientRect()
  raycaster.setFromCamera(new Vector2(((e.clientX - box.left) / box.width) * 2 - 1, -((e.clientY - box.top) / box.height) * 2 + 1), camera)
  return raycaster
}
// O que está sob o ponteiro, entre as coisas da aba aberta
const hitId = (e: PointerEvent) => props.layer && (ray(e).intersectObjects(pickables[props.layer], true)[0]?.object.userData.id as string | undefined)

// Na fase de captura, antes dos controles da câmera: sobre uma fixture ou um elemento, o gesto é dele.
// Só o botão esquerdo (ou o toque) escolhe; o do meio e o direito são sempre da câmera
function down(e: PointerEvent) {
  if (e.button !== 0) return
  downAt = { x: e.clientX, y: e.clientY }
  const id = hitId(e)
  if (!id) return
  const wasSelected = selected.value === id
  selected.value = id
  if (!props.editing || (e.pointerType !== 'mouse' && !wasSelected)) return
  const position = props.layer === 'fixtures' ? { ...stagePosition(project.value, project.value.fixtures.find((f) => f.id === id)!) } : { ...elementsById.value.get(id)! }
  // A fixture (ou o elemento) anda num plano de frente para a câmera: chão no topo, X e altura na frente, Z e altura na
  // lateral; na vista 3D, o plano acompanha a câmera (girando a câmera, ela vai em qualquer direção)
  const normal = { top: new Vector3(0, 1, 0), front: new Vector3(0, 0, 1), side: new Vector3(1, 0, 0), perspective: camera.getWorldDirection(new Vector3()) }[props.view]
  const plane = new Plane().setFromNormalAndCoplanarPoint(normal, new Vector3(position.x, position.y, position.z))
  const point = ray(e).ray.intersectPlane(plane, new Vector3())
  if (!point) return
  drag = { id, position, offset: point.sub(new Vector3(position.x, position.y, position.z)), plane, moved: false }
  controls!.enabled = false
  wrapper.value!.setPointerCapture(e.pointerId)
}

function move(e: PointerEvent) {
  if (!drag) return
  const point = ray(e).ray.intersectPlane(drag.plane, new Vector3())
  if (!point) return
  // Encaixe de 10 cm; nada desce abaixo do chão
  const snap = (v: number) => Math.round(v * 10) / 10
  drag.position.x = snap(point.x - drag.offset.x)
  drag.position.y = Math.max(0, snap(point.y - drag.offset.y))
  drag.position.z = snap(point.z - drag.offset.z)
  drag.moved = true
}

function up(e: PointerEvent) {
  if (drag?.moved) {
    const { id, position } = drag
    const { fixtures, elements } = stage.value
    send({
      op: 'saveStage',
      stage: props.layer === 'fixtures'
        ? { ...stage.value, fixtures: [...fixtures.filter((p) => p.fixture !== id), { ...position, fixture: id }] }
        : { ...stage.value, elements: elements.map((x) => (x.id === id ? { ...x, ...position } : x)) },
    })
  } else if (!drag && downAt && Math.hypot(e.clientX - downAt.x, e.clientY - downAt.y) < 4 && !hitId(e)) {
    selected.value = undefined // toque no vazio solta a escolha
  }
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
    </TresCanvas>
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

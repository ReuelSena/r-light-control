<script setup lang="ts">
import { TresCanvas } from '@tresjs/core'
import { AdditiveBlending, BoxGeometry, Color, ConeGeometry, DoubleSide, InstancedMesh, MeshStandardMaterial, Object3D, ShaderMaterial } from 'three'

// Spike da F1: moving heads com feixes aditivos instanciados, para medir o fps no celular.
// O 3D de verdade é a F15; aqui só validamos a técnica (1 draw call para todos os feixes).
const { t } = useI18n()
const counts = [25, 50, 100]
const count = ref(50)
const fps = ref(0)
const BEAM_LENGTH = 9

// Feixe: cone aberto, mais forte perto da lente e nas bordas voltadas para a câmera suavizadas
const beamMaterial = new ShaderMaterial({
  transparent: true,
  depthWrite: false,
  blending: AdditiveBlending,
  side: DoubleSide,
  uniforms: { uLength: { value: BEAM_LENGTH } },
  vertexShader: /* glsl */ `
    uniform float uLength;
    varying float vAlong;
    varying float vFacing;
    varying vec3 vColor;
    void main() {
      vAlong = -position.y / uLength;
      vColor = instanceColor;
      vec4 mv = modelViewMatrix * instanceMatrix * vec4(position, 1.0);
      vec3 n = normalize(normalMatrix * mat3(instanceMatrix) * normal);
      vFacing = abs(dot(n, normalize(-mv.xyz)));
      gl_Position = projectionMatrix * mv;
    }`,
  fragmentShader: /* glsl */ `
    varying float vAlong;
    varying float vFacing;
    varying vec3 vColor;
    void main() {
      float glow = pow(1.0 - vAlong, 2.0) * pow(vFacing, 1.5);
      gl_FragColor = vec4(vColor * glow * 0.6, 1.0);
    }`,
})
const headMaterial = new MeshStandardMaterial({ color: '#2a2d31', metalness: 0.6, roughness: 0.4 })

function build(n: number) {
  // Cone com a ponta na origem, pendurado para baixo (-Y)
  const beams = new InstancedMesh(new ConeGeometry(1.2, BEAM_LENGTH, 32, 1, true).translate(0, -BEAM_LENGTH / 2, 0), beamMaterial, n)
  const heads = new InstancedMesh(new BoxGeometry(0.35, 0.45, 0.35), headMaterial, n)
  for (let i = 0; i < n; i++) beams.setColorAt(i, new Color().setHSL(i / n, 1, 0.5))
  // As instâncias se movem; o volume calculado na criação não vale mais
  beams.frustumCulled = false
  heads.frustumCulled = false
  return { beams, heads }
}

const scene = shallowRef(build(count.value))
watch(count, (n) => {
  scene.value.beams.geometry.dispose()
  scene.value.heads.geometry.dispose()
  scene.value = build(n)
})

const dummy = new Object3D()
let frames = 0
let lastFps = 0

function onLoop({ elapsed }: { elapsed: number }) {
  const { beams, heads } = scene.value
  const rows = Math.ceil(beams.count / 10)
  for (let i = 0; i < beams.count; i++) {
    dummy.position.set(((i % 10) - 4.5) * 1.6, 6, (Math.floor(i / 10) - (rows - 1) / 2) * 2)
    // Pan/tilt simulados, com fase diferente por fixture (como um EFX)
    dummy.rotation.set(Math.sin(elapsed * 1.3 + i * 0.4) * 0.6, Math.sin(elapsed * 0.7 + i * 0.25) * 1.2, 0, 'YXZ')
    dummy.updateMatrix()
    beams.setMatrixAt(i, dummy.matrix)
    heads.setMatrixAt(i, dummy.matrix)
  }
  beams.instanceMatrix.needsUpdate = true
  heads.instanceMatrix.needsUpdate = true

  frames++
  if (elapsed - lastFps >= 1) {
    fps.value = Math.round(frames / (elapsed - lastFps))
    frames = 0
    lastFps = elapsed
  }
}
</script>

<template>
  <div class="fixed inset-0">
    <TresCanvas clear-color="#050607" :dpr="[1, 1.5]" @loop="onLoop">
      <TresPerspectiveCamera :position="[0, 4, 16]" :look-at="[0, 2, 0]" :fov="50" />
      <TresAmbientLight :intensity="0.5" />
      <TresDirectionalLight :position="[5, 10, 5]" :intensity="1.2" />
      <primitive :key="`beams-${count}`" :object="scene.beams" />
      <primitive :key="`heads-${count}`" :object="scene.heads" />
      <TresMesh :rotation-x="-Math.PI / 2">
        <TresPlaneGeometry :args="[40, 40]" />
        <TresMeshStandardMaterial color="#1a1c1f" />
      </TresMesh>
    </TresCanvas>
    <header class="brushed absolute inset-x-0 top-0 flex items-center gap-2 px-3 py-1.5">
      <RlButton size="sm" @press="navigateTo('/dev/showcase')">{{ t('dev.back') }}</RlButton>
      <RlLcd>{{ t('dev.fps', { n: fps }) }}</RlLcd>
      <div class="ml-auto flex gap-1">
        <RlButton v-for="n in counts" :key="n" size="sm" :model-value="count === n" @press="count = n">
          {{ t('dev.fixtures', { n }) }}
        </RlButton>
      </div>
    </header>
  </div>
</template>

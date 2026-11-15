<script setup lang="ts">
import { useLoop, useTres } from '@tresjs/core'
import { NoToneMapping, Vector2 } from 'three'
import { EffectComposer } from 'three/examples/jsm/postprocessing/EffectComposer.js'
import { OutputPass } from 'three/examples/jsm/postprocessing/OutputPass.js'
import { RenderPass } from 'three/examples/jsm/postprocessing/RenderPass.js'
import { UnrealBloomPass } from 'three/examples/jsm/postprocessing/UnrealBloomPass.js'

// Pós-processamento da 3D View (F15b), dentro do TresCanvas: em qualidade média e alta, um bloom leve no lugar do
// render padrão; na baixa, nada. Sem tone mapping (o ACES puxava as cores vivas dos feixes para o branco): o
// OutputPass só converte para sRGB, como o render direto, então as três qualidades têm as mesmas cores.
// Não desenha nada na cena.
const props = defineProps<{ quality: 'low' | 'medium' | 'high' }>()
const { renderer, scene, camera, sizes } = useTres()
const { render } = useLoop()

let composer: EffectComposer | undefined
const stop = () => {
  composer?.dispose()
  composer = undefined
  renderer.toneMapping = NoToneMapping
  render((done) => {
    if (camera.value) renderer.render(scene.value, camera.value)
    done()
  })
}

watch(() => props.quality, (quality) => {
  stop()
  if (quality === 'low') return
  const gl = renderer as import('three').WebGLRenderer
  composer = new EffectComposer(gl)
  composer.setPixelRatio(gl.getPixelRatio())
  composer.setSize(sizes.width.value, sizes.height.value)
  // Bloom fraco, só no que passa de 0,9 de brilho (lentes acesas, cruzamentos de feixes): um brilho, não neon
  const base = new RenderPass(scene.value, camera.value!)
  composer.addPass(base)
  composer.addPass(new UnrealBloomPass(new Vector2(sizes.width.value, sizes.height.value), quality === 'high' ? 0.3 : 0.2, 0.3, 0.9))
  composer.addPass(new OutputPass())
  render((done) => {
    if (!composer || !camera.value) return
    base.camera = camera.value
    composer.render()
    done()
  })
}, { immediate: true })

watch([sizes.width, sizes.height], ([w, h]) => composer?.setSize(w, h))
onBeforeUnmount(stop)
</script>

<template>
  <TresGroup />
</template>

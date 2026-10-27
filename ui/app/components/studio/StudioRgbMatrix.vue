<script setup lang="ts">
import { Trash2, Upload } from 'lucide-vue-next'
import type { Function } from '~/types/generated/Function'
import type { RgbMatrix } from '~/types/generated/RgbMatrix'
import type { RgbScript } from '~/types/generated/RgbScript'

// Editor de RGB Matrix: grupo de fixtures (grade 2D), algoritmo (cor fixa ou um script do QLC+ do
// projeto), cores, canais controlados e tempos. A prévia animada é `StudioRgbMatrixPreview`, à parte: o
// relógio dela redesenharia o editor e repunha nos campos o valor salvo enquanto se digita.
const props = defineProps<{ fn: Extract<Function, { type: 'rgbMatrix' }> }>()
const { t } = useI18n()
const { state, send } = useEngine()
const project = computed(() => state.value.project)
// Num canal só (dimmer, branco, âmbar, UV), a cor vale pelo brilho dela: branco é 100%
const palette = computed(() => (props.fn.control === 'rgb' ? [...tagColors, '#000000'] : widgetColors.grays))

const script = computed(() => matrixScript(props.fn, project.value.scripts))
const colors = computed(() => matrixColors(props.fn, project.value.scripts))
// Outras criações usam o script? Então ele não sai daqui
const sharedScript = computed(() =>
  project.value.functions.some((f) => f.type === 'rgbMatrix' && f.id !== props.fn.id && f.algorithm.type === 'script' && f.algorithm.script === script.value?.id),
)

function save(changes: Partial<RgbMatrix>) {
  send({ op: 'saveFunction', function: { ...props.fn, ...changes } })
}

// A cor `i` (a inicial, a final…); as cores a mais que o algoritmo não usa ficam como estão
const editingColor = ref<number>()
function setColor(i: number, color: string) {
  save({ colors: Array.from({ length: Math.max(colors.value, props.fn.colors.length) }, (_, k) => (k === i ? color : (props.fn.colors[k] ?? '#000000'))) })
}

// ---------- Texto e imagem ----------
const algorithm = computed(() => props.fn.algorithm)
function setAlgorithm(changes: Record<string, unknown>) {
  save({ algorithm: { ...props.fn.algorithm, ...changes } as RgbMatrix['algorithm'] })
}
const imageInput = ref<HTMLInputElement>()
const imageError = ref<string>()

/** Lê a imagem do arquivo pelo navegador, reduzida a no máximo 128 × 128, em RGBA e base64. */
async function chooseImage(e: Event) {
  const input = e.target as HTMLInputElement
  const file = input.files?.[0]
  input.value = ''
  if (!file) return
  try {
    const bitmap = await createImageBitmap(file)
    const scale = Math.min(1, 128 / Math.max(bitmap.width, bitmap.height))
    const [width, height] = [Math.max(1, Math.round(bitmap.width * scale)), Math.max(1, Math.round(bitmap.height * scale))]
    const canvas = document.createElement('canvas')
    canvas.width = width
    canvas.height = height
    const context = canvas.getContext('2d')!
    context.drawImage(bitmap, 0, 0, width, height)
    const bytes = context.getImageData(0, 0, width, height).data
    let binary = ''
    for (let i = 0; i < bytes.length; i += 0x8000) binary += String.fromCharCode(...bytes.subarray(i, i + 0x8000))
    setAlgorithm({ image: { name: file.name, width, height, pixels: btoa(binary) } })
    imageError.value = undefined
  } catch {
    imageError.value = t('matrix.imageError', { name: file.name })
  }
}

// ---------- Scripts ----------
const fileInput = ref<HTMLInputElement>()
// Um resultado por arquivo: o script importado ou o motivo da recusa
const imports = ref<{ file: string; error?: string }[]>([])

async function importScripts(e: Event) {
  const input = e.target as HTMLInputElement
  const results: typeof imports.value = []
  let last: RgbScript | undefined
  for (const file of input.files ?? []) {
    try {
      last = (await send({ op: 'importScript', fileName: file.name, source: await file.text() })) as RgbScript
    } catch (error) {
      // Recusado: os outros arquivos seguem, e o motivo vai para o relatório
      results.push({ file: file.name, error: (error as Error).message })
    }
  }
  input.value = ''
  // O último importado passa a ser o desta criação
  if (last) save({ algorithm: { type: 'script', script: last.id, properties: [] } })
  imports.value = results
}

async function removeScript() {
  const id = script.value!.id
  await save({ algorithm: { type: 'plain' } })
  await send({ op: 'removeScript', id })
}

/** O valor de uma propriedade do script: o da criação ou o padrão do script. */
function propertyValue(name: string, fallback: string) {
  const a = props.fn.algorithm
  return (a.type === 'script' ? a.properties.find((p) => p.name === name)?.value : undefined) ?? fallback
}

function setProperty(name: string, value: string) {
  const a = props.fn.algorithm
  if (a.type !== 'script') return
  save({ algorithm: { ...a, properties: [...a.properties.filter((p) => p.name !== name), { name, value }] } })
}
</script>

<template>
  <div class="flex flex-col gap-3">
    <StudioRgbMatrixPreview :fn="fn" />

    <div class="grid gap-3 @xl:grid-cols-2 @xl:max-w-xl">
      <label class="field-label">
        {{ t('matrix.group') }}
        <select class="field" :value="fn.group ?? ''" @change="save({ group: ($event.target as HTMLSelectElement).value || null })">
          <option value="">{{ t('io.none') }}</option>
          <option v-for="g in project.fixtureGroups" :key="g.id" :value="g.id">{{ g.name }} ({{ g.width }}×{{ g.height }})</option>
        </select>
      </label>
      <label class="field-label">
        {{ t('matrix.algorithm') }}
        <select class="field" :value="algorithmKey(fn.algorithm)" @change="save({ algorithm: algorithmOf(($event.target as HTMLSelectElement).value) })">
          <option value="plain">{{ t('matrixAlgorithm.plain') }}</option>
          <option value="text">{{ t('matrixAlgorithm.text') }}</option>
          <option value="image">{{ t('matrixAlgorithm.image') }}</option>
          <option v-for="s in project.scripts" :key="s.id" :value="`script:${s.id}`">{{ s.name }}</option>
        </select>
      </label>
    </div>
    <div class="flex flex-wrap gap-2">
      <RlButton size="sm" @press="fileInput?.click()">
        <Upload :size="16" />
        {{ t('matrix.importScript') }}
      </RlButton>
      <RlButton v-if="script && !sharedScript" size="sm" @press="removeScript">
        <Trash2 :size="16" />
        {{ t('matrix.removeScript') }}
      </RlButton>
      <input ref="fileInput" type="file" accept=".js" multiple class="hidden" @change="importScripts">
    </div>

    <label v-if="algorithm.type === 'text'" class="field-label @xl:max-w-xl">
      {{ t('matrix.text') }}
      <input class="field" :value="algorithm.text" maxlength="256" @change="setAlgorithm({ text: ($event.target as HTMLInputElement).value })">
    </label>
    <div v-if="algorithm.type === 'image'" class="flex flex-wrap items-center gap-2">
      <RlButton size="sm" @press="imageInput?.click()">
        <Upload :size="16" />
        {{ t('matrix.chooseImage') }}
      </RlButton>
      <span v-if="algorithm.image" class="text-sm text-fg-muted">{{ algorithm.image.name }} ({{ algorithm.image.width }}×{{ algorithm.image.height }})</span>
      <input ref="imageInput" type="file" accept="image/*" class="hidden" @change="chooseImage">
    </div>
    <p v-if="imageError" class="text-sm text-danger">⚠ {{ imageError }}</p>
    <div v-if="algorithm.type === 'text' || algorithm.type === 'image'" class="grid gap-3 @xl:max-w-xl @xl:grid-cols-[1fr_auto_auto]">
      <RlSegmented
        :model-value="algorithm.style"
        :options="(['static', 'horizontal', 'vertical'] as const).map((value) => ({ value, label: t(`matrixStyle.${algorithm.type}.${value}`) }))"
        @update:model-value="(style) => setAlgorithm({ style })"
      />
      <label class="field-label">
        {{ t('matrix.offsetX') }}
        <input class="field w-24" type="number" :value="algorithm.x" @change="setAlgorithm({ x: Number(($event.target as HTMLInputElement).value) || 0 })">
      </label>
      <label class="field-label">
        {{ t('matrix.offsetY') }}
        <input class="field w-24" type="number" :value="algorithm.y" @change="setAlgorithm({ y: Number(($event.target as HTMLInputElement).value) || 0 })">
      </label>
    </div>

    <div v-if="script?.properties.length" class="grid gap-3 @xl:grid-cols-2 @xl:max-w-xl">
      <label v-for="p in script.properties" :key="p.name" class="field-label">
        {{ p.display || p.name }}
        <select v-if="p.kind.type === 'list'" class="field" :value="propertyValue(p.name, p.default)" @change="setProperty(p.name, ($event.target as HTMLSelectElement).value)">
          <option v-for="v in p.kind.values" :key="v" :value="v">{{ v }}</option>
        </select>
        <input
          v-else-if="p.kind.type === 'range' || p.kind.type === 'float'"
          class="field"
          type="number"
          :min="p.kind.type === 'range' ? p.kind.min : undefined"
          :max="p.kind.type === 'range' ? p.kind.max : undefined"
          :step="p.kind.type === 'range' ? 1 : 'any'"
          :value="propertyValue(p.name, p.default)"
          @change="setProperty(p.name, ($event.target as HTMLInputElement).value)"
        >
        <input v-else class="field" :value="propertyValue(p.name, p.default)" @change="setProperty(p.name, ($event.target as HTMLInputElement).value)">
      </label>
    </div>

    <div class="grid content-start gap-2">
      <span class="field-label flex items-center gap-1">{{ t('matrix.control') }} <RlInfo :text="t('matrix.controlInfo')" /></span>
      <RlSegmented
        :model-value="fn.control"
        :options="(['rgb', 'dimmer', 'white', 'amber', 'uv'] as const).map((value) => ({ value, label: t(`matrixControl.${value}`) }))"
        @update:model-value="(control) => save({ control })"
      />
    </div>

    <div class="grid gap-3 @3xl:grid-cols-2">
      <div v-for="i in colors" :key="i" class="grid gap-1.5">
        <span class="field-label">{{ colors === 1 ? t('matrix.color') : i === 1 ? t('matrix.startColor') : t('matrix.endColor') }}</span>
        <RlSwatches
          :colors="palette"
          :custom="fn.control === 'rgb'"
          :model-value="fn.colors[i - 1] ?? '#000000'"
          @update:model-value="(c) => setColor(i - 1, c)"
          @custom="editingColor = i - 1"
        />
      </div>
    </div>

    <div class="grid gap-3 @3xl:grid-cols-2">
      <div class="grid content-start gap-2">
        <span class="field-label">{{ t('chaser.runOrder') }}</span>
        <RlSegmented
          :model-value="fn.runOrder"
          :options="(['loop', 'singleShot', 'pingPong'] as const).map((value) => ({ value, label: t(`runOrder.${value}`) }))"
          @update:model-value="(runOrder) => save({ runOrder })"
        />
        <span class="field-label">{{ t('chaser.direction') }}</span>
        <RlSegmented
          :model-value="fn.direction"
          :options="(['forward', 'backward'] as const).map((value) => ({ value, label: t(`direction.${value}`) }))"
          @update:model-value="(direction) => save({ direction })"
        />
      </div>
      <div class="grid content-start gap-2 @xl:grid-cols-3">
        <label class="field-label">
          {{ t('studio.fadeIn') }}
          <RlTimeField :model-value="fn.timing.fadeInMs" @update:model-value="(ms) => save({ timing: { ...fn.timing, fadeInMs: ms } })" />
        </label>
        <label class="field-label">
          {{ t('matrix.step') }}
          <RlTimeField :model-value="fn.timing.holdMs" @update:model-value="(ms) => save({ timing: { ...fn.timing, holdMs: ms } })" />
        </label>
        <label class="field-label">
          {{ t('studio.fadeOut') }}
          <RlTimeField :model-value="fn.timing.fadeOutMs" @update:model-value="(ms) => save({ timing: { ...fn.timing, fadeOutMs: ms } })" />
        </label>
      </div>
    </div>

    <RlDialog :open="editingColor !== undefined" :title="t('vc.color', { n: (editingColor ?? 0) + 1 })" @update:open="editingColor = undefined">
      <RlColorPicker v-if="editingColor !== undefined" :model-value="fn.colors[editingColor] ?? '#000000'" @update:model-value="(c: string) => setColor(editingColor!, c)" />
    </RlDialog>

    <RlDialog :open="imports.length > 0" :title="t('matrix.importReport')" @update:open="imports = []">
      <div class="flex flex-col gap-2">
        <p v-for="r in imports" :key="r.file" class="text-sm text-danger">⚠ {{ r.file }}: {{ r.error }}</p>
      </div>
    </RlDialog>
  </div>
</template>

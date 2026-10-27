<script setup lang="ts">
import { ArrowDown, ArrowUp, Grid3x3, Pencil, Plus, Trash2 } from 'lucide-vue-next'
import type { Efx } from '~/types/generated/Efx'
import type { EfxAlgorithm } from '~/types/generated/EfxAlgorithm'
import type { EfxFixture } from '~/types/generated/EfxFixture'
import type { EfxShape } from '~/types/generated/EfxShape'
import type { Fixture } from '~/types/generated/Fixture'
import type { Function } from '~/types/generated/Function'

// Editor de EFX: desenho (algoritmo, parâmetros, desenho à mão e velocidade) com prévia animada
// (StudioEfxPreview), fixtures com ordem, sentido, deslocamento, desenho próprio e espelho, propagação e tempos.
const props = defineProps<{ fn: Extract<Function, { type: 'efx' }> }>()
const { t } = useI18n()
const { state, send } = useEngine()
const project = computed(() => state.value.project)
const adding = ref(false)
const pickingGroup = ref(false)
const drawing = ref(false)

const algorithms: EfxAlgorithm[] = ['circle', 'eight', 'line', 'line2', 'diamond', 'square', 'squareChoppy', 'squareTrue', 'leaf', 'lissajous', 'freehand']
const shapeKeys = ['algorithm', 'width', 'height', 'xOffset', 'yOffset', 'rotation', 'xFrequency', 'yFrequency', 'xPhase', 'yPhase', 'path', 'speed', 'speedPoints'] as const
type Param = 'width' | 'height' | 'xOffset' | 'yOffset' | 'rotation' | 'xFrequency' | 'yFrequency' | 'xPhase' | 'yPhase'
const knobs: { key: Param; max: number; lissajous?: boolean }[] = [
  { key: 'width', max: 127 },
  { key: 'height', max: 127 },
  { key: 'xOffset', max: 255 },
  { key: 'yOffset', max: 255 },
  { key: 'rotation', max: 359 },
  { key: 'xFrequency', max: 32, lissajous: true },
  { key: 'yFrequency', max: 32, lissajous: true },
  { key: 'xPhase', max: 359, lissajous: true },
  { key: 'yPhase', max: 359, lissajous: true },
]

// As fixtures que andam no desenho: os grupos com as que estão neles agora (N1)
const moving = computed(() => efxFixtures(project.value, props.fn))
const fixtureOf = (id: string) => project.value.fixtures.find((f) => f.id === id)
const movers = (fixtures: Fixture[]) => fixtures.filter((f) => modeChannels(project.value, f).some((c) => c.attribute === 'pan' || c.attribute === 'tilt'))
const candidates = computed(() => movers(project.value.fixtures).filter((f) => !props.fn.fixtures.some((x) => 'fixture' in x && x.fixture === f.id)))
const groups = computed(() =>
  project.value.fixtureGroups.filter((g) => movers(groupFixtures(project.value, g.id)).length && !props.fn.fixtures.some((x) => 'group' in x && x.group === g.id)),
)
// Nome e cor de um item da lista: a fixture, ou o grupo com quantas fixtures dele andam
function itemLook(x: EfxFixture) {
  if ('fixture' in x) return { name: fixtureOf(x.fixture)?.name, color: fixtureOf(x.fixture)?.color }
  const name = project.value.fixtureGroups.find((g) => g.id === x.group)?.name
  return { name: `${name} (${movers(groupFixtures(project.value, x.group)).length})`, color: undefined }
}

function save(changes: Partial<Efx>) {
  send({ op: 'saveFunction', function: { ...props.fn, ...changes } })
}

function saveFixture(i: number, changes: Partial<EfxFixture>) {
  save({ fixtures: props.fn.fixtures.map((f, k) => (k === i ? { ...f, ...changes } : f)) })
}

function add(item: { fixture: string } | { group: string }) {
  save({ fixtures: [...props.fn.fixtures, { ...item, reverse: false, startOffset: 0, spread: 0, shape: null, invertPan: false, invertTilt: false, swapAxes: false }] })
}

function move(i: number, by: number) {
  const fixtures = [...props.fn.fixtures]
  fixtures.splice(i + by, 0, ...fixtures.splice(i, 1))
  save({ fixtures })
}

// O desenho que os knobs mexem: o geral, ou o do item escolhido (com desenho próprio)
const keyOf = (x: EfxFixture) => ('group' in x ? x.group : x.fixture)
const editing = ref<string | null>(null)
const editingIndex = computed(() => props.fn.fixtures.findIndex((x) => x.shape && keyOf(x) === editing.value))
const shape = computed<EfxShape>(() => props.fn.fixtures[editingIndex.value]?.shape ?? props.fn)
const ownItems = computed(() => props.fn.fixtures.filter((x) => x.shape))

function saveShape(changes: Partial<EfxShape>) {
  const own = props.fn.fixtures[editingIndex.value]?.shape
  if (own) saveFixture(editingIndex.value, { shape: { ...own, ...changes } })
  else save(changes)
}

// Ligar o desenho próprio copia o geral e o escolhe; desligar volta ao geral
function toggleOwn(i: number) {
  const x = props.fn.fixtures[i]!
  editing.value = x.shape ? null : keyOf(x)
  saveFixture(i, { shape: x.shape ? null : (Object.fromEntries(shapeKeys.map((k) => [k, props.fn[k]])) as EfxShape) })
}

// Velocidade: as curvas prontas ou a desenhada, que começa com a forma da curva de agora
const speedOptions = [...curves, 'drawn'] as const
function setSpeed(value: (typeof speedOptions)[number]) {
  if (value !== 'drawn') return saveShape({ speed: value, speedPoints: null })
  saveShape({ speedPoints: Array.from({ length: 16 }, (_, i) => Math.round(efxSpeed(shape.value, i / 15) * 255)) })
}

// O traço da prévia vira o desenho à mão, com até 64 pontos; o tamanho, o centro e a rotação voltam ao inteiro
// para o desenho ficar onde foi traçado
function draw(points: [number, number][]) {
  const kept = points.length > 64 ? Array.from({ length: 64 }, (_, k) => points[Math.floor((k * points.length) / 64)]!) : points
  const path = kept.map(([x, y]) => [Math.min(x - 127, 127), Math.min(y - 127, 127)] as [number, number])
  saveShape({ algorithm: 'freehand', path, width: 127, height: 127, xOffset: 127, yOffset: 127, rotation: 0 })
  drawing.value = false
}
</script>

<template>
  <div class="flex flex-col gap-3">
    <div class="grid gap-3 @3xl:grid-cols-[minmax(0,300px)_1fr]">
      <StudioEfxPreview :efx="fn" :drawing="drawing" @draw="draw" />

      <div class="flex flex-col gap-3">
        <!-- Com itens de desenho próprio: qual desenho os controles mostram -->
        <div v-if="ownItems.length" class="flex flex-wrap items-center gap-1.5">
          <span class="field-label">{{ t('efx.editing') }}</span>
          <RlButton size="sm" :model-value="editingIndex < 0" @press="editing = null">{{ t('efx.general') }}</RlButton>
          <RlButton v-for="x in ownItems" :key="keyOf(x)" size="sm" :model-value="editing === keyOf(x)" @press="editing = keyOf(x)">
            <Grid3x3 v-if="'group' in x" :size="14" />
            {{ itemLook(x).name }}
          </RlButton>
        </div>
        <div class="flex flex-wrap items-end gap-3">
          <label class="field-label w-full @xl:max-w-xs">
            {{ t('efx.algorithm') }}
            <select class="field" :value="shape.algorithm" @change="saveShape({ algorithm: ($event.target as HTMLSelectElement).value as EfxAlgorithm })">
              <option v-for="a in algorithms" :key="a" :value="a">{{ t(`efxAlgorithm.${a}`) }}</option>
            </select>
          </label>
          <span v-if="shape.algorithm === 'freehand'" class="flex items-center gap-3">
            <RlButton v-model="drawing" mode="toggle" highlight="led">
              <Pencil :size="16" />
              {{ t('efx.draw') }}
            </RlButton>
            <RlInfo :text="t('efx.drawInfo')" />
          </span>
        </div>
        <div class="flex flex-wrap gap-4">
          <div v-for="k in knobs.filter((k) => !k.lissajous || shape.algorithm === 'lissajous')" :key="k.key" class="flex w-20 flex-col items-center gap-1">
            <RlKnob :size="52" color="var(--color-position)" :model-value="shape[k.key] / k.max" @update:model-value="(v) => saveShape({ [k.key]: Math.round(v * k.max) })" />
            <span class="text-center text-xs text-fg-muted">{{ t(`efx.${k.key}`) }}</span>
            <RlLcd class="text-xs">{{ shape[k.key] }}</RlLcd>
          </div>
        </div>
        <div class="flex flex-wrap items-start gap-3">
          <label class="field-label w-full @xl:max-w-xs">
            <span class="flex items-center gap-2">{{ t('efx.speed') }} <RlInfo :text="t('efx.speedInfo')" /></span>
            <select class="field" :value="shape.speedPoints ? 'drawn' : shape.speed" @change="setSpeed(($event.target as HTMLSelectElement).value as (typeof speedOptions)[number])">
              <option v-for="s in speedOptions" :key="s" :value="s">{{ t(`efxSpeed.${s}`) }}</option>
            </select>
          </label>
          <StudioEfxCurve v-if="shape.speedPoints" :points="shape.speedPoints" @change="(speedPoints) => saveShape({ speedPoints })" />
        </div>
        <span class="flex items-center gap-3">
          <RlSwitch :model-value="fn.relative" @update:model-value="(relative) => save({ relative })">{{ t('efx.relative') }}</RlSwitch>
          <RlInfo :text="t('efx.relativeInfo')" />
        </span>
      </div>
    </div>

    <div class="grid gap-3 @3xl:grid-cols-2">
      <div class="grid content-start gap-2">
        <span class="field-label">{{ t('efx.propagation') }}</span>
        <RlSegmented
          :model-value="fn.propagation"
          :options="(['parallel', 'serial', 'asymmetric'] as const).map((value) => ({ value, label: t(`propagation.${value}`) }))"
          @update:model-value="(propagation) => save({ propagation })"
        />
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
          {{ t('efx.cycle') }}
          <RlTimeField :model-value="fn.timing.holdMs" @update:model-value="(ms) => save({ timing: { ...fn.timing, holdMs: ms } })" />
        </label>
        <label class="field-label">
          {{ t('studio.fadeOut') }}
          <RlTimeField :model-value="fn.timing.fadeOutMs" @update:model-value="(ms) => save({ timing: { ...fn.timing, fadeOutMs: ms } })" />
        </label>
      </div>
    </div>

    <div class="flex items-center justify-between gap-2">
      <h3 class="heading">{{ t('efx.fixtures', { n: moving.length }) }}</h3>
      <RlButton size="sm" @press="adding = true">
        <Plus :size="16" />
        {{ t('efx.addFixture') }}
      </RlButton>
    </div>
    <div v-for="(f, i) in fn.fixtures" :key="'group' in f ? f.group : f.fixture" class="sunken flex flex-wrap items-center gap-2 rounded-md p-2 pl-3">
      <span class="w-6 font-lcd text-lcd">{{ i + 1 }}</span>
      <Grid3x3 v-if="'group' in f" :size="14" class="text-fg-muted" />
      <span v-else class="rl-led" :style="{ '--led': itemLook(f).color }" />
      <span class="min-w-24 flex-1 truncate">{{ itemLook(f).name }}</span>
      <RlButton size="sm" mode="toggle" highlight="led" :model-value="!!f.shape" @press="toggleOwn(i)">{{ t('efx.ownShape') }}</RlButton>
      <RlButton size="sm" mode="toggle" highlight="led" :model-value="f.reverse" @press="saveFixture(i, { reverse: !f.reverse })">{{ t('efx.reverse') }}</RlButton>
      <RlButton size="sm" mode="toggle" highlight="led" :model-value="f.invertPan" @press="saveFixture(i, { invertPan: !f.invertPan })">{{ t('efx.invertPan') }}</RlButton>
      <RlButton size="sm" mode="toggle" highlight="led" :model-value="f.invertTilt" @press="saveFixture(i, { invertTilt: !f.invertTilt })">{{ t('efx.invertTilt') }}</RlButton>
      <RlButton size="sm" mode="toggle" highlight="led" :model-value="f.swapAxes" @press="saveFixture(i, { swapAxes: !f.swapAxes })">{{ t('efx.swapAxes') }}</RlButton>
      <label class="flex items-center gap-1 text-xs text-fg-muted">
        {{ t('efx.startOffset') }}
        <input type="number" min="0" max="359" class="field w-20" :value="f.startOffset" @change="saveFixture(i, { startOffset: Number(($event.target as HTMLInputElement).value) })">
      </label>
      <!-- No grupo, a defasagem: cada fixture começa esse tanto depois da anterior -->
      <label v-if="'group' in f" class="flex items-center gap-1 text-xs text-fg-muted">
        {{ t('efx.spread') }}
        <input type="number" min="0" max="359" class="field w-20" :value="f.spread" @change="saveFixture(i, { spread: Number(($event.target as HTMLInputElement).value) })">
      </label>
      <RlButton size="sm" :aria-label="t('chaser.up')" @press="i > 0 && move(i, -1)"><ArrowUp :size="14" /></RlButton>
      <RlButton size="sm" :aria-label="t('chaser.down')" @press="i < fn.fixtures.length - 1 && move(i, 1)"><ArrowDown :size="14" /></RlButton>
      <RlButton size="sm" :aria-label="t('common.remove')" @press="save({ fixtures: fn.fixtures.filter((_, k) => k !== i) })"><Trash2 :size="14" /></RlButton>
    </div>

    <RlDialog v-model:open="adding" :title="t('efx.addFixture')" :info="t('efx.addFixtureInfo')" sheet>
      <p v-if="!candidates.length && !groups.length" class="text-sm text-fg-muted">{{ t('efx.noCandidates') }}</p>
      <template v-else-if="groups.length">
        <div>
          <RlButton size="sm" highlight="led" :model-value="pickingGroup" @press="pickingGroup = !pickingGroup">{{ t('efx.pickGroup') }}</RlButton>
        </div>
        <div v-if="pickingGroup" class="flex flex-wrap gap-1.5">
          <RlButton
            v-for="g in groups"
            :key="g.id"
            size="sm"
            @press="add({ group: g.id })"
          >
            <Grid3x3 :size="14" />
            {{ g.name }}
          </RlButton>
        </div>
      </template>
      <div class="flex flex-wrap gap-1.5">
        <RlButton
          v-for="f in candidates"
          :key="f.id"
          size="sm"
          highlight="led"
          :color="f.color"
          @press="add({ fixture: f.id })"
        >
          {{ f.name }}
        </RlButton>
      </div>
    </RlDialog>
  </div>
</template>

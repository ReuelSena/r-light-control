<script setup lang="ts">
import { ArrowDown, ArrowUp, Grid3x3, Plus, Trash2 } from 'lucide-vue-next'
import type { Efx } from '~/types/generated/Efx'
import type { EfxAlgorithm } from '~/types/generated/EfxAlgorithm'
import type { EfxFixture } from '~/types/generated/EfxFixture'
import type { FixtureGroup } from '~/types/generated/FixtureGroup'
import type { Function } from '~/types/generated/Function'

// Editor de EFX: desenho (algoritmo e parâmetros) com prévia animada, fixtures com ordem, sentido
// e deslocamento, propagação e tempos. A prévia usa a mesma conta da saída (utils/functions).
const props = defineProps<{ fn: Extract<Function, { type: 'efx' }> }>()
const { t } = useI18n()
const { state, send } = useEngine()
const project = computed(() => state.value.project)
const now = useClock(33)
const adding = ref(false)
const pickingGroup = ref(false)

const algorithms: EfxAlgorithm[] = ['circle', 'eight', 'line', 'line2', 'diamond', 'square', 'squareChoppy', 'squareTrue', 'leaf', 'lissajous']
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

// Na prévia o tilt cresce para cima, como no palco visto de frente
const path = computed(() =>
  Array.from({ length: 129 }, (_, i) => efxPoint(props.fn, (i / 128) * Math.PI * 2)).map((p) => `${p.x},${255 - p.y}`).join(' '),
)
const dots = computed(() => efxPositions(props.fn, now.value))
const fixtureOf = (id: string) => project.value.fixtures.find((f) => f.id === id)
const candidates = computed(() =>
  project.value.fixtures.filter(
    (f) => modeChannels(project.value, f).some((c) => c.attribute === 'pan' || c.attribute === 'tilt') && !props.fn.fixtures.some((x) => x.fixture === f.id),
  ),
)
// Fixtures do grupo que podem entrar, na ordem da grade (linha por linha), que é a da propagação
const groupFixtures = (g: FixtureGroup) =>
  [...new Set([...g.cells].sort((a, b) => a.y - b.y || a.x - b.x).map((c) => c.fixture))].filter((id) => candidates.value.some((f) => f.id === id))
const groups = computed(() => project.value.fixtureGroups.filter((g) => groupFixtures(g).length))

function save(changes: Partial<Efx>) {
  send({ op: 'saveFunction', function: { ...props.fn, ...changes } })
}

function saveFixture(i: number, changes: Partial<EfxFixture>) {
  save({ fixtures: props.fn.fixtures.map((f, k) => (k === i ? { ...f, ...changes } : f)) })
}

function addFixtures(ids: string[]) {
  save({ fixtures: [...props.fn.fixtures, ...ids.map((fixture) => ({ fixture, reverse: false, startOffset: 0 }))] })
}

function move(i: number, by: number) {
  const fixtures = [...props.fn.fixtures]
  fixtures.splice(i + by, 0, ...fixtures.splice(i, 1))
  save({ fixtures })
}
</script>

<template>
  <div class="flex flex-col gap-3">
    <div class="grid gap-3 @3xl:grid-cols-[minmax(0,300px)_1fr]">
      <svg viewBox="-10 -10 275 275" class="sunken aspect-square w-full max-w-xs rounded-md">
        <path d="M127.5 -10V265M-10 127.5H265" stroke="var(--color-line)" stroke-dasharray="4 4" />
        <polyline :points="path" fill="none" stroke="var(--color-position)" stroke-width="2" opacity="0.7" />
        <circle
          v-for="(p, i) in dots"
          :key="i"
          :cx="p.x"
          :cy="255 - p.y"
          r="7"
          :fill="fixtureOf(fn.fixtures[i]!.fixture)?.color ?? '#fff'"
          stroke="#000"
          stroke-width="2"
        />
      </svg>

      <div class="flex flex-col gap-3">
        <label class="field-label @xl:max-w-xs">
          {{ t('efx.algorithm') }}
          <select class="field" :value="fn.algorithm" @change="save({ algorithm: ($event.target as HTMLSelectElement).value as EfxAlgorithm })">
            <option v-for="a in algorithms" :key="a" :value="a">{{ t(`efxAlgorithm.${a}`) }}</option>
          </select>
        </label>
        <div class="flex flex-wrap gap-4">
          <div v-for="k in knobs.filter((k) => !k.lissajous || fn.algorithm === 'lissajous')" :key="k.key" class="flex w-20 flex-col items-center gap-1">
            <RlKnob :size="52" color="var(--color-position)" :model-value="fn[k.key] / k.max" @update:model-value="(v) => save({ [k.key]: Math.round(v * k.max) })" />
            <span class="text-center text-xs text-fg-muted">{{ t(`efx.${k.key}`) }}</span>
            <RlLcd class="text-xs">{{ fn[k.key] }}</RlLcd>
          </div>
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
      <h3 class="heading">{{ t('efx.fixtures', { n: fn.fixtures.length }) }}</h3>
      <RlButton size="sm" @press="adding = true">
        <Plus :size="16" />
        {{ t('efx.addFixture') }}
      </RlButton>
    </div>
    <div v-for="(f, i) in fn.fixtures" :key="f.fixture" class="sunken flex flex-wrap items-center gap-2 rounded-md p-2 pl-3">
      <span class="w-6 font-lcd text-lcd">{{ i + 1 }}</span>
      <span class="rl-led" :style="{ '--led': fixtureOf(f.fixture)?.color }" />
      <span class="min-w-24 flex-1 truncate">{{ fixtureOf(f.fixture)?.name }}</span>
      <RlButton size="sm" mode="toggle" highlight="led" :model-value="f.reverse" @press="saveFixture(i, { reverse: !f.reverse })">{{ t('efx.reverse') }}</RlButton>
      <label class="flex items-center gap-1 text-xs text-fg-muted">
        {{ t('efx.startOffset') }}
        <input type="number" min="0" max="359" class="field w-20" :value="f.startOffset" @change="saveFixture(i, { startOffset: Number(($event.target as HTMLInputElement).value) })">
      </label>
      <RlButton size="sm" :aria-label="t('chaser.up')" @press="i > 0 && move(i, -1)"><ArrowUp :size="14" /></RlButton>
      <RlButton size="sm" :aria-label="t('chaser.down')" @press="i < fn.fixtures.length - 1 && move(i, 1)"><ArrowDown :size="14" /></RlButton>
      <RlButton size="sm" :aria-label="t('common.remove')" @press="save({ fixtures: fn.fixtures.filter((_, k) => k !== i) })"><Trash2 :size="14" /></RlButton>
    </div>

    <RlDialog v-model:open="adding" :title="t('efx.addFixture')" :info="t('efx.addFixtureInfo')" sheet>
      <p v-if="!candidates.length" class="text-sm text-fg-muted">{{ t('efx.noCandidates') }}</p>
      <template v-else-if="groups.length">
        <div>
          <RlButton size="sm" highlight="led" :model-value="pickingGroup" @press="pickingGroup = !pickingGroup">{{ t('efx.pickGroup') }}</RlButton>
        </div>
        <div v-if="pickingGroup" class="flex flex-wrap gap-1.5">
          <RlButton
            v-for="g in groups"
            :key="g.id"
            size="sm"
            @press="addFixtures(groupFixtures(g))"
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
          @press="addFixtures([f.id])"
        >
          {{ f.name }}
        </RlButton>
      </div>
    </RlDialog>
  </div>
</template>

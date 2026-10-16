<script setup lang="ts">
import type { Function } from '~/types/generated/Function'
import type { RgbMatrix } from '~/types/generated/RgbMatrix'

// Editor de RGB Matrix: grupo de fixtures (grade 2D), algoritmo, cores e tempos, com prévia animada.
// Os scripts do QLC+ entram na F9; aqui há alguns algoritmos de amostra (utils/functions).
const props = defineProps<{ fn: Extract<Function, { type: 'rgbMatrix' }> }>()
const { t } = useI18n()
const { state, send } = useEngine()
const project = computed(() => state.value.project)
const now = useClock(50)
const palette = [...tagColors, '#000000']

const group = computed(() => project.value.fixtureGroups.find((g) => g.id === props.fn.group))
const step = computed(() => Math.floor(now.value / Math.max(props.fn.timing.holdMs, 50)))
const colorAt = (x: number, y: number) =>
  group.value?.cells.some((c) => c.x === x && c.y === y) ? `rgb(${matrixColor(props.fn, x, y, group.value.width, step.value).join(',')})` : undefined

function save(changes: Partial<RgbMatrix>) {
  send({ op: 'saveFunction', function: { ...props.fn, ...changes } })
}
</script>

<template>
  <div class="flex flex-col gap-3">
    <div
      v-if="group"
      class="sunken grid max-w-xl gap-1 rounded-md p-2"
      :style="{ gridTemplateColumns: `repeat(${group.width}, minmax(0, 1fr))` }"
    >
      <template v-for="y in group.height" :key="y">
        <span v-for="x in group.width" :key="x" class="pixel" :style="{ '--px': colorAt(x - 1, y - 1) }" :class="{ empty: !colorAt(x - 1, y - 1) }" />
      </template>
    </div>

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
        <select class="field" :value="fn.algorithm" @change="save({ algorithm: ($event.target as HTMLSelectElement).value })">
          <option v-for="a in matrixAlgorithms" :key="a" :value="a">{{ t(`matrixAlgorithm.${a}`) }}</option>
        </select>
      </label>
    </div>

    <div class="grid gap-3 @3xl:grid-cols-2">
      <div v-for="(label, i) in [t('matrix.startColor'), t('matrix.endColor')]" :key="i" class="grid gap-1.5">
        <span class="field-label">{{ label }}</span>
        <RlSwatches
          :colors="palette"
          :model-value="fn.colors[i] ?? '#000000'"
          @update:model-value="(c) => save({ colors: [0, 1].map((k) => (k === i ? c : fn.colors[k] ?? '#000000')) })"
        />
      </div>
    </div>

    <div class="grid gap-3 @3xl:grid-cols-2">
      <div class="grid content-start gap-2">
        <span class="field-label">{{ t('chaser.runOrder') }}</span>
        <RlSegmented
          :model-value="fn.runOrder"
          :options="(['loop', 'singleShot', 'pingPong', 'random'] as const).map((value) => ({ value, label: t(`runOrder.${value}`) }))"
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
  </div>
</template>

<style scoped>
.pixel {
  aspect-ratio: 1;
  border-radius: 3px;
  background: var(--px);
  box-shadow: 0 0 10px var(--px), inset 0 1px 1px rgb(255 255 255 / 0.3);
}

.pixel.empty {
  background: #111;
  box-shadow: inset 0 1px 3px rgb(0 0 0 / 0.8);
}
</style>

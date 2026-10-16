<script setup lang="ts">
import { ArrowDown, ArrowUp, Plus, Trash2 } from 'lucide-vue-next'
import type { Chaser } from '~/types/generated/Chaser'
import type { ChaserStep } from '~/types/generated/ChaserStep'
import type { Function } from '~/types/generated/Function'

type ChaserFn = Extract<Function, { type: 'chaser' | 'sequence' }>

// Editor de chaser e de sequência: passos com fade in, hold e fade out, ordem, sentido e de onde vem
// cada tempo. Na sequência, cada passo é um estado da cena escolhida, editado nos faders abaixo.
const props = defineProps<{ fn: ChaserFn }>()
const { t } = useI18n()
const { state, send } = useEngine()
const project = computed(() => state.value.project)
const now = useClock(100)
const selectedStep = ref(0)
const adding = ref(false)

const isSequence = computed(() => props.fn.type === 'sequence')
const current = computed(() => (state.value.running.includes(props.fn.id) ? chaserStep(props.fn, now.value) : -1))
const functionOf = (id: string) => project.value.functions.find((f) => f.id === id)
const fixtureOf = (id: string) => project.value.fixtures.find((f) => f.id === id)
const step = computed(() => props.fn.steps[selectedStep.value])
const times = [
  { mode: 'fadeInMode', key: 'fadeInMs', label: 'studio.fadeIn' },
  { mode: 'holdMode', key: 'holdMs', label: 'studio.hold' },
  { mode: 'fadeOutMode', key: 'fadeOutMs', label: 'studio.fadeOut' },
] as const

function save(changes: Partial<Chaser>) {
  send({ op: 'saveFunction', function: { ...props.fn, ...changes } })
}

function saveStep(i: number, changes: Partial<ChaserStep>) {
  save({ steps: props.fn.steps.map((s, k) => (k === i ? { ...s, ...changes } : s)) })
}

function move(i: number, by: number) {
  const steps = [...props.fn.steps]
  steps.splice(i + by, 0, ...steps.splice(i, 1))
  save({ steps })
  selectedStep.value = i + by
}

// No chaser, o passo dispara uma função; na sequência, copia os valores do passo escolhido (ou da cena)
function addStep(functionId: string) {
  const base = step.value?.values ?? (functionOf(functionId) as Extract<Function, { type: 'scene' }> | undefined)?.values ?? []
  const values = isSequence.value ? base.map((v) => ({ ...v })) : []
  save({ steps: [...props.fn.steps, { function: functionId, timing: { ...props.fn.common }, note: '', values }] })
  selectedStep.value = props.fn.steps.length
  adding.value = false
}
</script>

<template>
  <div class="flex flex-col gap-3">
    <div v-if="isSequence" class="field-label @xl:max-w-sm">
      <span class="flex items-center gap-2">
        {{ t('chaser.scene') }}
        <RlInfo v-if="fn.steps.length" :text="t('chaser.sceneLocked')" />
      </span>
      <select
        class="field"
        :aria-label="t('chaser.scene')"
        :value="fn.scene ?? ''"
        :disabled="fn.steps.length > 0"
        @change="save({ scene: ($event.target as HTMLSelectElement).value })"
      >
        <option v-for="s in project.functions.filter((f) => f.type === 'scene')" :key="s.id" :value="s.id">{{ s.name }}</option>
      </select>
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
        <RlSwitch :model-value="fn.beats" @update:model-value="(beats) => save({ beats })">{{ t('chaser.beats') }}</RlSwitch>
      </div>

      <div class="grid content-start gap-2">
        <span class="field-label">{{ t('chaser.timingSource') }}</span>
        <div v-for="time in times" :key="time.key" class="grid grid-cols-[72px_1fr] items-center gap-2 @xl:grid-cols-[72px_1fr_96px]">
          <span class="text-sm">{{ t(time.label) }}</span>
          <RlSegmented
            :model-value="fn[time.mode]"
            :options="(['default', 'common', 'perStep'] as const).map((value) => ({ value, label: t(`speedMode.${value}`) }))"
            @update:model-value="(mode) => save({ [time.mode]: mode })"
          />
          <RlTimeField
            v-if="fn[time.mode] === 'common'"
            class="col-start-2 @xl:col-start-auto"
            :beats="fn.beats"
            :model-value="fn.common[time.key]"
            @update:model-value="(ms) => save({ common: { ...fn.common, [time.key]: ms } })"
          />
        </div>
      </div>
    </div>

    <div class="flex items-center justify-between gap-2">
      <h3 class="heading">{{ t('chaser.steps', { n: fn.steps.length }) }}</h3>
      <RlButton size="sm" @press="isSequence ? fn.scene && addStep(fn.scene) : (adding = true)">
        <Plus :size="16" />
        {{ t('chaser.addStep') }}
      </RlButton>
    </div>
    <div class="overflow-x-auto">
      <table class="rl-table min-w-[560px]">
        <thead>
          <tr>
            <th class="w-10">#</th>
            <th>{{ isSequence ? t('chaser.note') : t('chaser.function') }}</th>
            <th v-for="time in times" :key="time.key" class="w-24">{{ t(time.label) }}</th>
            <th class="w-32" />
          </tr>
        </thead>
        <tbody>
          <tr v-for="(s, i) in fn.steps" :key="i" class="step" :class="{ active: i === selectedStep }" @click="selectedStep = i">
            <td>
              <span class="flex items-center gap-1.5 font-lcd">
                <span class="rl-led" :style="{ '--led': i === current ? 'var(--color-on)' : 'var(--color-fg-dim)', opacity: i === current ? 1 : 0.3 }" />
                {{ i + 1 }}
              </span>
            </td>
            <td>
              <input v-if="isSequence" class="field w-full" :value="s.note" @change="saveStep(i, { note: ($event.target as HTMLInputElement).value })">
              <span v-else class="flex items-center gap-2">
                <component :is="functionIcons[functionOf(s.function)?.type ?? 'scene']" :size="16" class="shrink-0 text-fg-muted" />
                <span class="truncate">{{ functionOf(s.function)?.name ?? '?' }}</span>
              </span>
            </td>
            <td v-for="time in times" :key="time.key">
              <RlTimeField
                v-if="fn[time.mode] === 'perStep'"
                :beats="fn.beats"
                :model-value="s.timing[time.key]"
                @update:model-value="(ms) => saveStep(i, { timing: { ...s.timing, [time.key]: ms } })"
              />
              <span v-else class="text-xs text-fg-dim">{{ fn[time.mode] === 'common' ? `${fn.common[time.key] / 1000} ${fn.beats ? 'bt' : 's'}` : t('speedMode.default') }}</span>
            </td>
            <td>
              <span class="flex justify-end gap-1">
                <RlButton size="sm" :aria-label="t('chaser.up')" @press="i > 0 && move(i, -1)"><ArrowUp :size="14" /></RlButton>
                <RlButton size="sm" :aria-label="t('chaser.down')" @press="i < fn.steps.length - 1 && move(i, 1)"><ArrowDown :size="14" /></RlButton>
                <RlButton size="sm" :aria-label="t('common.remove')" @press="save({ steps: fn.steps.filter((_, k) => k !== i) })"><Trash2 :size="14" /></RlButton>
              </span>
            </td>
          </tr>
        </tbody>
      </table>
    </div>
    <p v-if="!fn.steps.length" class="text-sm text-fg-muted">{{ t('chaser.noSteps') }}</p>

    <template v-if="isSequence && step">
      <h3 class="heading">{{ t('chaser.stepValues', { n: selectedStep + 1 }) }}</h3>
      <div class="flex h-80 overflow-x-auto rounded-md border border-black">
        <RlChannelStrip
          v-for="(v, k) in step.values"
          :key="`${v.fixture}:${v.channel}`"
          plain
          :label="`${fixtureOf(v.fixture)?.name} · ${modeChannels(project, fixtureOf(v.fixture)!)[v.channel]?.name}`"
          :color="fixtureOf(v.fixture)?.color"
          :model-value="v.value / 255"
          @update:model-value="(x) => saveStep(selectedStep, { values: step!.values.map((o, j) => (j === k ? { ...o, value: Math.round(x * 255) } : o)) })"
        />
      </div>
    </template>

    <RlDialog v-model:open="adding" :title="t('chaser.addStep')" sheet>
      <div class="grid gap-1.5">
        <RlButton v-for="f in project.functions.filter((x) => x.id !== fn.id)" :key="f.id" class="justify-start!" @press="addStep(f.id)">
          <component :is="functionIcons[f.type]" :size="18" />
          {{ f.name }}
        </RlButton>
      </div>
    </RlDialog>
  </div>
</template>

<style scoped>
.step {
  cursor: pointer;
}

.step.active {
  background: color-mix(in srgb, var(--color-select) 15%, transparent);
  box-shadow: inset 3px 0 0 var(--color-select);
}
</style>

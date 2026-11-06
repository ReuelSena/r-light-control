<script setup lang="ts">
import { Plus, Trash2 } from 'lucide-vue-next'
import type { Function } from '~/types/generated/Function'
import type { Phaser } from '~/types/generated/Phaser'
import type { PhaserLayer } from '~/types/generated/PhaserLayer'

// Editor do Phaser: o grupo de fixtures, as camadas (um atributo e uma onda cada), a defasagem entre as
// fixtures e o ciclo. A prévia animada é `StudioPhaserPreview`, à parte: o relógio dela redesenharia o
// editor e repunha nos campos o valor salvo enquanto se digita.
const props = defineProps<{ fn: Extract<Function, { type: 'phaser' }> }>()
const { t } = useI18n()
const { state, send } = useEngine()
const project = computed(() => state.value.project)

function save(changes: Partial<Phaser>) {
  send({ op: 'saveFunction', function: { ...props.fn, ...changes } })
}

function saveLayer(i: number, changes: Partial<PhaserLayer>) {
  save({ layers: props.fn.layers.map((l, k) => (k === i ? { ...l, ...changes } : l)) })
}

function addLayer() {
  const free = phaserAttributes.find((a) => !props.fn.layers.some((l) => l.attribute === a && !l.relative)) ?? 'dimmer'
  save({ layers: [...props.fn.layers, { attribute: free, shape: 'sine', width: 25, low: 0, high: 255, phase: 0, relative: false }] })
}

// Piso e teto em %: do curso inteiro do canal; no relativo, o deslocamento em volta do que está embaixo
// (o zero em 127, como no EFX), de -50% a +50%
const percent = (l: PhaserLayer, v: number) => Math.round(((l.relative ? v - 127 : v) * 100) / 255)
const fromPercent = (l: PhaserLayer, p: number) => Math.min(255, Math.max(0, Math.round((p * 255) / 100) + (l.relative ? 127 : 0)))
const number = (e: Event) => Number((e.target as HTMLInputElement).value) || 0

// O piso nunca passa do teto, e o teto nunca fica abaixo do piso
function setRange(i: number, key: 'low' | 'high', e: Event) {
  const l = props.fn.layers[i]!
  const v = fromPercent(l, number(e))
  saveLayer(i, key === 'low' ? { low: v, high: Math.max(l.high, v) } : { high: v, low: Math.min(l.low, v) })
}

// Absoluta, a onda vai de 0 a 100%; relativa, balança em volta do que está embaixo
const toggleRelative = (i: number, relative: boolean) => saveLayer(i, relative ? { relative, low: 107, high: 147 } : { relative, low: 0, high: 255 })

const shapeLabel = (shape: (typeof waveShapes)[number]) => t(`waveShape.${shape}`)
</script>

<template>
  <div class="flex flex-col gap-3">
    <StudioPhaserPreview :fn="fn" />

    <div class="grid gap-3 @xl:max-w-xl @xl:grid-cols-2">
      <label class="field-label">
        {{ t('matrix.group') }}
        <select class="field" :value="fn.group ?? ''" @change="save({ group: ($event.target as HTMLSelectElement).value || null })">
          <option value="">{{ t('io.none') }}</option>
          <option v-for="g in project.fixtureGroups" :key="g.id" :value="g.id">{{ g.name }} ({{ g.width }}×{{ g.height }})</option>
        </select>
      </label>
      <label class="field-label">
        <span class="flex items-center gap-1">{{ t('phaser.spread') }} <RlInfo :text="t('phaser.spreadInfo')" /></span>
        <input class="field" type="number" min="-360" max="360" step="5" :value="fn.spread" @change="save({ spread: Math.min(360, Math.max(-360, Math.round(number($event)))) })">
      </label>
    </div>

    <div class="flex flex-wrap items-center gap-2">
      <span class="flex items-center gap-1 text-sm font-bold">{{ t('phaser.layers') }} <RlInfo :text="t('phaser.layersInfo')" /></span>
      <RlButton v-if="fn.layers.length < 8" size="sm" @press="addLayer">
        <Plus :size="16" />
        {{ t('phaser.addLayer') }}
      </RlButton>
    </div>
    <div class="grid gap-3 @3xl:grid-cols-2">
      <div v-for="(layer, i) in fn.layers" :key="i" class="sunken flex flex-col gap-2 rounded-md p-2">
        <div class="flex flex-wrap items-end gap-2">
          <label class="field-label min-w-40 flex-1">
            {{ t('phaser.attribute') }}
            <select class="field" :value="layer.attribute" @change="saveLayer(i, { attribute: ($event.target as HTMLSelectElement).value as PhaserLayer['attribute'] })">
              <option
                v-for="a in phaserAttributes"
                :key="a"
                :value="a"
                :disabled="fn.layers.some((o, k) => k !== i && o.attribute === a && o.relative === layer.relative)"
              >
                {{ t(`attribute.${a}`) }}
              </option>
            </select>
          </label>
          <span class="flex items-center gap-1">
            <RlSwitch :model-value="layer.relative" @update:model-value="(relative) => toggleRelative(i, relative)">{{ t('phaser.relative') }}</RlSwitch>
            <RlInfo :text="t('phaser.relativeInfo')" />
          </span>
          <RlButton v-if="fn.layers.length > 1" :aria-label="t('common.remove')" :title="t('common.remove')" @press="save({ layers: fn.layers.filter((_, k) => k !== i) })">
            <Trash2 :size="16" />
          </RlButton>
        </div>
        <RlSegmented
          :model-value="layer.shape"
          :options="waveShapes.map((value) => ({ value, label: shapeLabel(value) }))"
          @update:model-value="(shape) => saveLayer(i, { shape })"
        />
        <div class="grid grid-cols-2 gap-2 @xl:grid-cols-4">
          <label class="field-label">
            {{ layer.relative ? t('phaser.lowRelative') : t('phaser.low') }}
            <input class="field" type="number" :min="layer.relative ? -50 : 0" :max="layer.relative ? 50 : 100" :value="percent(layer, layer.low)" @change="setRange(i, 'low', $event)">
          </label>
          <label class="field-label">
            {{ layer.relative ? t('phaser.highRelative') : t('phaser.high') }}
            <input class="field" type="number" :min="layer.relative ? -50 : 0" :max="layer.relative ? 50 : 100" :value="percent(layer, layer.high)" @change="setRange(i, 'high', $event)">
          </label>
          <label v-if="layer.shape === 'pulse'" class="field-label">
            {{ t('phaser.width') }}
            <input class="field" type="number" min="1" max="99" :value="layer.width" @change="saveLayer(i, { width: Math.min(99, Math.max(1, Math.round(number($event)))) })">
          </label>
          <label class="field-label">
            {{ t('phaser.phase') }}
            <input class="field" type="number" min="0" max="359" step="5" :value="layer.phase" @change="saveLayer(i, { phase: (((Math.round(number($event)) % 360) + 360) % 360) })">
          </label>
        </div>
      </div>
    </div>

    <div class="grid content-start gap-2 @xl:max-w-xl @xl:grid-cols-3">
      <label class="field-label">
        {{ t('studio.fadeIn') }}
        <RlTimeField :model-value="fn.timing.fadeInMs" @update:model-value="(ms) => save({ timing: { ...fn.timing, fadeInMs: ms } })" />
      </label>
      <label class="field-label">
        {{ t('phaser.cycle') }}
        <RlTimeField :beats="fn.beats" :model-value="fn.timing.holdMs" @update:model-value="(ms) => save({ timing: { ...fn.timing, holdMs: ms } })" />
      </label>
      <label class="field-label">
        {{ t('studio.fadeOut') }}
        <RlTimeField :model-value="fn.timing.fadeOutMs" @update:model-value="(ms) => save({ timing: { ...fn.timing, fadeOutMs: ms } })" />
      </label>
    </div>
    <RlSwitch :model-value="fn.beats" @update:model-value="(beats) => save({ beats })">{{ t('phaser.beats') }}</RlSwitch>
  </div>
</template>

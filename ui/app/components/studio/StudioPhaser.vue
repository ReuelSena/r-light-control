<script setup lang="ts">
import { Plus, Trash2 } from 'lucide-vue-next'
import type { Function } from '~/types/generated/Function'
import type { Phaser } from '~/types/generated/Phaser'
import type { PhaserLayer } from '~/types/generated/PhaserLayer'

// Editor do Phaser: as camadas, cada uma com o seu grupo (de fixtures, com um atributo, ou de canais, com
// todos os canais dele), a sua defasagem e a sua onda, e o ciclo. A prévia animada é `StudioPhaserPreview`,
// à parte: o relógio dela redesenharia o editor e repunha nos campos o valor salvo enquanto se digita.
const props = defineProps<{ fn: Extract<Function, { type: 'phaser' }> }>()
const { t } = useI18n()
const { state, send } = useEngine()
const project = computed(() => state.value.project)

const MAX_LAYERS = 16

function save(changes: Partial<Phaser>) {
  send({ op: 'saveFunction', function: { ...props.fn, ...changes } })
}

function saveLayer(i: number, changes: Partial<PhaserLayer>) {
  save({ layers: props.fn.layers.map((l, k) => (k === i ? { ...l, ...changes } : l)) })
}

// A camada nova nasce sem grupo: a lista de grupos aparece para escolher
const addLayer = () => save({ layers: [...props.fn.layers, { group: null, channelGroup: null, attribute: 'dimmer', spread: 0, shape: 'sine', width: 25, low: 0, high: 255, phase: 0, relative: false }] })

const hasSource = (l: PhaserLayer) => !!(l.group || l.channelGroup)
const sourceName = (l: PhaserLayer) => (l.group ? project.value.fixtureGroups.find((g) => g.id === l.group)?.name : project.value.channelGroups.find((g) => g.id === l.channelGroup)?.name) ?? ''

// Ao escolher um grupo de fixtures, o atributo é o primeiro livre naquele grupo (uma camada por grupo,
// atributo e tipo)
function pickFixtureGroup(i: number, group: string) {
  const layer = props.fn.layers[i]!
  const taken = (a: string) => props.fn.layers.some((o, k) => k !== i && o.group === group && o.attribute === a && o.relative === layer.relative)
  saveLayer(i, { group, channelGroup: null, attribute: phaserAttributes.find((a) => !taken(a)) ?? layer.attribute })
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

    <div class="flex flex-wrap items-center gap-2">
      <span class="flex items-center gap-1 text-sm font-bold">{{ t('phaser.layers') }} <RlInfo :text="t('phaser.layersInfo')" /></span>
      <RlButton v-if="fn.layers.length < MAX_LAYERS" size="sm" @press="addLayer">
        <Plus :size="16" />
        {{ t('phaser.addLayer') }}
      </RlButton>
    </div>
    <div class="grid gap-3 @3xl:grid-cols-2">
      <div v-for="(layer, i) in fn.layers" :key="i" class="sunken flex flex-col gap-2 rounded-md p-2">
        <!-- Sem grupo: a lista dos grupos de fixtures e de canais para escolher -->
        <template v-if="!hasSource(layer)">
          <div class="flex items-center justify-between gap-2">
            <span class="text-sm font-bold">{{ t('phaser.pickGroup') }}</span>
            <RlButton v-if="fn.layers.length > 1" :aria-label="t('common.remove')" :title="t('common.remove')" @press="save({ layers: fn.layers.filter((_, k) => k !== i) })">
              <Trash2 :size="16" />
            </RlButton>
          </div>
          <p v-if="!project.fixtureGroups.length && !project.channelGroups.length" class="text-sm text-fg-muted">{{ t('phaser.noGroups') }}</p>
          <div v-if="project.fixtureGroups.length" class="flex flex-col gap-1">
            <span class="text-xs text-fg-muted">{{ t('phaser.fixtureGroups') }}</span>
            <div class="flex flex-wrap gap-1">
              <RlButton v-for="{ group: g, depth } in groupTree(project)" :key="g.id" size="sm" @press="pickFixtureGroup(i, g.id)">
                {{ '— '.repeat(depth) }}{{ g.name }} ({{ g.width }}×{{ g.height }})
              </RlButton>
            </div>
          </div>
          <div v-if="project.channelGroups.length" class="flex flex-col gap-1">
            <span class="text-xs text-fg-muted">{{ t('phaser.channelGroups') }}</span>
            <div class="flex flex-wrap gap-1">
              <RlButton v-for="g in project.channelGroups" :key="g.id" size="sm" @press="saveLayer(i, { group: null, channelGroup: g.id })">
                {{ g.name }} ({{ g.channels.length }})
              </RlButton>
            </div>
          </div>
        </template>

        <template v-else>
          <div class="flex flex-wrap items-end gap-2">
            <!-- O atributo só existe para o grupo de fixtures; o grupo de canais leva todos os canais dele -->
            <label v-if="layer.group" class="field-label min-w-40 flex-1">
              {{ sourceName(layer) }}
              <select class="field" :value="layer.attribute" @change="saveLayer(i, { attribute: ($event.target as HTMLSelectElement).value as PhaserLayer['attribute'] })">
                <option
                  v-for="a in phaserAttributes"
                  :key="a"
                  :value="a"
                  :disabled="fn.layers.some((o, k) => k !== i && o.group === layer.group && o.attribute === a && o.relative === layer.relative)"
                >
                  {{ t(`attribute.${a}`) }}
                </option>
              </select>
            </label>
            <span v-else class="min-w-40 flex-1 truncate text-sm font-bold">{{ sourceName(layer) }}</span>
            <RlButton size="sm" @press="saveLayer(i, { group: null, channelGroup: null })">{{ t('phaser.changeGroup') }}</RlButton>
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
          <div class="grid grid-cols-2 items-end gap-2 @xl:grid-cols-5">
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
            <label class="field-label">
              <span class="flex items-center gap-1">{{ t('phaser.spread') }} <RlInfo :text="t('phaser.spreadInfo')" /></span>
              <input class="field" type="number" min="-360" max="360" step="5" :value="layer.spread" @change="saveLayer(i, { spread: Math.min(360, Math.max(-360, Math.round(number($event)))) })">
            </label>
          </div>
        </template>
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

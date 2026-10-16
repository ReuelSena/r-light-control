<script setup lang="ts">
import { ArrowDown, ArrowUp, Plus, Trash2, TriangleAlert } from 'lucide-vue-next'
import type { Attribute } from '~/types/generated/Attribute'
import type { FixtureDef } from '~/types/generated/FixtureDef'
import type { FixtureKind } from '~/types/generated/FixtureKind'
import type { Physical } from '~/types/generated/Physical'

// Editor de fixture (protótipo; a F13 implementa): dados gerais, canais com faixas de valores,
// modos e dados físicos. Edita uma cópia e só entrega ao salvar.
const props = defineProps<{ def: FixtureDef }>()
const emit = defineEmits<{ save: [def: FixtureDef]; cancel: [] }>()
const { t } = useI18n()
const draft = ref<FixtureDef>(JSON.parse(JSON.stringify(props.def)))
const tab = ref('general')

const kinds: FixtureKind[] = ['colorChanger', 'dimmer', 'movingHead', 'scanner', 'ledBar', 'strobe', 'hazer', 'smoke', 'laser', 'other']
const attributes: Attribute[] = [
  'dimmer', 'red', 'green', 'blue', 'white', 'amber', 'uv', 'cyan', 'magenta', 'yellow', 'colorWheel', 'colorMacro',
  'pan', 'tilt', 'speed', 'shutter', 'gobo', 'goboRotation', 'prism', 'focus', 'zoom', 'iris', 'effect', 'maintenance', 'other',
]
const physicalFields: [keyof Physical, string][] = [
  ['widthMm', 'mm'], ['heightMm', 'mm'], ['depthMm', 'mm'], ['weightKg', 'kg'], ['powerW', 'W'],
  ['lensMinDeg', '°'], ['lensMaxDeg', '°'], ['panMaxDeg', '°'], ['tiltMaxDeg', '°'],
]

// ---------- Canais ----------
const channelId = ref('0')
const channel = computed(() => draft.value.channels[Number(channelId.value)])
const channelItems = computed(() =>
  draft.value.channels.map((c, i) => ({ id: String(i), label: `${i + 1}. ${c.name}`, detail: t(`attribute.${c.attribute}`) })),
)

function addChannel() {
  draft.value.channels.push({ name: t('editor.newChannel'), attribute: 'dimmer', fine: false, capabilities: [{ min: 0, max: 255, name: '', color: null }] })
  channelId.value = String(draft.value.channels.length - 1)
}

function removeChannel() {
  const removed = Number(channelId.value)
  draft.value.channels.splice(removed, 1)
  // Os modos guardam índices: tira o canal removido e corrige os que vinham depois
  for (const mode of draft.value.modes) mode.channels = mode.channels.filter((c) => c !== removed).map((c) => (c > removed ? c - 1 : c))
  channelId.value = String(Math.max(0, removed - 1))
}

// Avisos das faixas: valores sem significado, sobreposições e faixas invertidas
const capabilityIssues = computed(() => {
  const issues: string[] = []
  let next = 0
  for (const c of [...(channel.value?.capabilities ?? [])].sort((a, b) => a.min - b.min)) {
    if (c.min > c.max) issues.push(t('editor.capInverted', { min: c.min, max: c.max }))
    if (c.min > next) issues.push(t('editor.capGap', { from: next, to: c.min - 1 }))
    if (c.min < next) issues.push(t('editor.capOverlap', { from: c.min, to: Math.min(c.max, next - 1) }))
    next = Math.max(next, c.max + 1)
  }
  if (channel.value && next <= 255) issues.push(t('editor.capGap', { from: next, to: 255 }))
  return issues
})

// ---------- Modos ----------
const modeId = ref('0')
const mode = computed(() => draft.value.modes[Number(modeId.value)])
const modeItems = computed(() => draft.value.modes.map((m, i) => ({ id: String(i), label: m.name, detail: t('fixtures.channels', { n: m.channels.length }) })))
const channelToAdd = ref(0)

function addMode() {
  draft.value.modes.push({ name: t('editor.newMode'), channels: [], heads: [] })
  modeId.value = String(draft.value.modes.length - 1)
}

function move(position: number, delta: number) {
  const list = mode.value!.channels
  const [item] = list.splice(position, 1)
  list.splice(position + delta, 0, item!)
}

// ---------- Salvar ----------
const canSave = computed(() => draft.value.manufacturer.trim() !== '' && draft.value.model.trim() !== '')

function save() {
  const def = draft.value
  if (!def.id) def.id = `${def.manufacturer}-${def.model}`.toLowerCase().normalize('NFD').replace(/[^a-z0-9]+/g, '-')
  emit('save', def)
}
</script>

<template>
  <section class="panel @container min-h-0">
    <header class="flex flex-wrap items-center justify-between gap-2">
      <h2 class="text-lg font-bold">{{ draft.model || t('editor.untitled') }}</h2>
      <div class="flex gap-2">
        <RlButton size="sm" @press="emit('cancel')">{{ t('common.cancel') }}</RlButton>
        <RlButton size="sm" :model-value="canSave" color="var(--color-on)" highlight="led" @press="canSave && save()">
          {{ t('common.save') }}
        </RlButton>
      </div>
    </header>

    <RlTabs
      v-model="tab"
      class="min-h-0 flex-1"
      :tabs="[
        { value: 'general', label: t('editor.general') },
        { value: 'channels', label: t('editor.channels') },
        { value: 'modes', label: t('editor.modes') },
        { value: 'physical', label: t('editor.physical') },
      ]"
    >
      <template #general>
        <div class="grid gap-3 @xl:grid-cols-2">
          <label class="field-label">
            {{ t('editor.manufacturer') }}
            <input v-model="draft.manufacturer" class="field">
          </label>
          <label class="field-label">
            {{ t('editor.model') }}
            <input v-model="draft.model" class="field">
          </label>
          <label class="field-label">
            {{ t('editor.kind') }}
            <select v-model="draft.kind" class="field">
              <option v-for="kind in kinds" :key="kind" :value="kind">{{ t(`kind.${kind}`) }}</option>
            </select>
          </label>
        </div>
      </template>

      <template #channels>
        <div class="grid min-h-0 gap-3 @2xl:grid-cols-[220px_1fr]">
          <div class="flex flex-col gap-2">
            <RlList v-model="channelId" class="max-h-80" :items="channelItems" />
            <RlButton size="sm" @press="addChannel">
              <Plus :size="16" />
              {{ t('editor.addChannel') }}
            </RlButton>
          </div>
          <div v-if="channel" class="flex flex-col gap-3">
            <div class="grid gap-2 @xl:grid-cols-[1fr_1fr_auto] @xl:items-end">
              <label class="field-label">
                {{ t('editor.name') }}
                <input v-model="channel.name" class="field">
              </label>
              <label class="field-label">
                {{ t('editor.attribute') }}
                <select v-model="channel.attribute" class="field">
                  <option v-for="a in attributes" :key="a" :value="a">{{ t(`attribute.${a}`) }}</option>
                </select>
              </label>
              <RlSwitch v-model="channel.fine">{{ t('editor.fine') }}</RlSwitch>
            </div>
            <h3 class="heading">{{ t('editor.capabilities') }}</h3>
            <table class="rl-table">
              <thead>
                <tr>
                  <th>{{ t('editor.from') }}</th>
                  <th>{{ t('editor.to') }}</th>
                  <th>{{ t('editor.meaning') }}</th>
                  <th />
                </tr>
              </thead>
              <tbody>
                <tr v-for="(c, i) in channel.capabilities" :key="i">
                  <td><input v-model.number="c.min" type="number" min="0" max="255" class="field w-20"></td>
                  <td><input v-model.number="c.max" type="number" min="0" max="255" class="field w-20"></td>
                  <td><input v-model="c.name" class="field w-full"></td>
                  <td>
                    <RlButton size="sm" :aria-label="t('common.remove')" @press="channel.capabilities.splice(i, 1)"><Trash2 :size="16" /></RlButton>
                  </td>
                </tr>
              </tbody>
            </table>
            <ul v-if="capabilityIssues.length" class="flex flex-col gap-1 text-sm text-flash">
              <li v-for="issue in capabilityIssues" :key="issue" class="flex items-center gap-2">
                <TriangleAlert :size="14" />
                {{ issue }}
              </li>
            </ul>
            <div class="flex flex-wrap gap-2">
              <RlButton size="sm" @press="channel.capabilities.push({ min: 0, max: 255, name: '', color: null })">
                <Plus :size="16" />
                {{ t('editor.addCapability') }}
              </RlButton>
              <RlButton size="sm" color="var(--color-danger)" @press="removeChannel">
                <Trash2 :size="16" />
                {{ t('editor.removeChannel') }}
              </RlButton>
            </div>
          </div>
        </div>
      </template>

      <template #modes>
        <div class="grid min-h-0 gap-3 @2xl:grid-cols-[220px_1fr]">
          <div class="flex flex-col gap-2">
            <RlList v-model="modeId" class="max-h-80" :items="modeItems" />
            <RlButton size="sm" @press="addMode">
              <Plus :size="16" />
              {{ t('editor.addMode') }}
            </RlButton>
          </div>
          <div v-if="mode" class="flex flex-col gap-3">
            <label class="field-label">
              {{ t('editor.name') }}
              <input v-model="mode.name" class="field">
            </label>
            <table class="rl-table">
              <thead>
                <tr>
                  <th>DMX</th>
                  <th>{{ t('editor.channel') }}</th>
                  <th />
                </tr>
              </thead>
              <tbody>
                <tr v-for="(index, position) in mode.channels" :key="position">
                  <td class="font-lcd text-lcd">{{ position + 1 }}</td>
                  <td>{{ draft.channels[index]?.name }}</td>
                  <td class="flex justify-end gap-1">
                    <RlButton size="sm" :aria-label="t('editor.up')" @press="position > 0 && move(position, -1)"><ArrowUp :size="16" /></RlButton>
                    <RlButton size="sm" :aria-label="t('editor.down')" @press="position < mode.channels.length - 1 && move(position, 1)"><ArrowDown :size="16" /></RlButton>
                    <RlButton size="sm" :aria-label="t('common.remove')" @press="mode.channels.splice(position, 1)"><Trash2 :size="16" /></RlButton>
                  </td>
                </tr>
              </tbody>
            </table>
            <div class="flex gap-2">
              <select v-model.number="channelToAdd" class="field min-w-0 flex-1">
                <option v-for="(c, i) in draft.channels" :key="i" :value="i">{{ i + 1 }}. {{ c.name }}</option>
              </select>
              <RlButton :disabled="!draft.channels.length" @press="draft.channels.length && mode.channels.push(channelToAdd)">
                <Plus :size="16" />
                {{ t('editor.addToMode') }}
              </RlButton>
            </div>
          </div>
        </div>
      </template>

      <template #physical>
        <div class="grid gap-3 @xl:grid-cols-3">
          <label v-for="[key, unit] in physicalFields" :key="key" class="field-label">
            {{ t(`editor.${key}`) }} ({{ unit }})
            <input v-model.number="draft.physical[key]" type="number" min="0" class="field">
          </label>
        </div>
      </template>
    </RlTabs>
  </section>
</template>

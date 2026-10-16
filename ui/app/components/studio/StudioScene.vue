<script setup lang="ts">
import { Grid3x3, Plus } from 'lucide-vue-next'
import type { Attribute } from '~/types/generated/Attribute'
import type { Fixture } from '~/types/generated/Fixture'
import type { Function } from '~/types/generated/Function'
import type { SceneValue } from '~/types/generated/SceneValue'

type Scene = Extract<Function, { type: 'scene' }>

// Editor de cena (E12): a fixture escolhida recebe os valores, pelos canais um a um ou pelas
// ferramentas de atributo que fazem sentido para ela. Ctrl + clique (ou toque longo) soma outras do
// mesmo tipo, e o que se mexe vale para todas. Os grupos de canais da cena mexem em várias de uma vez.
const props = defineProps<{ fn: Scene }>()
const { t } = useI18n()
const { format } = useValueFormat()
const { state, send } = useEngine()
const project = computed(() => state.value.project)
const selection = ref<string[]>([])
const tool = ref('channels')
const editingFixtures = ref(false)
const sameType = ref(false)
const pickingGroup = ref(false)

// Atributos de cada ferramenta: knobs para níveis, botões para as faixas (gobo, roda de cores…)
const panels: Record<'color' | 'effects', { knobs: Attribute[]; caps: Attribute[] }> = {
  color: { knobs: ['white', 'amber', 'uv'], caps: ['colorWheel', 'colorMacro'] },
  effects: { knobs: ['zoom', 'focus', 'iris', 'goboRotation'], caps: ['shutter', 'gobo', 'prism'] },
}
// A aba aparece só se a fixture tem algum destes atributos
const toolAttributes: Record<string, Attribute[]> = {
  intensity: ['dimmer'],
  color: ['red', ...panels.color.knobs, ...panels.color.caps],
  position: ['pan'],
  effects: [...panels.effects.knobs, ...panels.effects.caps],
}

const inScene = (id: string) => props.fn.values.some((v) => v.fixture === id)
const fixtures = computed(() => project.value.fixtures.filter((f) => inScene(f.id)))
// Escolhidas (todas do mesmo tipo); sem escolha, a primeira da cena. A primeira é a referência dos valores
const selected = computed(() => {
  const list = fixtures.value.filter((f) => selection.value.includes(f.id))
  return list.length ? list : fixtures.value.slice(0, 1)
})
const fixture = computed(() => selected.value[0])
const channels = computed(() => (fixture.value ? modeChannels(project.value, fixture.value) : []))
const valueOf = (channel: number) => props.fn.values.find((v) => v.fixture === fixture.value?.id && v.channel === channel)?.value ?? 0
const channelsWith = (a: Attribute) => channels.value.flatMap((c, i) => (c.attribute === a && !c.fine ? [i] : []))
// Valor (0–255) do atributo na fixture escolhida; undefined se ela não tem o atributo
const attr = (a: Attribute) => {
  const channel = channelsWith(a)[0]
  return channel === undefined ? undefined : valueOf(channel)
}
const present = (list: Attribute[]) => list.filter((a) => attr(a) !== undefined)
const capsOf = (a: Attribute) => channels.value[channelsWith(a)[0]!]!.capabilities
const tools = computed(() => ['channels', ...Object.keys(toolAttributes).filter((name) => present(toolAttributes[name]!).length)])
const groups = computed(() =>
  props.fn.channelGroups.flatMap((level) => project.value.channelGroups.filter((g) => g.id === level.group).map((g) => ({ ...g, value: level.value }))),
)

watch(tools, (list) => {
  if (!list.includes(tool.value)) tool.value = 'channels'
})

// Junta valores novos aos da cena (os canais que ainda não estão nela entram)
function merge(list: SceneValue[]) {
  const key = (v: SceneValue) => `${v.fixture}:${v.channel}`
  const all = new Map(props.fn.values.map((v) => [key(v), v]))
  for (const v of list) all.set(key(v), v)
  return [...all.values()]
}

function save(changes: Partial<Scene>) {
  send({ op: 'saveFunction', function: { ...props.fn, ...changes } })
}

// Grava valores (0–255) por canal em todas as escolhidas (mesmo tipo, então os mesmos canais)
function setChannels(values: [channel: number, value: number][]) {
  save({ values: merge(selected.value.flatMap((f) => values.map(([channel, value]) => ({ fixture: f.id, channel, value: Math.round(value) })))) })
}

// Toque escolhe só esta; Ctrl + clique ou toque longo soma (ou tira) outra do mesmo modelo e modo.
// A última escolhida não sai.
function pick(f: Fixture, add: boolean) {
  const ids = selected.value.map((x) => x.id)
  const same = fixture.value && fixture.value.defId === f.defId && fixture.value.mode === f.mode
  if (!add || !same) selection.value = [f.id]
  else if (!ids.includes(f.id)) selection.value = [...ids, f.id]
  else if (ids.length > 1) selection.value = ids.filter((id) => id !== f.id)
}

// Grava por atributo, em todos os canais dele (ex.: o vermelho de cada pixel de uma barra)
function setAttributes(values: Partial<Record<Attribute, number>>) {
  setChannels(Object.entries(values).flatMap(([a, value]) => channelsWith(a as Attribute).map((channel): [number, number] => [channel, value])))
}

const color = computed({
  get: () => `#${(['red', 'green', 'blue'] as const).map((a) => (attr(a) ?? 0).toString(16).padStart(2, '0')).join('')}`,
  set: (hex) => {
    const [red, green, blue] = [1, 3, 5].map((i) => parseInt(hex.slice(i, i + 2), 16))
    setAttributes({ red, green, blue })
  },
})

const position = computed({
  get: () => ({ x: (attr('pan') ?? 0) / 255, y: (attr('tilt') ?? 0) / 255 }),
  set: ({ x, y }) => setAttributes({ pan: x * 255, tilt: y * 255 }),
})

// Valores zerados de todos os canais das fixtures que ainda não estão na cena (ao entrar nela)
const zeros = (ids: string[]) =>
  project.value.fixtures
    .filter((f) => ids.includes(f.id) && !inScene(f.id))
    .flatMap((f) => modeChannels(project.value, f).map((_, channel) => ({ fixture: f.id, channel, value: 0 })))

// Tocar inclui ou tira a fixture; com "do mesmo tipo", vale para todas do mesmo modelo
function toggleFixture(id: string) {
  const defId = project.value.fixtures.find((f) => f.id === id)!.defId
  const ids = sameType.value ? project.value.fixtures.filter((f) => f.defId === defId).map((f) => f.id) : [id]
  save({ values: inScene(id) ? props.fn.values.filter((v) => !ids.includes(v.fixture)) : [...props.fn.values, ...zeros(ids)] })
}

function addFixtureGroup(id: string) {
  save({ values: [...props.fn.values, ...zeros(project.value.fixtureGroups.find((g) => g.id === id)!.cells.map((c) => c.fixture))] })
  pickingGroup.value = false
}

// Ligar um grupo de canais traz para a cena as fixtures dele que ainda não estão
function toggleGroup(id: string) {
  const rest = props.fn.channelGroups.filter((g) => g.group !== id)
  if (rest.length < props.fn.channelGroups.length) return save({ channelGroups: rest })
  const ids = project.value.channelGroups.find((g) => g.id === id)?.channels.map((c) => c.fixture) ?? []
  save({ channelGroups: [...rest, { group: id, value: 0 }], values: [...props.fn.values, ...zeros(ids)] })
}

// O fader do grupo grava o nível e os valores dos canais do grupo, só nas fixtures que estão na cena
function setGroup(id: string, v: number) {
  const value = Math.round(v * 255)
  const groupChannels = project.value.channelGroups.find((g) => g.id === id)?.channels.filter((c) => inScene(c.fixture)) ?? []
  save({ channelGroups: props.fn.channelGroups.map((g) => (g.group === id ? { group: id, value } : g)), values: merge(groupChannels.map((c) => ({ ...c, value }))) })
}
</script>

<template>
  <div class="flex flex-col gap-3">
    <section class="grid gap-2">
      <h3 class="heading flex items-center gap-2">
        {{ t('scene.fixtures') }}
        <RlInfo :text="t('scene.fixturesInfo')" />
      </h3>
      <div class="flex flex-wrap gap-1.5">
        <RlButton size="sm" @press="editingFixtures = true">
          <Plus :size="14" />
          {{ t('scene.addFixtures') }}
        </RlButton>
        <RlButton
          v-for="f in fixtures"
        :key="f.id"
          size="sm"
          highlight="led"
          :color="f.color"
          :model-value="selected.includes(f)"
          @press="(e) => pick(f, e.ctrlKey || e.metaKey)"
          @longpress="pick(f, true)"
        >
          {{ f.name }}
        </RlButton>
      </div>
    </section>

    <RlTabs v-if="fixture" v-model="tool" class="mt-2" :tabs="tools.map((value) => ({ value, label: t(`scene.${value}`) }))">
      <template #channels>
        <div class="flex h-96 overflow-x-auto rounded-md border border-black">
          <RlChannelStrip
            v-for="(c, i) in channels"
            :key="i"
            plain
            :label="c.name"
            :color="attributeLook[c.attribute].color"
            :model-value="valueOf(i) / 255"
            @update:model-value="(v) => setChannels([[i, v * 255]])"
          >
            <template #label>
              <component :is="attributeLook[c.attribute].icon" :size="12" class="inline" />
              {{ c.name }}
            </template>
          </RlChannelStrip>
        </div>
      </template>

      <template #intensity>
        <div class="flex h-80 w-24 flex-col gap-2">
          <RlLcd>{{ format(attr('dimmer')! / 255) }}</RlLcd>
          <RlFader class="flex-1" :model-value="attr('dimmer')! / 255" @update:model-value="(v) => setAttributes({ dimmer: v * 255 })" />
        </div>
      </template>

      <template v-for="(panel, name) in panels" :key="name" #[name]>
        <div class="flex flex-wrap items-start gap-5">
          <RlColorPicker v-if="name === 'color' && attr('red') !== undefined" v-model="color" class="w-full max-w-xs" />
          <div v-if="present(panel.knobs).length" class="flex flex-wrap gap-4">
            <div v-for="a in present(panel.knobs)" :key="a" class="flex flex-col items-center gap-1">
              <RlKnob :model-value="attr(a)! / 255" :color="attributeLook[a].color" @update:model-value="(v) => setAttributes({ [a]: v * 255 })" />
              <span class="text-xs text-fg-muted">{{ t(`attribute.${a}`) }} · {{ format(attr(a)! / 255) }}</span>
            </div>
          </div>
          <div v-for="a in present(panel.caps)" :key="a" class="grid gap-1.5">
            <h4 class="field-label">{{ t(`attribute.${a}`) }}</h4>
            <div class="flex flex-wrap gap-1.5">
              <RlButton
                v-for="cap in capsOf(a)"
                :key="cap.min"
                size="sm"
                highlight="led"
                :color="cap.color ?? 'var(--color-select)'"
                :model-value="attr(a)! >= cap.min && attr(a)! <= cap.max"
                @press="setAttributes({ [a]: cap.min })"
              >
                {{ cap.name }}
              </RlButton>
            </div>
          </div>
        </div>
      </template>

      <template #position>
        <RlXYPad v-model="position" class="aspect-square w-full max-w-xs" />
      </template>
    </RlTabs>

    <!-- Divisor: acima, a fixture escolhida; abaixo, os grupos (várias fixtures de uma vez) -->
    <section class="divider grid gap-2">
      <h3 class="heading flex items-center gap-2">
        {{ t('groups.channelGroups') }}
        <RlInfo :text="t('scene.groupsHint')" />
      </h3>
      <div class="flex flex-wrap gap-1.5">
        <RlButton
          v-for="g in project.channelGroups"
          :key="g.id"
          size="sm"
          highlight="led"
          :model-value="fn.channelGroups.some((x) => x.group === g.id)"
          @press="toggleGroup(g.id)"
        >
          <component :is="channelGroupLook(project, g).icon" :size="14" :style="{ color: channelGroupLook(project, g).color }" />
          {{ g.name }}
        </RlButton>
      </div>
      <div v-if="groups.length" class="flex h-80 overflow-x-auto rounded-md border border-black">
        <RlChannelStrip
          v-for="g in groups"
          :key="g.id"
          plain
          :label="g.name"
          :color="channelGroupLook(project, g).color"
          :model-value="g.value / 255"
          @update:model-value="(v) => setGroup(g.id, v)"
        >
          <template #label>
            <component :is="channelGroupLook(project, g).icon" :size="12" class="inline" />
            {{ g.name }}
          </template>
        </RlChannelStrip>
      </div>
    </section>

    <!-- Tempos por último: no show, o console também ajusta os fades na hora (VC-3) -->
    <section class="divider grid gap-2">
      <h3 class="heading">{{ t('scene.times') }}</h3>
      <div class="grid grid-cols-2 gap-2 @xl:max-w-sm">
        <label class="field-label">
          {{ t('studio.fadeIn') }}
          <RlTimeField :model-value="fn.fadeInMs" @update:model-value="(ms) => save({ fadeInMs: ms })" />
        </label>
        <label class="field-label">
          {{ t('studio.fadeOut') }}
          <RlTimeField :model-value="fn.fadeOutMs" @update:model-value="(ms) => save({ fadeOutMs: ms })" />
        </label>
      </div>
    </section>

    <RlDialog v-model:open="editingFixtures" :title="t('scene.addFixtures')" :info="t('scene.addFixturesInfo')" sheet>
      <div class="flex flex-wrap gap-1.5">
        <RlButton size="sm" @press="save({ values: [] })">{{ t('scene.clearFixtures') }}</RlButton>
        <RlButton v-model="sameType" size="sm" mode="toggle" highlight="led">{{ t('scene.sameType') }}</RlButton>
        <RlButton size="sm" highlight="led" :model-value="pickingGroup" @press="pickingGroup = !pickingGroup">{{ t('scene.pickGroup') }}</RlButton>
      </div>
      <div v-if="pickingGroup" class="flex flex-wrap gap-1.5">
        <RlButton v-for="g in project.fixtureGroups" :key="g.id" size="sm" @press="addFixtureGroup(g.id)">
          <Grid3x3 :size="14" />
          {{ g.name }}
        </RlButton>
      </div>
      <!-- A lista só serve de botão: tocar na linha inclui ou tira a fixture (o LED mostra se está na cena) -->
      <RlList
        class="min-h-0"
        :model-value="undefined"
        :items="project.fixtures.map((f) => ({ id: f.id, label: f.name, color: f.color, universe: project.universes.find((u) => u.id === f.universe)?.name, end: f.address + channelCount(project, f) - 1, address: f.address }))"
        @update:model-value="(id) => id && toggleFixture(id)"
      >
        <template #item="{ item }">
          <span class="rl-led" :style="{ '--led': inScene(item.id) ? 'var(--color-on)' : 'var(--color-fg-dim)', opacity: inScene(item.id) ? 1 : 0.3 }" />
          <span class="h-6 w-1.5 shrink-0 rounded-sm" :style="{ background: item.color }" />
          <span class="min-w-0 flex-1 truncate font-semibold">{{ item.label }}</span>
          <span class="shrink-0 text-xs text-fg-muted">{{ item.universe }}</span>
          <span class="w-16 shrink-0 text-right font-lcd text-sm text-lcd">{{ item.address }}–{{ item.end }}</span>
        </template>
      </RlList>
    </RlDialog>
  </div>
</template>

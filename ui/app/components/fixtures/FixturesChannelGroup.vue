<script setup lang="ts">
import { Trash2 } from 'lucide-vue-next'
import type { Attribute } from '~/types/generated/Attribute'
import type { ChannelGroup } from '~/types/generated/ChannelGroup'

// Grupo de canais: toque nos canais de cada fixture para incluir ou tirar do grupo. Os atalhos
// por atributo incluem (ou tiram) de uma vez esse atributo em todas as fixtures, sem os canais finos.
const props = defineProps<{ group: ChannelGroup }>()
const { t } = useI18n()
const { state, send } = useEngine()
const project = computed(() => state.value.project)
const confirmRemove = ref(false)

watch(() => props.group.id, () => (confirmRemove.value = false))

const fixtures = computed(() => project.value.fixtures.map((f) => ({ ...f, channels: modeChannels(project.value, f) })))
const attributes = computed(() => [...new Set(fixtures.value.flatMap((f) => f.channels.map((c) => c.attribute)))])
const look = computed(() => channelGroupLook(project.value, props.group))
const has = (fixture: string, channel: number) => props.group.channels.some((c) => c.fixture === fixture && c.channel === channel)
const withAttribute = (a: Attribute) =>
  fixtures.value.flatMap((f) => f.channels.flatMap((c, channel) => (c.attribute === a && !c.fine ? [{ fixture: f.id, channel }] : [])))
const hasAll = (a: Attribute) => withAttribute(a).every((c) => has(c.fixture, c.channel))

function save(changes: Partial<ChannelGroup>) {
  send({ op: 'saveChannelGroup', group: { ...props.group, ...changes } })
}

function toggle(fixture: string, channel: number) {
  const rest = props.group.channels.filter((c) => c.fixture !== fixture || c.channel !== channel)
  save({ channels: has(fixture, channel) ? rest : [...rest, { fixture, channel }] })
}

function toggleAttribute(a: Attribute) {
  const all = withAttribute(a)
  const rest = props.group.channels.filter((c) => !all.some((x) => x.fixture === c.fixture && x.channel === c.channel))
  save({ channels: hasAll(a) ? rest : [...rest, ...all] })
}

function remove() {
  if (!confirmRemove.value) return (confirmRemove.value = true)
  send({ op: 'removeChannelGroup', id: props.group.id })
}
</script>

<template>
  <section class="panel min-h-0 overflow-auto">
    <div class="flex items-end gap-2">
      <label class="field-label flex-1">
        {{ t('fixtures.name') }}
        <input class="field" :value="group.name" @change="save({ name: ($event.target as HTMLInputElement).value })">
      </label>
      <RlButton :model-value="confirmRemove" color="var(--color-danger)" @press="remove">
        <Trash2 :size="16" />
        {{ confirmRemove ? t('common.confirmRemove') : t('common.remove') }}
      </RlButton>
    </div>
    <p class="flex items-center gap-2 text-sm text-fg-muted">
      <component :is="look.icon" :size="18" :style="{ color: look.color }" />
      {{ t('groups.channelCount', group.channels.length) }}
    </p>

    <h3 class="heading">{{ t('groups.byAttribute') }}</h3>
    <div class="flex flex-wrap gap-1.5">
      <RlButton
        v-for="a in attributes"
        :key="a"
        size="sm"
        highlight="led"
        :color="attributeLook[a].color"
        :model-value="hasAll(a)"
        @press="toggleAttribute(a)"
      >
        <component :is="attributeLook[a].icon" :size="14" :style="{ color: attributeLook[a].color }" />
        {{ t(`attribute.${a}`) }}
      </RlButton>
    </div>

    <h3 class="heading">{{ t('groups.channelsByFixture') }}</h3>
    <div v-for="f in fixtures" :key="f.id" class="grid gap-1.5 border-t border-black pt-2 @3xl:grid-cols-[110px_1fr]">
      <span class="flex items-center gap-2 text-sm">
        <span class="rl-led" :style="{ '--led': f.color }" />
        {{ f.name }}
      </span>
      <div class="flex flex-wrap gap-1.5">
        <RlButton
          v-for="(c, i) in f.channels"
          :key="i"
          size="sm"
          highlight="led"
          :color="attributeLook[c.attribute].color"
          :model-value="has(f.id, i)"
          @press="toggle(f.id, i)"
        >
          <component :is="attributeLook[c.attribute].icon" :size="14" :style="{ color: attributeLook[c.attribute].color }" />
          {{ c.name }}
        </RlButton>
      </div>
    </div>
  </section>
</template>

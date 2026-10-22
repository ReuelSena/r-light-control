<script setup lang="ts">
import { Grid3x3, Plus } from 'lucide-vue-next'

// Grupos de fixtures (grade 2D de cabeças, para o RGB Matrix) e grupos de canais (canais de
// quaisquer fixtures, para faders do console e da mesa simples e para cenas).
const { t } = useI18n()
const { state, send } = useEngine()
const project = computed(() => state.value.project)
const selectedId = ref(project.value.fixtureGroups[0]?.id)
const fixtureGroup = computed(() => project.value.fixtureGroups.find((g) => g.id === selectedId.value))
const channelGroup = computed(() => project.value.channelGroups.find((g) => g.id === selectedId.value))
const name = () => t('groups.newName', { n: project.value.fixtureGroups.length + project.value.channelGroups.length + 1 })

async function createFixtureGroup() {
  const id = uuidv7()
  await send({ op: 'saveFixtureGroup', group: { id, name: name(), width: 4, height: 2, cells: [] } })
  selectedId.value = id
}

async function createChannelGroup() {
  const id = uuidv7()
  await send({ op: 'saveChannelGroup', group: { id, name: name(), fixtureGroup: null, attributes: [], channels: [] } })
  selectedId.value = id
}
</script>

<template>
  <div class="grid gap-3 @3xl:h-full @3xl:grid-cols-[260px_1fr]">
    <section class="panel min-h-0 overflow-auto">
      <div class="flex items-center justify-between gap-2">
        <h2 class="heading">{{ t('groups.fixtureGroups') }}</h2>
        <RlButton size="sm" :aria-label="t('groups.newFixtureGroup')" @press="createFixtureGroup"><Plus :size="16" /></RlButton>
      </div>
      <RlList
        v-model="selectedId"
        reorderable
        class="shrink-0"
        :items="project.fixtureGroups.map((g) => ({ id: g.id, label: g.name, detail: `${g.width}×${g.height}` }))"
        @reorder="(id, to) => send({ op: 'moveFixtureGroup', id, to })"
      >
        <template #item="{ item }">
          <Grid3x3 :size="18" class="shrink-0 text-fg-muted" />
          <span class="flex-1 truncate">{{ item.label }}</span>
          <small class="text-xs text-fg-muted">{{ item.detail }}</small>
        </template>
      </RlList>

      <div class="mt-2 flex items-center justify-between gap-2">
        <h2 class="heading">{{ t('groups.channelGroups') }}</h2>
        <RlButton size="sm" :aria-label="t('groups.newChannelGroup')" @press="createChannelGroup"><Plus :size="16" /></RlButton>
      </div>
      <RlList
        v-model="selectedId"
        reorderable
        class="shrink-0"
        :items="project.channelGroups.map((g) => ({ id: g.id, label: g.name, detail: t('groups.channelCount', g.channels.length), look: channelGroupLook(project, g) }))"
        @reorder="(id, to) => send({ op: 'moveChannelGroup', id, to })"
      >
        <template #item="{ item }">
          <component :is="item.look.icon" :size="18" class="shrink-0" :style="{ color: item.look.color }" />
          <span class="flex-1 truncate">{{ item.label }}</span>
          <small class="text-xs text-fg-muted">{{ item.detail }}</small>
        </template>
      </RlList>
    </section>

    <FixturesGroupGrid v-if="fixtureGroup" :group="fixtureGroup" @duplicated="selectedId = $event" />
    <FixturesChannelGroup v-else-if="channelGroup" :group="channelGroup" @duplicated="selectedId = $event" />
    <section v-else class="panel justify-center text-center text-fg-muted">{{ t('groups.selectHint') }}</section>
  </div>
</template>

<script setup lang="ts">
import { LocateFixed, Plus, Trash2, TriangleAlert } from 'lucide-vue-next'
import type { Fixture } from '~/types/generated/Fixture'

// Fixtures do projeto: lista e edição (nome, cor, modo, universo e endereço).
const emit = defineEmits<{ add: [] }>()
const { t } = useI18n()
const { state, send } = useEngine()
const project = computed(() => state.value.project)
const clashing = computed(() => conflicts(project.value))
const selectedId = ref<string>()
const selected = computed(() => project.value.fixtures.find((f) => f.id === selectedId.value))
const def = computed(() => selected.value && findDef(project.value, selected.value.defId))
const confirmRemove = ref(false)
// Localizar acompanha a fixture escolhida (para conferir o patch no palco)
const locating = useLocate(computed(() => (selectedId.value ? [selectedId.value] : [])))

watch(selectedId, () => (confirmRemove.value = false))

const universeName = (id: string) => project.value.universes.find((u) => u.id === id)?.name ?? '?'
const value = (e: Event) => (e.target as HTMLInputElement | HTMLSelectElement).value

function update(changes: Partial<Fixture>) {
  if (selected.value) send({ op: 'updateFixture', fixture: { ...selected.value, ...changes } })
}

function remove() {
  if (!confirmRemove.value) return (confirmRemove.value = true)
  send({ op: 'removeFixtures', ids: [selectedId.value!] })
  selectedId.value = undefined
}
</script>

<template>
  <div class="grid h-full gap-3 @3xl:grid-cols-[1fr_320px]">
    <section class="panel min-h-0">
      <div class="flex items-center justify-between gap-2">
        <h2 class="heading">{{ t('fixtures.count', { n: project.fixtures.length }) }}</h2>
        <RlButton size="sm" @press="emit('add')">
          <Plus :size="16" />
          {{ t('fixtures.add') }}
        </RlButton>
      </div>
      <RlList v-model="selectedId" class="min-h-0 flex-1" :items="project.fixtures">
        <template #item="{ item }">
          <span class="h-6 w-1.5 shrink-0 rounded-sm" :style="{ background: item.color }" />
          <span class="min-w-0 flex-1">
            <span class="block truncate font-semibold">{{ item.name }}</span>
            <span class="block truncate text-xs text-fg-muted">{{ findDef(project, item.defId)?.model }} · {{ item.mode }}</span>
          </span>
          <TriangleAlert v-if="clashing.has(item.id)" :size="16" class="shrink-0 text-danger" />
          <span class="shrink-0 font-lcd text-sm text-lcd">
            {{ universeName(item.universe) }} · {{ item.address }}–{{ item.address + channelCount(project, item) - 1 }}
          </span>
        </template>
      </RlList>
    </section>

    <section v-if="selected && def" class="panel overflow-auto">
      <div class="flex items-start justify-between gap-2">
        <div class="min-w-0">
          <h2 class="text-lg font-bold">{{ selected.name }}</h2>
          <p class="text-sm text-fg-muted">{{ def.manufacturer }} {{ def.model }}</p>
        </div>
        <RlButton v-model="locating" size="sm" mode="toggle" highlight="led" color="var(--color-flash)">
          <LocateFixed :size="16" />
          {{ t('fixtures.locate') }}
        </RlButton>
      </div>
      <label class="field-label">
        {{ t('fixtures.name') }}
        <input class="field" :value="selected.name" @change="update({ name: value($event) })">
      </label>
      <div class="field-label">
        {{ t('fixtures.color') }}
        <RlSwatches :model-value="selected.color" :colors="tagColors" @update:model-value="update({ color: $event })" />
      </div>
      <label class="field-label">
        {{ t('fixtures.mode') }}
        <select class="field" :value="selected.mode" @change="update({ mode: value($event) })">
          <option v-for="m in def.modes" :key="m.name" :value="m.name">{{ m.name }}</option>
        </select>
      </label>
      <div class="grid grid-cols-2 gap-2">
        <label class="field-label">
          {{ t('fixtures.universe') }}
          <select class="field" :value="selected.universe" @change="update({ universe: value($event) })">
            <option v-for="u in project.universes" :key="u.id" :value="u.id">{{ u.name }}</option>
          </select>
        </label>
        <label class="field-label">
          {{ t('fixtures.address') }}
          <input type="number" min="1" max="512" class="field" :value="selected.address" @change="update({ address: Number(value($event)) })">
        </label>
      </div>
      <p v-if="clashing.has(selected.id)" class="flex items-center gap-2 text-sm text-danger">
        <TriangleAlert :size="16" />
        {{ t('patch.conflictHint') }}
      </p>
      <RlButton class="mt-auto" :model-value="confirmRemove" color="var(--color-danger)" @press="remove">
        <Trash2 :size="16" />
        {{ confirmRemove ? t('common.confirmRemove') : t('common.remove') }}
      </RlButton>
    </section>
    <section v-else class="panel justify-center text-center text-fg-muted">{{ t('fixtures.selectHint') }}</section>
  </div>
</template>

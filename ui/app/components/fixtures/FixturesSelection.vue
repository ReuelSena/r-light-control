<script setup lang="ts">
import { Copy, Grid3x3, Trash2 } from 'lucide-vue-next'
import type { Member } from '~/types/generated/Member'
import type { SelectionGroup } from '~/types/generated/SelectionGroup'

// Seleção salva (VC-7): fixtures e grupos de fixtures misturados, sem grade, que podem se repetir entre
// seleções. O grupo vale para quem está nele na hora de usar (N1).
const props = defineProps<{ selection: SelectionGroup }>()
const emit = defineEmits<{ duplicated: [id: string] }>()
const { t } = useI18n()
const { state, send } = useEngine()
const project = computed(() => state.value.project)
const confirmRemove = ref(false)

watch(() => props.selection.id, () => (confirmRemove.value = false))

const hasFixture = (id: string) => props.selection.members.some((m) => 'fixture' in m && m.fixture === id)
const hasGroup = (id: string) => props.selection.members.some((m) => 'group' in m && m.group === id)

function save(changes: Partial<SelectionGroup>) {
  send({ op: 'saveSelection', selection: { ...props.selection, ...changes } })
}

// Toque inclui o membro no fim da lista ou o tira
function toggle(member: Member, on: boolean) {
  const rest = props.selection.members.filter((m) => JSON.stringify(m) !== JSON.stringify(member))
  save({ members: on ? rest : [...rest, member] })
}

async function duplicate() {
  const id = uuidv7()
  await send({ op: 'saveSelection', selection: { ...props.selection, id, name: t('studio.copyName', { name: props.selection.name }) } })
  emit('duplicated', id)
}

function remove() {
  if (!confirmRemove.value) return (confirmRemove.value = true)
  send({ op: 'removeSelection', id: props.selection.id })
}
</script>

<template>
  <section class="panel min-h-0 overflow-auto">
    <div class="flex items-end gap-2">
      <label class="field-label flex-1">
        {{ t('fixtures.name') }}
        <input class="field" :value="selection.name" @change="save({ name: ($event.target as HTMLInputElement).value })">
      </label>
      <RlButton @press="duplicate">
        <Copy :size="16" />
        {{ t('common.duplicate') }}
      </RlButton>
      <RlButton :model-value="confirmRemove" color="var(--color-danger)" @press="remove">
        <Trash2 :size="16" />
        {{ confirmRemove ? t('common.confirmRemove') : t('common.remove') }}
      </RlButton>
    </div>
    <p class="text-sm text-fg-muted">{{ t('groups.memberCount', selection.members.length) }}</p>

    <h3 class="heading flex items-center gap-2">
      {{ t('groups.fixtureGroups') }}
      <RlInfo :text="t('groups.selectionInfo')" />
    </h3>
    <div class="flex flex-wrap gap-1.5">
      <RlButton v-for="{ group: g, depth } in groupTree(project)" :key="g.id" size="sm" highlight="led" :model-value="hasGroup(g.id)" @press="toggle({ group: g.id }, hasGroup(g.id))">
        <Grid3x3 :size="14" />
        {{ '— '.repeat(depth) }}{{ g.name }}
      </RlButton>
    </div>

    <h3 class="heading">{{ t('fixtures.project') }}</h3>
    <div class="flex flex-wrap gap-1.5">
      <RlButton v-for="f in project.fixtures" :key="f.id" size="sm" highlight="led" :model-value="hasFixture(f.id)" @press="toggle({ fixture: f.id }, hasFixture(f.id))">
        <span class="rl-led" :style="{ '--led': f.color }" />
        {{ f.name }}
      </RlButton>
    </div>
  </section>
</template>

<script setup lang="ts">
import { Plus, Trash2 } from 'lucide-vue-next'
import type { Function } from '~/types/generated/Function'

// Coleção: funções disparadas juntas.
const props = defineProps<{ fn: Extract<Function, { type: 'collection' }> }>()
const { t } = useI18n()
const { state, send } = useEngine()
const adding = ref(false)
const members = computed(() => props.fn.functions.flatMap((id) => state.value.project.functions.filter((f) => f.id === id)))

function save(functions: string[]) {
  send({ op: 'saveFunction', function: { ...props.fn, functions } })
}
</script>

<template>
  <div class="flex flex-col gap-3">
    <div class="flex items-center justify-between gap-2">
      <h3 class="heading">{{ t('collection.functions', { n: members.length }) }}</h3>
      <RlButton size="sm" @press="adding = true">
        <Plus :size="16" />
        {{ t('collection.add') }}
      </RlButton>
    </div>
    <div v-for="f in members" :key="f.id" class="sunken flex items-center gap-3 rounded-md p-2 pl-3">
      <component :is="functionIcons[f.type]" :size="18" class="shrink-0 text-fg-muted" />
      <span class="flex-1 truncate">{{ f.name }}</span>
      <RlButton size="sm" :aria-label="t('common.remove')" @press="save(fn.functions.filter((id) => id !== f.id))"><Trash2 :size="14" /></RlButton>
    </div>
    <p v-if="!members.length" class="text-sm text-fg-muted">{{ t('collection.empty') }}</p>

    <RlDialog v-model:open="adding" :title="t('collection.add')" sheet>
      <div class="grid gap-1.5">
        <RlButton
          v-for="f in state.project.functions.filter((x) => x.id !== fn.id && !fn.functions.includes(x.id) && !contains(state.project.functions, x, fn.id))"
          :key="f.id"
          class="justify-start!"
          @press="save([...fn.functions, f.id]); adding = false"
        >
          <component :is="functionIcons[f.type]" :size="18" />
          {{ f.name }}
        </RlButton>
      </div>
    </RlDialog>
  </div>
</template>

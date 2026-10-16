<script setup lang="ts">
import { ChevronDown, FilePlus, Save } from 'lucide-vue-next'
import type { Op } from '~/types/generated/Op'

// Menu Projeto: novo, salvar, salvar como e recentes (os arquivos de verdade chegam na F5).
const { t } = useI18n()
const { state, send } = useEngine()
const open = ref(false)
const saveAsName = ref('')
// "Novo" com alterações não salvas pede um segundo toque
const confirmNew = ref(false)

const recent = computed(() => state.value.recentProjects.map((path) => ({ id: path, label: path.split('\\').pop()!, detail: path })))

async function run(op: Op) {
  await send(op)
  open.value = false
  confirmNew.value = false
}

function newProject() {
  if (state.value.dirty && !confirmNew.value) confirmNew.value = true
  else run({ op: 'newProject' })
}
</script>

<template>
  <RlButton class="min-w-0" @press="open = true">
    <span class="max-w-36 truncate normal-case">{{ state.project.name }}</span>
    <span v-if="state.dirty" class="text-flash" :title="t('project.unsaved')">●</span>
    <ChevronDown :size="16" />
  </RlButton>

  <RlDialog v-model:open="open" :title="t('project.title')">
    <div class="flex flex-col gap-3">
      <p class="text-sm break-all text-fg-muted">{{ state.projectPath ?? t('project.notSaved') }}</p>
      <div class="grid grid-cols-2 gap-2">
        <RlButton :model-value="confirmNew" color="var(--color-danger)" @press="newProject">
          <FilePlus :size="16" />
          {{ confirmNew ? t('project.discard') : t('project.new') }}
        </RlButton>
        <RlButton @press="run({ op: 'saveProject', path: null })">
          <Save :size="16" />
          {{ t('project.save') }}
        </RlButton>
      </div>
      <label class="field-label">
        {{ t('project.saveAs') }}
        <span class="flex gap-2">
          <input v-model="saveAsName" class="field min-w-0 flex-1" :placeholder="state.project.name">
          <RlButton @press="run({ op: 'saveProject', path: `C:\\Shows\\${saveAsName || state.project.name}.rlc` })">
            {{ t('project.saveAsButton') }}
          </RlButton>
        </span>
      </label>
      <h3 class="heading">{{ t('project.recent') }}</h3>
      <RlList :items="recent" @update:model-value="(path) => path && run({ op: 'openProject', path })" />
    </div>
  </RlDialog>
</template>

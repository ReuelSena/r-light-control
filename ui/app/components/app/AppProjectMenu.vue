<script setup lang="ts">
import { ChevronDown, FilePlus } from 'lucide-vue-next'
import type { Op } from '~/types/generated/Op'

// Menu Projeto: novo projeto. Salvar, salvar como e recentes voltam na F5, com os arquivos de verdade.
const { t } = useI18n()
const { state, send } = useEngine()
const open = ref(false)
// "Novo" com alterações não salvas pede um segundo toque
const confirmNew = ref(false)

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
      <RlButton :model-value="confirmNew" color="var(--color-danger)" @press="newProject">
        <FilePlus :size="16" />
        {{ confirmNew ? t('project.discard') : t('project.new') }}
      </RlButton>
      <p class="text-sm text-flash">{{ t('project.filesLater') }}</p>
    </div>
  </RlDialog>
</template>

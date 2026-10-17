<script setup lang="ts">
import { ChevronDown, FilePlus, Save } from 'lucide-vue-next'
import type { Op } from '~/types/generated/Op'

// Menu Projeto (A5): novo, salvar, salvar como (pelo nome, na pasta dos projetos) e abrir pela lista.
const { t } = useI18n()
const { state, send } = useEngine()
const open = ref(false)
const saveAsName = ref('')
// Com alterações não salvas, "Novo" e abrir outro projeto pedem um segundo toque
const confirmNew = ref(false)
const confirmOpen = ref<string>()

const projects = computed(() => state.value.projects.map((name) => ({ id: name, label: name })))

watch(open, () => {
  confirmNew.value = false
  confirmOpen.value = undefined
  saveAsName.value = ''
})

async function run(op: Op) {
  await send(op)
  open.value = false
}

function newProject() {
  if (state.value.dirty && !confirmNew.value) confirmNew.value = true
  else run({ op: 'newProject' })
}

function openProject(name?: string) {
  if (!name) return
  if (state.value.dirty && confirmOpen.value !== name) confirmOpen.value = name
  else run({ op: 'openProject', name })
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
      <p v-if="state.recovered" class="text-sm text-flash">{{ t('project.recovered') }}</p>
      <div class="grid grid-cols-2 gap-2">
        <RlButton :model-value="confirmNew" color="var(--color-danger)" @press="newProject">
          <FilePlus :size="16" />
          {{ confirmNew ? t('project.discard') : t('project.new') }}
        </RlButton>
        <RlButton @press="run({ op: 'saveProject', name: null })">
          <Save :size="16" />
          {{ t('project.save') }}
        </RlButton>
      </div>
      <label class="field-label">
        {{ t('project.saveAs') }}
        <span class="flex gap-2">
          <input v-model="saveAsName" class="field min-w-0 flex-1" :placeholder="state.project.name">
          <RlButton @press="run({ op: 'saveProject', name: saveAsName || state.project.name })">
            {{ t('project.saveAsButton') }}
          </RlButton>
        </span>
      </label>
      <h3 class="heading">{{ t('project.projects') }}</h3>
      <p v-if="confirmOpen" class="text-sm text-flash">{{ t('project.discardOpen', { name: confirmOpen }) }}</p>
      <!-- Sem item marcado: assim o segundo toque no mesmo projeto também chega -->
      <RlList class="max-h-72" :model-value="undefined" :items="projects" @update:model-value="openProject" />
    </div>
  </RlDialog>
</template>

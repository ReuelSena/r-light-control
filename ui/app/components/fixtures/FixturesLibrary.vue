<script setup lang="ts">
import { FilePlus, Pencil, Plus, Search, Trash2, Upload } from 'lucide-vue-next'
import type { FixtureDef } from '~/types/generated/FixtureDef'
import type { ImportReport } from '~/types/generated/ImportReport'

// Biblioteca de fixtures (E3): busca, detalhes, adicionar ao projeto, importar .qxf, editar e excluir.
const emit = defineEmits<{ added: [] }>()
const { t } = useI18n()
const { state, send } = useEngine()
const search = ref('')
const selectedId = ref<string>()
const selected = computed(() => state.value.library.find((d) => d.id === selectedId.value))
const modeName = ref('')
const mode = computed(() => selected.value?.modes.find((m) => m.name === modeName.value) ?? selected.value?.modes[0])
const items = computed(() =>
  state.value.library
    .filter((d) => `${d.manufacturer} ${d.model}`.toLowerCase().includes(search.value.toLowerCase()))
    .map((d) => ({ id: d.id, label: `${d.manufacturer} ${d.model}`, detail: t(`kind.${d.kind}`) }))
    .sort((a, b) => a.label.localeCompare(b.label)),
)
// Cópia em edição (nova ou existente); só vai para a biblioteca ao salvar
const editing = ref<FixtureDef>()

watch(selectedId, () => {
  modeName.value = selected.value?.modes[0]?.name ?? ''
  confirmRemove.value = false
})

// ---------- Adicionar ao projeto ----------
const adding = ref(false)
const form = reactive({ mode: '', count: 1, name: '', color: tagColors[1]!, universe: '', address: undefined as number | undefined })

function startAdd() {
  Object.assign(form, { mode: mode.value?.name ?? '', count: 1, name: selected.value!.model, universe: state.value.project.universes[0]?.id ?? '', address: undefined })
  adding.value = true
}

async function confirmAdd() {
  await send({ op: 'addFixtures', defId: selected.value!.id, mode: form.mode, name: form.name, color: form.color, count: form.count, universe: form.universe, address: form.address || null })
  adding.value = false
  emit('added')
}

// ---------- Importar .qxf (vários de uma vez) ----------
const fileInput = ref<HTMLInputElement>()
// Um resultado por arquivo: o relatório da conversão ou o motivo da recusa
const reports = ref<{ file: string; report?: ImportReport; error?: string }[]>([])

async function importFiles(e: Event) {
  const input = e.target as HTMLInputElement
  const results: typeof reports.value = []
  for (const file of input.files ?? []) {
    try {
      const report = (await send({ op: 'importQxf', fileName: file.name, content: await file.text() })) as ImportReport
      results.push({ file: file.name, report })
      selectedId.value = report.defId
    } catch (error) {
      // Recusado: os outros arquivos seguem, e o motivo vai para o relatório
      results.push({ file: file.name, error: (error as Error).message })
    }
  }
  reports.value = results
  input.value = ''
}

// ---------- Excluir (o projeto continua com a cópia dele) ----------
const confirmRemove = ref(false)

function remove() {
  if (!confirmRemove.value) return (confirmRemove.value = true)
  send({ op: 'removeFixtureDef', id: selected.value!.id })
  selectedId.value = undefined
}

// ---------- Editor ----------
function newDef(): FixtureDef {
  return {
    id: '', manufacturer: '', model: '', kind: 'colorChanger', channels: [],
    modes: [{ name: t('editor.defaultMode'), channels: [], heads: [] }],
    physical: { widthMm: 0, heightMm: 0, depthMm: 0, weightKg: 0, powerW: 0, lensMinDeg: 25, lensMaxDeg: 25, panMaxDeg: 0, tiltMaxDeg: 0 },
  }
}

async function save(def: FixtureDef) {
  await send({ op: 'saveFixtureDef', def })
  selectedId.value = def.id
  editing.value = undefined
}
</script>

<template>
  <div class="grid h-full gap-3 @3xl:grid-cols-[minmax(240px,1fr)_2fr]">
    <section class="panel min-h-0 @max-3xl:max-h-[45vh]" :class="{ '@max-3xl:hidden': editing }">
      <label class="relative">
        <Search :size="16" class="absolute top-1/2 left-3 -translate-y-1/2 text-fg-dim" />
        <input v-model="search" class="field w-full pl-9" :placeholder="t('library.search')">
      </label>
      <div class="flex flex-wrap gap-2">
        <RlButton size="sm" @press="editing = newDef()">
          <FilePlus :size="16" />
          {{ t('library.new') }}
        </RlButton>
        <RlButton size="sm" @press="fileInput?.click()">
          <Upload :size="16" />
          {{ t('library.import') }}
        </RlButton>
        <input ref="fileInput" type="file" accept=".qxf" multiple class="hidden" @change="importFiles">
      </div>
      <RlList v-model="selectedId" class="min-h-0 flex-1" :items="items" />
    </section>

    <FixturesEditor v-if="editing" :def="editing" @save="save" @cancel="editing = undefined" />

    <section v-else-if="selected" class="panel min-h-0 overflow-auto">
      <header class="flex flex-wrap items-start justify-between gap-2">
        <div>
          <h2 class="text-lg font-bold">{{ selected.model }}</h2>
          <p class="text-sm text-fg-muted">{{ selected.manufacturer }} · {{ t(`kind.${selected.kind}`) }}</p>
        </div>
        <div class="flex flex-wrap gap-2">
          <RlButton size="sm" @press="editing = JSON.parse(JSON.stringify(selected))">
            <Pencil :size="16" />
            {{ t('common.edit') }}
          </RlButton>
          <RlButton size="sm" @press="startAdd">
            <Plus :size="16" />
            {{ t('library.addToProject') }}
          </RlButton>
          <RlButton size="sm" :model-value="confirmRemove" color="var(--color-danger)" @press="remove">
            <Trash2 :size="16" />
            {{ confirmRemove ? t('common.confirmRemove') : t('common.remove') }}
          </RlButton>
        </div>
      </header>
      <label class="field-label">
        {{ t('fixtures.mode') }}
        <select v-model="modeName" class="field">
          <option v-for="m in selected.modes" :key="m.name" :value="m.name">{{ m.name }}</option>
        </select>
      </label>
      <table class="rl-table">
        <thead>
          <tr>
            <th>DMX</th>
            <th>{{ t('editor.channel') }}</th>
            <th>{{ t('editor.attribute') }}</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="(index, position) in mode?.channels" :key="position">
            <td class="font-lcd text-lcd">{{ position + 1 }}</td>
            <td>{{ selected.channels[index]?.name }}</td>
            <td class="text-fg-muted">{{ t(`attribute.${selected.channels[index]?.attribute}`) }}</td>
          </tr>
        </tbody>
      </table>
      <h3 class="heading">{{ t('editor.physical') }}</h3>
      <p class="text-sm text-fg-muted">
        {{ selected.physical.widthMm }} × {{ selected.physical.heightMm }} × {{ selected.physical.depthMm }} mm ·
        {{ selected.physical.weightKg }} kg · {{ selected.physical.powerW }} W ·
        {{ t('library.lens', { min: selected.physical.lensMinDeg, max: selected.physical.lensMaxDeg }) }}
        <template v-if="selected.physical.panMaxDeg">· pan {{ selected.physical.panMaxDeg }}° / tilt {{ selected.physical.tiltMaxDeg }}°</template>
      </p>
    </section>

    <section v-else class="panel justify-center text-center text-fg-muted">{{ t('library.selectHint') }}</section>

    <RlDialog v-model:open="adding" :title="t('library.addToProject')">
      <div class="flex flex-col gap-3">
        <label class="field-label">
          {{ t('fixtures.mode') }}
          <select v-model="form.mode" class="field">
            <option v-for="m in selected?.modes" :key="m.name" :value="m.name">{{ m.name }}</option>
          </select>
        </label>
        <div class="grid grid-cols-2 gap-2">
          <label class="field-label">
            {{ t('library.quantity') }}
            <input v-model.number="form.count" type="number" min="1" max="64" class="field">
          </label>
          <label class="field-label">
            {{ t('fixtures.name') }}
            <input v-model="form.name" class="field">
          </label>
        </div>
        <div class="field-label">
          {{ t('fixtures.color') }}
          <RlSwatches v-model="form.color" :colors="tagColors" />
        </div>
        <div class="grid grid-cols-2 gap-2">
          <label class="field-label">
            {{ t('fixtures.universe') }}
            <select v-model="form.universe" class="field">
              <option v-for="u in state.project.universes" :key="u.id" :value="u.id">{{ u.name }}</option>
            </select>
          </label>
          <label class="field-label">
            {{ t('library.startAddress') }}
            <input v-model.number="form.address" type="number" min="1" max="512" class="field" :placeholder="t('library.automatic')">
          </label>
        </div>
        <RlButton size="lg" @press="confirmAdd">
          <Plus :size="18" />
          {{ t('library.addConfirm', { n: form.count }) }}
        </RlButton>
      </div>
    </RlDialog>

    <RlDialog :open="reports.length > 0" :title="t('library.importReport')" @update:open="reports = []">
      <div class="flex flex-col gap-4">
        <section v-for="r in reports" :key="r.file" class="flex flex-col gap-2">
          <h3 class="heading">{{ r.file }}</h3>
          <p v-if="r.error" class="text-sm text-danger">⚠ {{ r.error }}</p>
          <template v-else-if="r.report">
            <p>{{ t('library.imported', { channels: r.report.channels }, r.report.modes) }}</p>
            <ul v-if="r.report.warnings.length" class="flex flex-col gap-2 text-sm text-flash">
              <li v-for="warning in r.report.warnings" :key="warning">⚠ {{ warning }}</li>
            </ul>
          </template>
        </section>
      </div>
    </RlDialog>
  </div>
</template>

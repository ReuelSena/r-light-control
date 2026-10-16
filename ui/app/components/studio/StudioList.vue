<script setup lang="ts">
import { ChevronDown, ChevronRight, Folder, FolderPlus, Pencil, Play, Square, Trash2 } from 'lucide-vue-next'
import type { Function } from '~/types/generated/Function'

// Lista do Studio: criações agrupadas por tipo e, dentro do tipo, em pastas. Para mover uma criação,
// arraste-a (mouse: clicar e arrastar; toque: segurar e arrastar) até uma pasta do mesmo tipo;
// solta no título do tipo, ela sai da pasta.
const props = defineProps<{ query: string }>()
const selectedId = defineModel<string>()
const { t } = useI18n()
const { state, send } = useEngine()
const project = computed(() => state.value.project)
const collapsed = ref(new Set<string>())
const folderDialog = ref<{ id?: string; kind: FunctionType; name: string }>()
const confirmRemove = ref(false)

const isOpen = (key: string) => !!props.query || !collapsed.value.has(key)
const toggle = (key: string) => (collapsed.value.has(key) ? collapsed.value.delete(key) : collapsed.value.add(key))
const running = (id: string) => state.value.running.includes(id)

// Linhas da lista já na ordem de exibição: tipo, pastas dele (com as criações) e as criações soltas.
// `folder` é a pasta onde soltar ao arrastar até a linha ('' = solta no tipo).
type Row = { key: string; kind: 'type' | 'folder' | 'item'; type: FunctionType; folder: string; label: string; count?: number; fn?: Function }
const rows = computed(() => {
  const list: Row[] = []
  for (const type of functionTypes) {
    const items = project.value.functions.filter((f) => f.type === type && f.name.toLowerCase().includes(props.query.toLowerCase()))
    const folders = project.value.folders.filter((d) => d.kind === type)
    if (!items.length && (props.query || !folders.length)) continue
    list.push({ key: type, kind: 'type', type, folder: '', label: t(`functionTypePlural.${type}`), count: items.length })
    if (!isOpen(type)) continue
    for (const d of folders) {
      const inside = items.filter((f) => f.folder === d.id)
      if (props.query && !inside.length) continue
      list.push({ key: d.id, kind: 'folder', type, folder: d.id, label: d.name, count: inside.length })
      if (isOpen(d.id)) list.push(...inside.map((fn): Row => ({ key: fn.id, kind: 'item', type, folder: d.id, label: fn.name, fn })))
    }
    const loose = items.filter((f) => !folders.some((d) => d.id === f.folder))
    list.push(...loose.map((fn): Row => ({ key: fn.id, kind: 'item', type, folder: '', label: fn.name, fn })))
  }
  return list
})

// Arrastar: no mouse começa ao mover 6 px; no toque, depois de segurar 400 ms (antes disso o gesto rola a lista)
const drag = ref<{ fn: Function; x: number; y: number; active: boolean; moved?: boolean; target?: string }>()
let start = { x: 0, y: 0 }
let holdTimer: ReturnType<typeof setTimeout> | undefined

function down(e: PointerEvent, fn: Function) {
  start = { x: e.clientX, y: e.clientY }
  drag.value = { fn, x: e.clientX, y: e.clientY, active: false }
  ;(e.currentTarget as HTMLElement).setPointerCapture(e.pointerId)
  if (e.pointerType !== 'mouse') holdTimer = setTimeout(() => drag.value && (drag.value.active = true), 400)
}

function move(e: PointerEvent) {
  const d = drag.value
  if (!d) return
  const moved = Math.hypot(e.clientX - start.x, e.clientY - start.y)
  if (!d.active && e.pointerType === 'mouse' && moved > 6) d.active = true
  if (!d.active) {
    if (moved > 8) {
      clearTimeout(holdTimer)
      d.moved = true
    }
    return
  }
  const el = document.elementFromPoint(e.clientX, e.clientY)?.closest('[data-folder]') as HTMLElement | null
  Object.assign(d, { x: e.clientX, y: e.clientY, target: el?.dataset.type === d.fn.type ? el.dataset.folder : undefined })
}

// Soltar sem ter arrastado nem deslizado é um toque: escolhe a criação
function up() {
  clearTimeout(holdTimer)
  const d = drag.value
  drag.value = undefined
  if (!d) return
  if (!d.active) {
    if (!d.moved) selectedId.value = d.fn.id
    return
  }
  if (d.target !== undefined && (d.target || null) !== d.fn.folder) send({ op: 'saveFunction', function: { ...d.fn, folder: d.target || null } })
}

function cancel() {
  clearTimeout(holdTimer)
  drag.value = undefined
}

function saveFolder() {
  const { id, kind, name } = folderDialog.value!
  send({ op: 'saveFolder', folder: { id: id ?? `d${Date.now().toString(36)}`, kind, name: name.trim() || t('studio.newFolder') } })
  folderDialog.value = undefined
}

function removeFolder() {
  if (!confirmRemove.value) return (confirmRemove.value = true)
  send({ op: 'removeFolder', id: folderDialog.value!.id! })
  folderDialog.value = undefined
}

watch(folderDialog, () => (confirmRemove.value = false))
</script>

<template>
  <div class="flex min-h-0 flex-col">
    <!-- Durante o arrasto por toque, o movimento não pode virar rolagem -->
    <ul class="studio-list sunken flex-1" @touchmove="drag?.active && $event.preventDefault()">
      <li
        v-for="row in rows"
        :key="row.key"
        :data-type="row.type"
        :data-folder="row.folder"
        :class="[row.kind, {
          active: row.fn && row.fn.id === selectedId,
          dragging: drag?.active && row.fn === drag.fn,
          target: drag?.active && row.kind !== 'item' && drag.target === row.folder && drag.fn.type === row.type,
        }]"
        @pointerdown="row.fn && down($event, row.fn)"
        @pointermove="move"
        @pointerup="up"
        @pointercancel="cancel"
      >
        <template v-if="row.fn">
          <component :is="functionIcons[row.type]" :size="18" class="shrink-0 text-fg-muted" />
          <span class="min-w-0 flex-1 truncate">{{ row.label }}</span>
          <!-- O botão não deixa o toque nele começar um arrasto -->
          <span @pointerdown.stop>
            <RlButton
              size="sm"
              highlight="led"
              color="var(--color-on)"
              :model-value="running(row.fn.id)"
              :aria-label="running(row.fn.id) ? t('studio.stop') : t('studio.run')"
              @press="send({ op: 'setFunctionRunning', id: row.fn.id, running: !running(row.fn.id) })"
            >
              <component :is="running(row.fn.id) ? Square : Play" :size="14" />
            </RlButton>
          </span>
        </template>
        <template v-else>
          <button type="button" class="toggle" @click="toggle(row.key)">
            <component :is="isOpen(row.key) ? ChevronDown : ChevronRight" :size="16" class="shrink-0 text-fg-dim" />
            <component :is="row.kind === 'type' ? functionIcons[row.type] : Folder" :size="18" class="shrink-0" />
            <span class="truncate">{{ row.label }}</span>
            <small>{{ row.count }}</small>
          </button>
          <RlButton
            v-if="row.kind === 'type'"
            size="sm"
            :aria-label="t('studio.newFolder')"
            :title="t('studio.newFolder')"
            @press="folderDialog = { kind: row.type, name: '' }"
          >
            <FolderPlus :size="16" />
          </RlButton>
          <RlButton
            v-else
            size="sm"
            :aria-label="t('studio.editFolder')"
            @press="folderDialog = { id: row.folder, kind: row.type, name: row.label }"
          >
            <Pencil :size="14" />
          </RlButton>
        </template>
      </li>
    </ul>

    <div v-if="drag?.active" class="ghost" :style="{ left: `${drag.x + 14}px`, top: `${drag.y - 40}px` }">
      <component :is="functionIcons[drag.fn.type]" :size="16" />
      {{ drag.fn.name }}
    </div>

    <RlDialog :open="!!folderDialog" :title="folderDialog?.id ? t('studio.editFolder') : t('studio.newFolder')" @update:open="(o) => o || (folderDialog = undefined)">
      <form v-if="folderDialog" class="grid gap-3" @submit.prevent="saveFolder">
        <label class="field-label">
          {{ t('fixtures.name') }}
          <input v-model="folderDialog.name" class="field" :placeholder="t('studio.newFolder')">
        </label>
        <div class="flex gap-2">
          <RlButton v-if="folderDialog.id" :model-value="confirmRemove" color="var(--color-danger)" @press="removeFolder">
            <Trash2 :size="16" />
            {{ confirmRemove ? t('common.confirmRemove') : t('common.remove') }}
          </RlButton>
          <RlButton class="ml-auto" @press="saveFolder">{{ t('common.save') }}</RlButton>
        </div>
      </form>
    </RlDialog>
  </div>
</template>

<style scoped>
.studio-list {
  min-height: 0;
  overflow: auto;
  border-radius: 6px;
}

li {
  display: flex;
  align-items: center;
  gap: 8px;
  min-height: 48px;
  padding: 0 8px 0 12px;
  border-bottom: 1px solid #000;
  box-shadow: inset 0 1px 0 rgb(255 255 255 / 0.03);
}

li.item {
  padding-left: 28px;
  cursor: pointer;
}

li.item[data-folder]:not([data-folder='']) {
  padding-left: 44px;
}

li.type {
  background: linear-gradient(#1f2226, #181a1d);
}

li.folder {
  padding-left: 24px;
}

li.active {
  background: color-mix(in srgb, var(--color-select) 22%, transparent);
  box-shadow: inset 3px 0 0 var(--color-select);
}

li.dragging {
  opacity: 0.4;
}

li.target {
  background: color-mix(in srgb, var(--color-on) 18%, transparent);
  box-shadow: inset 0 0 0 2px var(--color-on);
}

.toggle {
  display: flex;
  flex: 1;
  align-items: center;
  gap: 8px;
  min-width: 0;
  min-height: 44px;
  font-weight: 600;
  text-align: left;
}

.type .toggle {
  font-size: 13px;
  letter-spacing: 0.06em;
  text-transform: uppercase;
}

.toggle small {
  color: var(--color-fg-muted);
  font-size: 12px;
  font-weight: 400;
}

/* Etiqueta que segue o ponteiro, acima do dedo */
.ghost {
  position: fixed;
  z-index: 70;
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 8px 12px;
  border: 1px solid #000;
  border-radius: 6px;
  background: var(--color-surface-3);
  box-shadow: 0 8px 20px rgb(0 0 0 / 0.6);
  pointer-events: none;
  white-space: nowrap;
}
</style>

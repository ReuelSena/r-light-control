<script setup lang="ts">
import { ChevronLeft, Copy, Eye, Pause, Play, Plus, Search, Square, Trash2 } from 'lucide-vue-next'
import type { Chaser } from '~/types/generated/Chaser'
import type { Function } from '~/types/generated/Function'

// Studio (E12): todas as criações, agrupadas por tipo e em pastas (StudioList), com busca, e o editor
// da criação escolhida; em espaço estreito aparece um de cada vez.
const { t } = useI18n()
const { state, send } = useEngine()
const project = computed(() => state.value.project)
const selectedId = ref<string>()
const query = ref('')
const creating = ref(false)
const askCollection = ref(false)
const confirmRemove = ref(false)

watch(selectedId, () => (confirmRemove.value = false))

const selected = computed(() => project.value.functions.find((f) => f.id === selectedId.value))
const running = (id: string) => state.value.running.includes(id)
const toggleRun = (id: string) => send({ op: 'setFunctionRunning', id, running: !running(id) })
const paused = (id: string) => state.value.paused.includes(id)

// Prévia (o programador, E12): a cena em edição vai para a luz. Desligada ao abrir a cena; este aparelho
// desliga a prévia que ele ligou ao sair do editor ou trocar de criação
const previewing = computed(() => !!selected.value && state.value.preview === selected.value.id)
let mine: string | undefined
function togglePreview() {
  mine = previewing.value ? undefined : selected.value!.id
  send({ op: 'setPreview', function: mine ?? null })
}
function stopPreview() {
  if (mine && state.value.preview === mine) send({ op: 'setPreview', function: null })
  mine = undefined
}
watch(selectedId, stopPreview)
onBeforeUnmount(stopPreview)

// Conteúdo inicial de cada tipo
function blank(type: FunctionType): Function {
  const name = t('studio.newName', { type: t(`functionType.${type}`), n: project.value.functions.filter((f) => f.type === type).length + 1 })
  const base = { id: uuidv7(), name, folder: null, priority: 'normal' } as const
  const timing = { fadeInMs: 0, holdMs: 1000, fadeOutMs: 0 }
  const playback = { runOrder: 'loop', direction: 'forward', timing } as const
  const chaser: Chaser = {
    scene: null, steps: [], runOrder: 'loop', direction: 'forward', beats: false,
    fadeInMode: 'default', holdMode: 'default', fadeOutMode: 'default', keepLast: false,
    common: { delayInMs: 0, ...timing, delayOutMs: 0, curve: 'linear' },
  }
  switch (type) {
    case 'scene':
      return {
        ...base, type, values: [], groups: [], channelGroups: [],
        delayInMs: 0, fadeInMs: 0, delayOutMs: 0, fadeOutMs: 0, curve: 'linear', attributeTimes: [],
      }
    case 'chaser': return { ...base, type, ...chaser }
    case 'sequence': return { ...base, type, ...chaser, scene: project.value.functions.find((f) => f.type === 'scene')?.id ?? null }
    case 'collection': return { ...base, type, functions: [] }
    case 'efx':
      return {
        ...base, type, algorithm: 'circle', width: 127, height: 127, xOffset: 127, yOffset: 127, rotation: 0,
        xFrequency: 2, yFrequency: 3, xPhase: 90, yPhase: 0, path: [], speed: 'linear', speedPoints: null, relative: false, propagation: 'parallel', fixtures: [], ...playback,
      }
    case 'rgbMatrix':
      return { ...base, type, group: project.value.fixtureGroups[0]?.id ?? null, algorithm: { type: 'plain' }, control: 'rgb', colors: ['#ff0000'], ...playback }
  }
}

// Coleção com criações rodando: antes de criar, pergunta se ela já nasce com elas
function choose(type: FunctionType) {
  creating.value = false
  if (type === 'collection' && state.value.running.length) askCollection.value = true
  else create(type)
}

async function create(type: FunctionType, withRunning = false) {
  const fn = blank(type)
  if (fn.type === 'collection' && withRunning) fn.functions = [...state.value.running]
  await send({ op: 'saveFunction', function: fn })
  askCollection.value = false
  selectedId.value = fn.id
}

async function duplicate() {
  const copy = { ...JSON.parse(JSON.stringify(selected.value)), id: uuidv7(), name: t('studio.copyName', { name: selected.value!.name }) }
  await send({ op: 'saveFunction', function: copy })
  selectedId.value = copy.id
}

function remove() {
  if (!confirmRemove.value) return (confirmRemove.value = true)
  send({ op: 'removeFunction', id: selectedId.value! })
  selectedId.value = undefined
}
</script>

<template>
  <div class="grid h-full gap-3 p-2 @3xl:grid-cols-[360px_1fr]">
    <section class="panel min-h-0" :class="{ '@max-3xl:hidden': selected }">
      <div class="flex gap-2">
        <label class="field flex min-w-0 flex-1 items-center gap-2">
          <Search :size="16" class="shrink-0 text-fg-muted" />
          <input v-model="query" class="min-w-0 flex-1 bg-transparent outline-none" :placeholder="t('studio.search')">
        </label>
        <!-- Para todas as criações; o LED fica aceso enquanto alguma está rodando -->
        <RlButton
          color="var(--color-on)"
          :model-value="state.running.length > 0"
          :aria-label="t('studio.stopAll')"
          :title="t('studio.stopAll')"
          @press="send({ op: 'stopAllFunctions' })"
        >
          <Square :size="16" />
        </RlButton>
        <RlButton @press="creating = true">
          <Plus :size="16" />
          {{ t('studio.new') }}
        </RlButton>
      </div>
      <StudioList v-model="selectedId" class="flex-1" :query="query" />
    </section>

    <section v-if="selected" class="panel @container min-h-0 overflow-auto">
      <!-- Fica no topo da área rolável: o nome e os botões (rodar, pausar…) estão sempre à mão. Os valores
           negativos (o respiro do painel) fazem a barra encostar nas bordas, para o que rola não aparecer
           atrás dela -->
      <header class="sticky -top-3 z-10 -mx-3 -mt-3 flex flex-wrap items-center gap-2 bg-[#1b1d21] px-3 pt-3 pb-2 shadow-[0_2px_4px_rgb(0_0_0/0.5)]">
        <RlButton size="sm" class="@3xl:hidden!" :aria-label="t('studio.back')" @press="selectedId = undefined"><ChevronLeft :size="18" /></RlButton>
        <component :is="functionIcons[selected.type]" :size="22" class="shrink-0 text-fg-muted" :title="t(`functionType.${selected.type}`)" />
        <input
          class="field min-w-40 flex-1 text-lg font-bold"
          :value="selected.name"
          @change="send({ op: 'saveFunction', function: { ...selected, name: ($event.target as HTMLInputElement).value } })"
        >
        <RlButton v-if="selected.type === 'scene'" highlight="led" color="var(--color-flash)" :model-value="previewing" @press="togglePreview">
          <Eye :size="16" />
          {{ t('studio.preview') }}
        </RlButton>
        <!-- Rodar/Parar e Pausar: só o ícone, com o botão aceso enquanto vale -->
        <RlButton
          color="var(--color-on)"
          :model-value="running(selected.id)"
          :aria-label="running(selected.id) ? t('studio.stop') : t('studio.run')"
          :title="running(selected.id) ? t('studio.stop') : t('studio.run')"
          @press="toggleRun(selected.id)"
        >
          <component :is="running(selected.id) ? Square : Play" :size="16" />
        </RlButton>
        <RlButton
          v-if="running(selected.id)"
          color="var(--color-flash)"
          :model-value="paused(selected.id)"
          :aria-label="t('studio.pause')"
          :title="t('studio.pause')"
          @press="send({ op: 'setFunctionPaused', id: selected.id, paused: !paused(selected.id) })"
        >
          <Pause :size="16" />
        </RlButton>
        <RlButton :aria-label="t('studio.duplicate')" :title="t('studio.duplicate')" @press="duplicate"><Copy :size="16" /></RlButton>
        <RlButton :model-value="confirmRemove" color="var(--color-danger)" :aria-label="t('common.remove')" @press="remove">
          <Trash2 :size="16" />
          <template v-if="confirmRemove">{{ t('common.confirmRemove') }}</template>
        </RlButton>
      </header>
      <!-- A prioridade vale para todos os tipos: fica logo abaixo da barra do editor -->
      <div class="flex flex-wrap items-center gap-2">
        <span class="flex items-center gap-1 text-xs text-fg-muted">{{ t('studio.priority') }} <RlInfo :text="t('studio.priorityInfo')" /></span>
        <RlSegmented
          size="sm"
          class="w-fit!"
          :model-value="selected.priority"
          :options="(['low', 'normal', 'high'] as const).map((value) => ({ value, label: t(`priority.${value}`) }))"
          @update:model-value="(priority) => send({ op: 'saveFunction', function: { ...selected!, priority } })"
        />
      </div>
      <StudioScene v-if="selected.type === 'scene'" :key="selected.id" :fn="selected" />
      <StudioChaser v-else-if="selected.type === 'chaser' || selected.type === 'sequence'" :key="`c${selected.id}`" :fn="selected" />
      <StudioCollection v-else-if="selected.type === 'collection'" :fn="selected" />
      <StudioEfx v-else-if="selected.type === 'efx'" :fn="selected" />
      <StudioRgbMatrix v-else :fn="selected" />
    </section>
    <section v-else class="panel justify-center text-center text-fg-muted @max-3xl:hidden">{{ t('studio.selectHint') }}</section>

    <RlDialog
      v-model:open="creating"
      :title="t('studio.new')"
      :info="functionTypes.map((type) => `${t(`functionType.${type}`)}: ${t(`functionHint.${type}`)}`).join('\n')"
      sheet
    >
      <div class="grid gap-2">
        <RlButton v-for="type in functionTypes" :key="type" size="lg" class="justify-start!" @press="choose(type)">
          <component :is="functionIcons[type]" :size="24" class="shrink-0" />
          {{ t(`functionType.${type}`) }}
        </RlButton>
      </div>
    </RlDialog>

    <RlDialog v-model:open="askCollection" :title="t('studio.newCollection')">
      <div class="grid gap-4">
        <p>{{ t('studio.collectionFromRunning', state.running.length) }}</p>
        <div class="flex flex-wrap gap-1.5">
          <span v-for="id in state.running" :key="id" class="sunken flex items-center gap-1.5 rounded-md px-2 py-1 text-sm">
            <component :is="functionIcons[project.functions.find((f) => f.id === id)?.type ?? 'scene']" :size="14" class="text-fg-muted" />
            {{ project.functions.find((f) => f.id === id)?.name }}
          </span>
        </div>
        <div class="flex justify-end gap-2">
          <RlButton @press="create('collection')">{{ t('studio.emptyCollection') }}</RlButton>
          <RlButton color="var(--color-on)" highlight="led" :model-value="true" @press="create('collection', true)">{{ t('studio.withRunning') }}</RlButton>
        </div>
      </div>
    </RlDialog>
  </div>
</template>

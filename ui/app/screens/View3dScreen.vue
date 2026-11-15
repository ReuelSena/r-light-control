<script setup lang="ts">
import { Box, Copy, Eye, EyeOff, Lightbulb, Maximize2, Minimize2, Move3d, Plus, Trash2 } from 'lucide-vue-next'
import type { Stage } from '~/types/generated/Stage'
import type { StageElement } from '~/types/generated/StageElement'
import type { StageElementKind } from '~/types/generated/StageElementKind'

// 3D View (E12): prévia ao vivo do palco, com as vistas, o tamanho do palco, a posição de cada fixture
// e os elementos de palco. O toque e o arrasto valem para o que é da aba aberta (fixtures ou elementos).
// Qualidade e fumaça valem só para este aparelho (o celular pode desenhar mais leve).
const { t } = useI18n()
const { state, send, subscribePreview } = useEngine()
const prefs = usePrefs()
const project = computed(() => state.value.project)
const stage = computed(() => project.value.stage)
const view = ref<'perspective' | 'top' | 'front' | 'side'>('perspective')
const editingPosition = ref(false)
// Só a visualização, sem o painel de fixtures, elementos e palco
const only3d = ref(false)
const tab = ref('fixtures')
const layer = computed(() => (tab.value === 'fixtures' || tab.value === 'elements' ? tab.value : undefined))
// Cada aba guarda a sua escolha
// (Shift ou Ctrl + clique na lista, ou no palco, soma à escolha)
const selection = reactive<Record<string, string[]>>({ fixtures: [], elements: [] })
const adding = ref(false)
// Visibilidade, só para arrumar o palco (não vai para o projeto): por tipo e por peça
const hide = reactive({ fixtures: false, elements: false, ids: [] as string[] })
const toggleHidden = (id: string) => (hide.ids = hide.ids.includes(id) ? hide.ids.filter((x) => x !== id) : [...hide.ids, id])

const views = computed(() => (['perspective', 'top', 'front', 'side'] as const).map((value) => ({ value, label: t(`view3d.views.${value}`) })))
const tabs = computed(() => [
  { value: 'fixtures', label: t('screens.fixtures') },
  { value: 'elements', label: t('view3d.elementsTab') },
  { value: 'stage', label: t('view3d.stage') },
])

const saveStage = (patch: Partial<Stage>) => send({ op: 'saveStage', stage: { ...stage.value, ...patch } })

function pick(key: 'fixtures' | 'elements', id: string, add: boolean) {
  const ids = selection[key]!
  selection[key] = !add ? [id] : ids.includes(id) ? ids.filter((x) => x !== id) : [...ids, id]
}

const chosenFixtures = computed(() => selection.fixtures!.flatMap((id) => project.value.fixtures.find((f) => f.id === id) ?? []))
const fixturePlacements = computed(() => chosenFixtures.value.map((f) => stagePosition(project.value, f)))
const fixture = computed(() => (chosenFixtures.value.length === 1 ? chosenFixtures.value[0] : undefined))
const position = computed(() => fixture.value && fixturePlacements.value[0])
// Aplica um ajuste a cada fixture escolhida, na ordem da escolha
function moveFixtures(patches: Partial<Placement>[]) {
  const next = chosenFixtures.value.map((f, i) => ({ ...stagePosition(project.value, f), ...patches[i] }))
  saveStage({ fixtures: [...stage.value.fixtures.filter((p) => !next.some((n) => n.fixture === p.fixture)), ...next] })
}

// ---------- Elementos de palco ----------
const chosenElements = computed(() => selection.elements!.flatMap((id) => stage.value.elements.find((e) => e.id === id) ?? []))
const element = computed(() => (chosenElements.value.length === 1 ? chosenElements.value[0] : undefined))
const elementKinds = Object.keys(stageElementKinds) as StageElementKind[]
const elementName = (e: StageElement) => t(`view3d.elements.${e.kind}`)

function saveElements(elements: StageElement[], select: string[] = []) {
  saveStage({ elements })
  selection.elements = select
}

// Entra no centro do palco, ou ao lado do que já está lá
function addElement(kind: StageElementKind) {
  let x = 0
  while (stage.value.elements.some((e) => Math.abs(e.x - x) < 0.05 && Math.abs(e.z) < 0.05)) x += stageElementKinds[kind].width
  const created = { id: uuidv7(), kind, x, y: 0, z: 0, rotationX: 0, rotationY: 0, rotationZ: 0 }
  saveElements([...stage.value.elements, created], [created.id])
  adding.value = false
}

function moveElements(patches: Partial<Placement>[]) {
  const byId = new Map(chosenElements.value.map((e, i) => [e.id, patches[i]]))
  saveStage({ elements: stage.value.elements.map((e) => ({ ...e, ...byId.get(e.id) })) })
}

// A cópia de vários anda como a do primeiro: o conjunto inteiro um passo ao lado
function duplicate() {
  const [first] = chosenElements.value
  const beside = besideOf(first!)
  const copies = chosenElements.value.map((e) => ({ ...e, id: uuidv7(), x: e.x + beside.x - first!.x, y: Math.max(0, e.y + beside.y - first!.y), z: e.z + beside.z - first!.z }))
  saveElements([...stage.value.elements, ...copies], copies.map((c) => c.id))
}

const removeElement = () => saveElements(stage.value.elements.filter((e) => !selection.elements!.includes(e.id)))

// Com um elemento escolhido na aba Elementos: Ctrl + D duplica ao lado e Del exclui
function onKey(e: KeyboardEvent) {
  if (tab.value !== 'elements' || !chosenElements.value.length || (e.target as HTMLElement).closest('input, textarea, select')) return
  if (e.ctrlKey && e.code === 'KeyD') {
    e.preventDefault()
    if (!e.repeat) duplicate()
  } else if (e.key === 'Delete') {
    e.preventDefault()
    removeElement()
  }
}
onMounted(() => window.addEventListener('keydown', onKey))
onBeforeUnmount(() => window.removeEventListener('keydown', onKey))

// Modo Preparar (por aparelho): o servidor só manda os quadros do preview enquanto esta tela os pede.
// Na janela externa o estado é próprio, então o botão "Preparar" daqui liga o preview dela.
const { prepare, setPrepare } = useConsole()
watch(prepare, (on) => subscribePreview(on), { immediate: true })
onBeforeUnmount(() => subscribePreview(false))
</script>

<template>
  <div
    class="grid h-full gap-2 p-2"
    :class="only3d ? 'grid-rows-1' : 'grid-rows-[minmax(260px,1fr)_auto] @5xl:grid-cols-[1fr_430px] @5xl:grid-rows-1'"
  >
    <div class="relative min-h-0">
      <!-- Preparar ligado: o real e o preview lado a lado (empilhados no celular), ou só o preview. O preview não se edita -->
      <div class="absolute inset-0 flex flex-col gap-2 @3xl:flex-row">
        <div v-if="!prepare || prefs.previewView === 'split'" class="relative min-h-0 min-w-0 flex-1">
          <View3dStage :hidden="hide" :selected="selection[tab]" :view="view" :editing="editingPosition" :layer="layer" class="absolute! inset-0" @update:selected="selection[tab] = $event" />
          <span v-if="prepare" class="tag" style="--tag: var(--color-danger)">{{ t('view3d.live') }}</span>
        </div>
        <div v-if="prepare" class="relative min-h-0 min-w-0 flex-1">
          <View3dStage source="preview" :hidden="hide" :selected="selection[tab]" :view="view" :editing="false" class="absolute! inset-0" />
          <span class="tag" style="--tag: #00ffff">{{ t('view3d.preview') }}</span>
        </div>
      </div>
      <div class="pointer-events-none absolute inset-x-2 top-2 flex flex-wrap items-start gap-2">
        <RlSegmented v-model="view" size="sm" class="sunken pointer-events-auto" :options="views" />
        <div class="pointer-events-auto flex items-center gap-2">
          <RlButton v-model="editingPosition" mode="toggle" highlight="led" size="sm">
            <Move3d :size="16" />
            {{ t('view3d.editPosition') }}
          </RlButton>
          <RlInfo :text="t('view3d.info')" />
          <RlButton :model-value="!hide.fixtures" mode="toggle" highlight="led" size="sm" :title="t('view3d.showFixtures')" @update:model-value="hide.fixtures = !$event">
            <Lightbulb :size="16" />
            <!-- {{ t('screens.fixtures') }} -->
          </RlButton>
          <RlButton :model-value="!hide.elements" mode="toggle" highlight="led" size="sm" :title="t('view3d.showElements')" @update:model-value="hide.elements = !$event">
            <Box :size="16" />
            <!-- {{ t('view3d.elementsTab') }} -->
          </RlButton>
        </div>
        <div class="pointer-events-auto ml-auto flex items-center gap-2">
          <RlSegmented
            v-if="prepare"
            v-model="prefs.previewView"
            size="sm"
            class="sunken"
            :options="(['split', 'preview'] as const).map((value) => ({ value, label: t(`view3d.previewView.${value}`) }))"
          />
          <RlButton :model-value="prepare" mode="toggle" highlight="led" size="sm" :title="t('view3d.prepare')" @update:model-value="setPrepare">
            <Eye :size="16" />
            <!-- {{ t('view3d.prepare') }} -->
          </RlButton>
          <RlButton v-model="only3d" mode="toggle" size="sm" :title="t('view3d.only3d')" :aria-label="t('view3d.only3d')">
            <component :is="only3d ? Minimize2 : Maximize2" :size="16" />
          </RlButton>
        </div>
      </div>
    </div>

    <section v-if="!only3d" class="panel min-h-0 overflow-y-auto @max-5xl:h-[45vh]">
      <RlTabs v-model="tab" :tabs="tabs">
        <template #actions>
          <RlButton v-if="tab === 'elements'" size="sm" :title="t('view3d.addElement')" :aria-label="t('view3d.addElement')" @press="adding = true"><Plus :size="16" /></RlButton>
        </template>
        <template #fixtures>
          <div class="flex flex-col gap-3">
            <div class="sunken max-h-64 overflow-y-auto rounded-md">
              <div v-for="f in project.fixtures" :key="f.id" class="row" :class="{ active: selection.fixtures!.includes(f.id) }">
                <button type="button" class="eye" :class="{ off: hide.ids.includes(f.id) }" :title="t('view3d.toggleVisible')" :aria-label="t('view3d.toggleVisible')" @click="toggleHidden(f.id)">
                  <component :is="hide.ids.includes(f.id) ? EyeOff : Eye" :size="16" />
                </button>
                <button type="button" class="pick" :class="{ off: hide.ids.includes(f.id) }" @click="pick('fixtures', f.id, $event.shiftKey || $event.ctrlKey)">
                  <span class="h-3 w-3 shrink-0 rounded-sm" :style="{ background: f.color }" />
                  <span class="flex-1 truncate">{{ f.name }}</span>
                  <span class="text-xs text-fg-muted">{{ t(`kind.${findDef(project, f.defId)?.kind ?? 'other'}`) }}</span>
                </button>
              </div>
            </div>
            <View3dPlacement v-if="fixture && position" :value="position" :title="t('view3d.position', { name: fixture.name })" @change="moveFixtures([$event])" />
            <View3dGroup v-else-if="chosenFixtures.length > 1" :items="fixturePlacements" :title="t('view3d.group', { count: chosenFixtures.length })" @change="moveFixtures" />
            <p v-else class="text-sm text-fg-muted">{{ t('view3d.pick') }}</p>
          </div>
        </template>

        <template #elements>
          <div class="flex flex-col gap-3">
            <div v-if="stage.elements.length" class="sunken max-h-64 overflow-y-auto rounded-md">
              <div v-for="e in stage.elements" :key="e.id" class="row" :class="{ active: selection.elements!.includes(e.id) }">
                <button type="button" class="eye" :class="{ off: hide.ids.includes(e.id) }" :title="t('view3d.toggleVisible')" :aria-label="t('view3d.toggleVisible')" @click="toggleHidden(e.id)">
                  <component :is="hide.ids.includes(e.id) ? EyeOff : Eye" :size="16" />
                </button>
                <button type="button" class="pick" :class="{ off: hide.ids.includes(e.id) }" @click="pick('elements', e.id, $event.shiftKey || $event.ctrlKey)">
                  <component :is="stageElementKinds[e.kind].icon" :size="16" class="shrink-0 text-fg-muted" />
                  <span class="flex-1 truncate">{{ elementName(e) }}</span>
                </button>
              </div>
            </div>
            <p v-else class="text-sm text-fg-muted">{{ t('view3d.noElements') }}</p>
            <template v-if="chosenElements.length">
              <View3dPlacement v-if="element" :value="element" :title="t('view3d.position', { name: elementName(element) })" @change="moveElements([$event])" />
              <View3dGroup v-else :items="chosenElements" :title="t('view3d.group', { count: chosenElements.length })" @change="moveElements" />
              <div class="flex gap-2">
                <RlButton size="sm" :title="`${t('common.duplicate')} (Ctrl+D)`" @press="duplicate"><Copy :size="16" />{{ t('common.duplicate') }}</RlButton>
                <RlButton size="sm" color="var(--color-danger)" :title="`${t('common.remove')} (Del)`" @press="removeElement">
                  <Trash2 :size="16" />{{ t('common.remove') }}
                </RlButton>
              </div>
            </template>
            <p v-else-if="stage.elements.length" class="text-sm text-fg-muted">{{ t('view3d.pickElement') }}</p>
          </div>
        </template>

        <template #stage>
          <div class="flex flex-col gap-3">
            <div class="grid grid-cols-3 gap-2">
              <label v-for="key in ['width', 'depth', 'height'] as const" :key="key" class="field-label">
                {{ t(`view3d.size.${key}`) }}
                <input
                  class="field w-full"
                  type="number"
                  min="1"
                  max="50"
                  step="0.5"
                  :value="stage[key]"
                  @change="saveStage({ [key]: Math.min(50, Math.max(1, Number(($event.target as HTMLInputElement).value) || 1)) })"
                >
              </label>
            </div>
            <div class="field-label divider">
              <span class="flex items-center gap-1">{{ t('view3d.quality') }} <RlInfo :text="t('view3d.deviceInfo')" /></span>
              <RlSegmented v-model="prefs.quality" :options="(['low', 'medium', 'high'] as const).map((value) => ({ value, label: t(`view3d.qualities.${value}`) }))" />
            </div>
            <div class="field-label">
              {{ t('view3d.smoke') }}
              <div class="flex items-center gap-3">
                <RlKnob v-model="prefs.smoke" :default="0.6" />
                <RlLcd>{{ Math.round(prefs.smoke * 100) }}%</RlLcd>
              </div>
            </div>
          </div>
        </template>
      </RlTabs>
    </section>

    <RlDialog v-model:open="adding" sheet :title="t('view3d.addElement')">
      <div class="flex flex-col gap-3">
        <section v-for="group in stageElementGroups" :key="group" class="flex flex-col gap-1.5">
          <h3 class="heading">{{ t(`view3d.elementGroups.${group}`) }}</h3>
          <div class="grid grid-cols-2 gap-1.5">
            <RlButton
              v-for="kind in elementKinds.filter((k) => stageElementKinds[k].group === group)"
              :key="kind"
              class="justify-start! text-left! leading-tight! whitespace-normal!"
              @press="addElement(kind)"
            >
              <component :is="stageElementKinds[kind].icon" :size="18" class="shrink-0" />
              {{ t(`view3d.elements.${kind}`) }}
            </RlButton>
          </div>
        </section>
      </div>
    </RlDialog>
  </div>
</template>

<style scoped>
.row {
  display: flex;
  width: 100%;
  min-height: 44px;
  align-items: center;
  padding: 0 12px 0 4px;
  border-bottom: 1px solid rgb(255 255 255 / 0.05);
  font-size: 14px;
  text-align: left;
}

.pick {
  display: flex;
  min-width: 0;
  flex: 1;
  gap: 10px;
  align-items: center;
  align-self: stretch;
  text-align: left;
}

.eye {
  display: grid;
  width: 36px;
  height: 36px;
  flex-shrink: 0;
  place-items: center;
  color: var(--color-fg-muted);
}

.off {
  opacity: 0.45;
}

/* Etiqueta do viewport (AO VIVO / PREVIEW) */
.tag {
  position: absolute;
  bottom: 8px;
  left: 8px;
  padding: 2px 8px;
  border: 1px solid var(--tag);
  border-radius: 4px;
  background: rgb(0 0 0 / 0.6);
  color: var(--tag);
  font-size: 11px;
  font-weight: 700;
  letter-spacing: 0.08em;
  pointer-events: none;
}

.active {
  background: color-mix(in srgb, var(--color-select) 25%, transparent);
}
</style>

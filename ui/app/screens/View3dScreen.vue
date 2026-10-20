<script setup lang="ts">
import { Copy, Maximize2, Minimize2, Move3d, Plus, Trash2 } from 'lucide-vue-next'
import type { Stage } from '~/types/generated/Stage'
import type { StageElement } from '~/types/generated/StageElement'
import type { StageElementKind } from '~/types/generated/StageElementKind'

// 3D View (E12): prévia ao vivo do palco, com as vistas, o tamanho do palco, a posição de cada fixture
// e os elementos de palco. O toque e o arrasto valem para o que é da aba aberta (fixtures ou elementos).
// Qualidade e fumaça valem só para este aparelho (o celular pode desenhar mais leve).
const { t } = useI18n()
const { state, send } = useEngine()
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
const selection = reactive<Record<string, string | undefined>>({})
const adding = ref(false)

const views = computed(() => (['perspective', 'top', 'front', 'side'] as const).map((value) => ({ value, label: t(`view3d.views.${value}`) })))
const tabs = computed(() => [
  { value: 'fixtures', label: t('screens.fixtures') },
  { value: 'elements', label: t('view3d.elementsTab') },
  { value: 'stage', label: t('view3d.stage') },
])

const saveStage = (patch: Partial<Stage>) => send({ op: 'saveStage', stage: { ...stage.value, ...patch } })

const fixture = computed(() => project.value.fixtures.find((f) => f.id === selection.fixtures))
const position = computed(() => fixture.value && stagePosition(project.value, fixture.value))
function moveFixture(patch: Partial<Placement>) {
  const next = { ...position.value!, ...patch }
  saveStage({ fixtures: [...stage.value.fixtures.filter((p) => p.fixture !== next.fixture), next] })
}

// ---------- Elementos de palco ----------
const element = computed(() => stage.value.elements.find((e) => e.id === selection.elements))
const elementKinds = Object.keys(stageElementKinds) as StageElementKind[]
const elementName = (e: StageElement) => t(`view3d.elements.${e.kind}`)

function saveElements(elements: StageElement[], select?: string) {
  saveStage({ elements })
  selection.elements = select
}

// Entra no centro do palco, ou ao lado do que já está lá
function addElement(kind: StageElementKind) {
  let x = 0
  while (stage.value.elements.some((e) => Math.abs(e.x - x) < 0.05 && Math.abs(e.z) < 0.05)) x += stageElementKinds[kind].width
  const created = { id: uuidv7(), kind, x, y: 0, z: 0, rotationX: 0, rotationY: 0, rotationZ: 0 }
  saveElements([...stage.value.elements, created], created.id)
  adding.value = false
}

const moveElement = (patch: Partial<Placement>) => saveElements(stage.value.elements.map((e) => (e.id === element.value!.id ? { ...e, ...patch } : e)), element.value!.id)

function duplicate() {
  const copy = { ...element.value!, id: uuidv7(), ...besideOf(element.value!) }
  saveElements([...stage.value.elements, copy], copy.id)
}

const removeElement = () => saveElements(stage.value.elements.filter((e) => e.id !== element.value!.id))

// Com um elemento escolhido na aba Elementos: Ctrl + D duplica ao lado e Del exclui
function onKey(e: KeyboardEvent) {
  if (tab.value !== 'elements' || !element.value || (e.target as HTMLElement).closest('input, textarea, select')) return
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
</script>

<template>
  <div
    class="grid h-full gap-2 p-2"
    :class="only3d ? 'grid-rows-1' : 'grid-rows-[minmax(260px,1fr)_auto] @5xl:grid-cols-[1fr_380px] @5xl:grid-rows-1'"
  >
    <div class="relative min-h-0">
      <View3dStage v-model:selected="selection[tab]" :view="view" :editing="editingPosition" :layer="layer" class="absolute! inset-0" />
      <div class="pointer-events-none absolute inset-x-2 top-2 flex flex-wrap items-start gap-2">
        <RlSegmented v-model="view" size="sm" class="sunken pointer-events-auto" :options="views" />
        <div class="pointer-events-auto flex items-center gap-2">
          <RlButton v-model="editingPosition" mode="toggle" highlight="led" size="sm">
            <Move3d :size="16" />
            {{ t('view3d.editPosition') }}
          </RlButton>
          <RlInfo :text="t('view3d.info')" />
        </div>
        <RlButton v-model="only3d" mode="toggle" highlight="led" size="sm" class="pointer-events-auto ml-auto" :title="t('view3d.only3d')" :aria-label="t('view3d.only3d')">
          <component :is="only3d ? Minimize2 : Maximize2" :size="16" />
          {{ t('view3d.only3d') }}
        </RlButton>
      </div>
    </div>

    <section v-if="!only3d" class="panel min-h-0 overflow-y-auto @max-5xl:max-h-[45vh]">
      <RlTabs v-model="tab" :tabs="tabs">
        <template #fixtures>
          <div class="flex flex-col gap-3">
            <div class="sunken max-h-64 overflow-y-auto rounded-md">
              <button
                v-for="f in project.fixtures"
                :key="f.id"
                type="button"
                class="row"
                :class="{ active: f.id === selection.fixtures }"
                @click="selection.fixtures = f.id"
              >
                <span class="h-3 w-3 shrink-0 rounded-sm" :style="{ background: f.color }" />
                <span class="flex-1 truncate">{{ f.name }}</span>
                <span class="text-xs text-fg-muted">{{ t(`kind.${findDef(project, f.defId)?.kind ?? 'other'}`) }}</span>
              </button>
            </div>
            <View3dPlacement v-if="fixture && position" :value="position" :title="t('view3d.position', { name: fixture.name })" @change="moveFixture" />
            <p v-else class="text-sm text-fg-muted">{{ t('view3d.pick') }}</p>
          </div>
        </template>

        <template #elements>
          <div class="flex flex-col gap-3">
            <RlButton size="sm" class="self-start" @press="adding = true"><Plus :size="16" />{{ t('view3d.addElement') }}</RlButton>
            <div v-if="stage.elements.length" class="sunken max-h-64 overflow-y-auto rounded-md">
              <button
                v-for="e in stage.elements"
                :key="e.id"
                type="button"
                class="row"
                :class="{ active: e.id === selection.elements }"
                @click="selection.elements = e.id"
              >
                <component :is="stageElementKinds[e.kind].icon" :size="16" class="shrink-0 text-fg-muted" />
                <span class="flex-1 truncate">{{ elementName(e) }}</span>
              </button>
            </div>
            <p v-else class="text-sm text-fg-muted">{{ t('view3d.noElements') }}</p>
            <template v-if="element">
              <View3dPlacement :value="element" :title="t('view3d.position', { name: elementName(element) })" @change="moveElement" />
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
  gap: 10px;
  width: 100%;
  min-height: 44px;
  align-items: center;
  padding: 0 12px;
  border-bottom: 1px solid rgb(255 255 255 / 0.05);
  font-size: 14px;
  text-align: left;
}

.active {
  background: color-mix(in srgb, var(--color-select) 25%, transparent);
}
</style>

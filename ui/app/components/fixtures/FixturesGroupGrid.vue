<script setup lang="ts">
import { Eraser, Trash2, Wand2 } from 'lucide-vue-next'
import type { FixtureGroup } from '~/types/generated/FixtureGroup'

// Grupo de fixtures: grade 2D de cabeças, usada pelo EFX (a ordem das fixtures) e pelo RGB Matrix
// (F9). Escolha uma cabeça na paleta e toque nas células; tocar numa célula sem cabeça escolhida a
// esvazia. Cada cabeça entra uma vez só: pintar uma que já está no grupo a muda de lugar.
const props = defineProps<{ group: FixtureGroup }>()
const { t } = useI18n()
const { state, send } = useEngine()
const project = computed(() => state.value.project)
const brush = ref<{ fixture: string; head: number }>()
const confirmRemove = ref(false)

watch(() => props.group.id, () => (confirmRemove.value = false))

// Uma cabeça por fixture, ou uma por pixel em barras de LED
const heads = computed(() =>
  project.value.fixtures.flatMap((f) => {
    const count = Math.max(1, findDef(project.value, f.defId)?.modes.find((m) => m.name === f.mode)?.heads.length ?? 0)
    return Array.from({ length: count }, (_, head) => ({ fixture: f.id, head, color: f.color, label: count > 1 ? `${f.name} · ${head + 1}` : f.name }))
  }),
)
const headAt = (x: number, y: number) => {
  const cell = props.group.cells.find((c) => c.x === x && c.y === y)
  return cell && heads.value.find((h) => h.fixture === cell.fixture && h.head === cell.head)
}
const isBrush = (h: { fixture: string; head: number }) => brush.value?.fixture === h.fixture && brush.value.head === h.head

function save(changes: Partial<FixtureGroup>) {
  send({ op: 'saveFixtureGroup', group: { ...props.group, ...changes } })
}

function tap(x: number, y: number) {
  const cells = props.group.cells.filter((c) => (c.x !== x || c.y !== y) && !(brush.value && isBrush(c)))
  if (brush.value) cells.push({ x, y, ...brush.value })
  save({ cells })
}

// Preenche linha a linha com as cabeças na ordem da paleta
function fillInOrder() {
  const { width, height } = props.group
  save({ cells: heads.value.slice(0, width * height).map((h, i) => ({ x: i % width, y: Math.floor(i / width), fixture: h.fixture, head: h.head })) })
}

function resize(width: number, height: number) {
  save({ width, height, cells: props.group.cells.filter((c) => c.x < width && c.y < height) })
}

function remove() {
  if (!confirmRemove.value) return (confirmRemove.value = true)
  send({ op: 'removeFixtureGroup', id: props.group.id })
}
</script>

<template>
  <section class="panel min-h-0 overflow-auto">
    <div class="grid gap-2 @xl:grid-cols-[1fr_100px_100px] @xl:items-end">
      <label class="field-label">
        {{ t('fixtures.name') }}
        <input class="field" :value="group.name" @change="save({ name: ($event.target as HTMLInputElement).value })">
      </label>
      <label class="field-label">
        {{ t('groups.width') }}
        <input type="number" min="1" max="64" class="field" :value="group.width" @change="resize(Number(($event.target as HTMLInputElement).value), group.height)">
      </label>
      <label class="field-label">
        {{ t('groups.height') }}
        <input type="number" min="1" max="64" class="field" :value="group.height" @change="resize(group.width, Number(($event.target as HTMLInputElement).value))">
      </label>
    </div>

    <div class="grid shrink-0 gap-1 overflow-x-auto" :style="{ gridTemplateColumns: `repeat(${group.width}, minmax(52px, 80px))` }">
      <template v-for="y in group.height" :key="y">
        <button
          v-for="x in group.width"
          :key="x"
          type="button"
          class="cell"
          :style="headAt(x - 1, y - 1) ? { '--tag': headAt(x - 1, y - 1)!.color } : undefined"
          :class="{ filled: headAt(x - 1, y - 1) }"
          @click="tap(x - 1, y - 1)"
        >
          {{ headAt(x - 1, y - 1)?.label }}
        </button>
      </template>
    </div>

    <div class="flex flex-wrap gap-2">
      <RlButton size="sm" @press="fillInOrder">
        <Wand2 :size="16" />
        {{ t('groups.fillInOrder') }}
      </RlButton>
      <RlButton size="sm" :model-value="!brush" highlight="led" @press="brush = undefined">
        <Eraser :size="16" />
        {{ t('groups.eraser') }}
      </RlButton>
      <RlButton size="sm" @press="save({ cells: [] })">{{ t('groups.clear') }}</RlButton>
      <RlButton size="sm" class="ml-auto" :model-value="confirmRemove" color="var(--color-danger)" @press="remove">
        <Trash2 :size="16" />
        {{ confirmRemove ? t('common.confirmRemove') : t('common.remove') }}
      </RlButton>
    </div>

    <h3 class="heading flex items-center gap-2">
      {{ t('groups.palette') }}
      <RlInfo :text="t('groups.paletteInfo')" />
    </h3>
    <div class="flex flex-wrap gap-1.5">
      <RlButton
        v-for="h in heads"
        :key="`${h.fixture}-${h.head}`"
        size="sm"
        highlight="led"
        :color="h.color"
        :model-value="isBrush(h)"
        @press="brush = { fixture: h.fixture, head: h.head }"
      >
        {{ h.label }}
      </RlButton>
    </div>
  </section>
</template>

<style scoped>
.cell {
  aspect-ratio: 1;
  min-height: 52px;
  padding: 2px;
  overflow: hidden;
  border: 1px solid #000;
  border-radius: 4px;
  background: #08090a;
  box-shadow: inset 0 2px 4px rgb(0 0 0 / 0.8);
  color: var(--color-fg-muted);
  font-size: 10px;
  line-height: 1.1;
  word-break: break-word;
}

.cell.filled {
  background: color-mix(in srgb, var(--tag) 30%, #08090a);
  box-shadow: inset 0 0 0 1px var(--tag);
  color: var(--color-fg);
}
</style>

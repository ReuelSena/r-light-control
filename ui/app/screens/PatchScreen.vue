<script setup lang="ts">
import { Plus, TriangleAlert, Wand2 } from 'lucide-vue-next'
import type { Fixture } from '~/types/generated/Fixture'

// Patch (E12): endereços DMX das fixtures em cada universo. Canais disputados ficam em vermelho.
const { t } = useI18n()
const { state, send } = useEngine()
const project = computed(() => state.value.project)
const universeId = ref(project.value.universes[0]?.id ?? '')
const selectedId = ref<string>()
const start = ref(1)
const clashing = computed(() => conflicts(project.value))
const fixtures = computed(() => project.value.fixtures.filter((f) => f.universe === universeId.value).sort((a, b) => a.address - b.address))

// Dono e quantidade de fixtures em cada canal (posições 1–512)
const channels = computed(() => {
  const owner: (Fixture | undefined)[] = []
  const count: number[] = []
  for (const f of fixtures.value) {
    for (let c = f.address; c < f.address + channelCount(project.value, f) && c <= 512; c++) {
      owner[c] = f
      count[c] = (count[c] ?? 0) + 1
    }
  }
  return { owner, count }
})
const used = computed(() => channels.value.count.filter(Boolean).length)

function setAddress(fixture: Fixture, address: number) {
  send({ op: 'updateFixture', fixture: { ...fixture, address } })
}

// Arrastar na grade muda o endereço. No toque, a fixture precisa estar selecionada (senão o gesto
// rola a grade); com o mouse, qualquer uma. O canal agarrado fica sob o ponteiro, e o destino
// aparece em verde, ou em vermelho se invadir outra fixture.
const drag = ref<{ fixture: Fixture; offset: number; target: number }>()
const cellAt = (e: PointerEvent) => Number((document.elementFromPoint(e.clientX, e.clientY)?.closest('[data-ch]') as HTMLElement | null)?.dataset.ch) || 0
const drop = computed(() => {
  if (!drag.value) return
  const { fixture, target: start } = drag.value
  const end = start + channelCount(project.value, fixture)
  return { start, end, conflict: fixtures.value.some((f) => f.id !== fixture.id && f.address < end && f.address + channelCount(project.value, f) > start) }
})

function down(e: PointerEvent) {
  const c = cellAt(e)
  const owner = channels.value.owner[c]
  const wasSelected = owner && owner.id === selectedId.value
  selectedId.value = owner?.id
  if (!owner || (!wasSelected && e.pointerType !== 'mouse')) return
  ;(e.currentTarget as HTMLElement).setPointerCapture(e.pointerId)
  drag.value = { fixture: owner, offset: c - owner.address, target: owner.address }
}

function move(e: PointerEvent) {
  const c = cellAt(e)
  if (!drag.value || !c) return
  drag.value.target = Math.min(Math.max(c - drag.value.offset, 1), 513 - channelCount(project.value, drag.value.fixture))
}

function release() {
  if (drag.value && drag.value.target !== drag.value.fixture.address) setAddress(drag.value.fixture, drag.value.target)
  drag.value = undefined
}

function autoAddress() {
  send({ op: 'autoAddress', ids: fixtures.value.map((f) => f.id), universe: universeId.value, start: start.value })
}
</script>

<template>
  <RlTabs v-model="universeId" class="h-full p-2" :tabs="project.universes.map((u) => ({ value: u.id, label: u.name }))">
    <template #actions>
      <RlButton size="sm" :aria-label="t('io.addUniverse')" @press="send({ op: 'addUniverse' })"><Plus :size="16" /></RlButton>
    </template>

    <div class="grid h-full gap-3 @5xl:grid-cols-[1fr_360px]">
      <section class="panel @container min-h-0 overflow-auto">
        <div class="flex flex-wrap items-baseline justify-between gap-x-3">
          <h2 class="heading">{{ t('patch.grid') }}</h2>
          <p class="text-sm text-fg-muted">
            {{ t('patch.summary', { used, fixtures: fixtures.length }) }}
            <span v-if="fixtures.some((f) => clashing.has(f.id))" class="text-danger">· {{ t('patch.hasConflicts') }}</span>
          </p>
        </div>
        <div
          class="grid-512"
          :class="{ dragging: drag, bad: drop?.conflict }"
          @pointerdown="down"
          @pointermove="move"
          @pointerup="release"
          @pointercancel="drag = undefined"
        >
          <button
            v-for="c in 512"
            :key="c"
            type="button"
            class="cell"
            :data-ch="c"
            :class="{
              used: channels.owner[c],
              first: channels.owner[c]?.address === c,
              conflict: (channels.count[c] ?? 0) > 1,
              selected: channels.owner[c] && channels.owner[c]!.id === selectedId,
              drop: drop && c >= drop.start && c < drop.end,
            }"
            :style="channels.owner[c] ? { '--tag': channels.owner[c]!.color } : undefined"
            :title="channels.owner[c] ? `${c} · ${channels.owner[c]!.name}` : String(c)"
          >
            {{ drop?.start === c ? drag!.fixture.name : channels.owner[c]?.address === c ? channels.owner[c]!.name : c }}
          </button>
        </div>
      </section>

      <section class="panel min-h-0">
        <h2 class="heading">{{ t('patch.fixtures') }}</h2>
        <RlList v-model="selectedId" class="min-h-0 flex-1" :items="fixtures">
          <template #item="{ item }">
            <span class="w-20 shrink-0 font-lcd text-lcd">{{ item.address }}–{{ item.address + channelCount(project, item) - 1 }}</span>
            <span class="h-6 w-1.5 shrink-0 rounded-sm" :style="{ background: item.color }" />
            <span class="min-w-0 flex-1 truncate font-semibold">{{ item.name }}</span>
            <TriangleAlert v-if="clashing.has(item.id)" :size="16" class="shrink-0 text-danger" />
            <input
              v-if="item.id === selectedId"
              type="number"
              min="1"
              max="512"
              class="field w-20"
              :value="item.address"
              :aria-label="t('fixtures.address')"
              @click.stop
              @change="setAddress(item, Number(($event.target as HTMLInputElement).value))"
            >
          </template>
        </RlList>
        <div class="flex items-end gap-2">
          <label class="field-label">
            {{ t('patch.startAt') }}
            <input v-model.number="start" type="number" min="1" max="512" class="field w-24">
          </label>
          <RlButton class="flex-1" @press="autoAddress">
            <Wand2 :size="16" />
            {{ t('patch.autoAddress') }}
          </RlButton>
        </div>
      </section>
    </div>
  </RlTabs>
</template>

<style scoped>
.grid-512 {
  display: grid;
  grid-template-columns: repeat(16, minmax(0, 1fr));
  gap: 2px;
}

/* 32 colunas quando o painel tem espaço (16 linhas de 32 = 512) */
@container (min-width: 760px) {
  .grid-512 {
    grid-template-columns: repeat(32, minmax(0, 1fr));
  }
}

.cell {
  min-height: 28px;
  overflow: hidden;
  border-radius: 2px;
  background: #08090a;
  color: var(--color-fg-dim);
  font-size: 9px;
  white-space: nowrap;
}

.cell.used {
  background: color-mix(in srgb, var(--tag) 28%, #08090a);
  color: var(--color-fg-muted);
}

.cell.first {
  box-shadow: inset 2px 0 0 var(--tag);
  color: var(--color-fg);
  font-weight: 700;
  text-align: left;
  padding-left: 3px;
}

.cell.conflict {
  background: repeating-linear-gradient(45deg, var(--color-danger) 0 4px, #4a0000 4px 8px);
  color: #fff;
}

/* A fixture selecionada pode ser arrastada: o toque nela não rola a grade */
.cell.selected {
  outline: 2px solid var(--color-select);
  outline-offset: -1px;
  touch-action: none;
  cursor: grab;
}

.dragging,
.dragging .cell {
  cursor: grabbing;
}

/* Destino do arrasto: verde se livre, vermelho se invade outra fixture */
.cell.drop {
  background: color-mix(in srgb, var(--color-on) 35%, #08090a);
  outline: 2px solid var(--color-on);
  outline-offset: -1px;
  color: #fff;
}

.bad .cell.drop {
  background: color-mix(in srgb, var(--color-danger) 45%, #08090a);
  outline-color: var(--color-danger);
}
</style>

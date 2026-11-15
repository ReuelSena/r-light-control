<script setup lang="ts">
import { ChevronDown, ChevronRight, Grid3x3, Plus } from 'lucide-vue-next'
import type { FixtureGroup } from '~/types/generated/FixtureGroup'

// Grupos de fixtures (grade 2D de cabeças, para o RGB Matrix e o EFX; em árvore, com subgrupos) e grupos de
// canais (canais de quaisquer fixtures, para faders do console e da mesa simples e para cenas).
const { t } = useI18n()
const { state, send } = useEngine()
const project = computed(() => state.value.project)
const selectedId = ref(project.value.fixtureGroups[0]?.id)
const fixtureGroup = computed(() => project.value.fixtureGroups.find((g) => g.id === selectedId.value))
const channelGroup = computed(() => project.value.channelGroups.find((g) => g.id === selectedId.value))
const name = () => t('groups.newName', { n: project.value.fixtureGroups.length + project.value.channelGroups.length + 1 })

async function createFixtureGroup() {
  const id = uuidv7()
  await send({ op: 'saveFixtureGroup', group: { id, name: name(), width: 4, height: 2, cells: [], parent: null } })
  selectedId.value = id
}

async function createChannelGroup() {
  const id = uuidv7()
  await send({ op: 'saveChannelGroup', group: { id, name: name(), fixtureGroup: null, attributes: [], channels: [] } })
  selectedId.value = id
}

// Árvore: os filhos ficam logo abaixo do pai; fechar o pai esconde tudo o que há dentro
const collapsed = ref(new Set<string>())
const rows = computed(() => {
  const list: { group: FixtureGroup; depth: number; children: boolean }[] = []
  const tree = groupTree(project.value)
  let hidden = Infinity
  tree.forEach(({ group, depth }, i) => {
    if (depth > hidden) return
    hidden = Infinity
    list.push({ group, depth, children: tree[i + 1]?.depth === depth + 1 })
    if (collapsed.value.has(group.id)) hidden = depth
  })
  return list
})
const toggle = (id: string) => (collapsed.value.has(id) ? collapsed.value.delete(id) : collapsed.value.add(id))
// O grupo `id` é o `ancestor` ou está dentro dele
const isInside = (id: string, ancestor: string): boolean => {
  const g = project.value.fixtureGroups.find((x) => x.id === id)
  return !!g && (g.id === ancestor || isInside(g.parent ?? '', ancestor))
}

// Arrastar (mouse: mover 6 px; toque: segurar 400 ms): o quarto de cima da linha põe o grupo antes dela, o de
// baixo depois e o meio dentro dela (vira filho); soltar na área vazia o leva para a raiz.
type Zone = 'before' | 'into' | 'after'
const drag = ref<{ id: string; active: boolean; target?: string; zone?: Zone }>()
let start = { x: 0, y: 0 }
let holdTimer: ReturnType<typeof setTimeout> | undefined
let grab: { el: HTMLElement; pointer: number } | undefined

function activate() {
  if (!drag.value || !grab) return
  drag.value.active = true
  grab.el.setPointerCapture(grab.pointer)
}

function down(e: PointerEvent, id: string) {
  start = { x: e.clientX, y: e.clientY }
  drag.value = { id, active: false }
  grab = { el: e.currentTarget as HTMLElement, pointer: e.pointerId }
  if (e.pointerType !== 'mouse') holdTimer = setTimeout(activate, 400)
}

function move(e: PointerEvent) {
  const d = drag.value
  if (!d) return
  const moved = Math.hypot(e.clientX - start.x, e.clientY - start.y)
  if (!d.active && e.pointerType === 'mouse' && moved > 6) activate()
  if (!d.active) {
    if (moved > 8) clearTimeout(holdTimer)
    return
  }
  const el = document.elementFromPoint(e.clientX, e.clientY)?.closest<HTMLElement>('[data-group]')
  const box = el?.getBoundingClientRect()
  const at = box ? (e.clientY - box.top) / box.height : 0.5
  d.zone = at < 0.25 ? 'before' : at > 0.75 ? 'after' : 'into'
  // Um grupo não vai para dentro de si mesmo nem dos filhos dele; fora da lista, o destino é a raiz
  d.target = el && isInside(el.dataset.group!, d.id) ? undefined : (el?.dataset.group ?? '')
}

function up() {
  clearTimeout(holdTimer)
  const d = drag.value
  drag.value = undefined
  if (!d) return
  if (!d.active) {
    selectedId.value = d.id
    return
  }
  if (d.target === undefined) return
  const others = project.value.fixtureGroups.filter((g) => g.id !== d.id)
  const target = others.find((g) => g.id === d.target)
  let parent: string | null = null
  // `next`: o grupo que vai ficar depois do arrastado (nenhum: o fim da lista)
  let next: string | undefined
  if (target && d.zone === 'into') parent = target.id
  else if (target) {
    parent = target.parent ?? null
    const tree = groupTree(project.value).filter((r) => r.group.id !== d.id)
    const i = tree.findIndex((r) => r.group.id === target.id)
    let at = d.zone === 'before' ? i : i + 1
    while (d.zone === 'after' && tree[at] && tree[at]!.depth > tree[i]!.depth) at++
    next = tree[at]?.group.id
  }
  send({ op: 'moveFixtureGroup', id: d.id, to: next ? others.findIndex((g) => g.id === next) : others.length, parent })
  if (parent) collapsed.value.delete(parent)
}

function cancel() {
  clearTimeout(holdTimer)
  drag.value = undefined
}
</script>

<template>
  <div class="grid gap-3 @3xl:h-full @3xl:grid-cols-[260px_1fr]">
    <section class="panel min-h-0 overflow-auto">
      <div class="flex items-center justify-between gap-2">
        <h2 class="heading">{{ t('groups.fixtureGroups') }}</h2>
        <RlButton size="sm" :aria-label="t('groups.newFixtureGroup')" @press="createFixtureGroup"><Plus :size="16" /></RlButton>
      </div>
      <!-- Durante o arrasto por toque, o movimento não pode virar rolagem -->
      <ul class="tree sunken" :class="{ root: drag?.active && drag.target === '' }" data-group="" @touchmove="drag?.active && $event.preventDefault()">
        <li
          v-for="{ group: g, depth, children } in rows"
          :key="g.id"
          :data-group="g.id"
          :style="{ paddingLeft: `${8 + depth * 18}px` }"
          :class="{
            active: g.id === selectedId,
            dragging: drag?.active && drag.id === g.id,
            into: drag?.active && drag.target === g.id && drag.zone === 'into',
            before: drag?.active && drag.target === g.id && drag.zone === 'before',
            after: drag?.active && drag.target === g.id && drag.zone === 'after',
          }"
          @pointerdown="down($event, g.id)"
          @pointermove="move"
          @pointerup="up"
          @pointercancel="cancel"
        >
          <button v-if="children" type="button" class="toggle" :aria-label="t('groups.toggle')" @pointerdown.stop @click="toggle(g.id)">
            <component :is="collapsed.has(g.id) ? ChevronRight : ChevronDown" :size="16" />
          </button>
          <span v-else class="toggle" />
          <Grid3x3 :size="18" class="shrink-0 text-fg-muted" />
          <span class="min-w-0 flex-1 truncate">{{ g.name }}</span>
          <small class="text-xs text-fg-muted">{{ g.width }}×{{ g.height }}</small>
        </li>
      </ul>

      <div class="mt-2 flex items-center justify-between gap-2">
        <h2 class="heading">{{ t('groups.channelGroups') }}</h2>
        <RlButton size="sm" :aria-label="t('groups.newChannelGroup')" @press="createChannelGroup"><Plus :size="16" /></RlButton>
      </div>
      <RlList
        v-model="selectedId"
        reorderable
        class="shrink-0"
        :items="project.channelGroups.map((g) => ({ id: g.id, label: g.name, detail: t('groups.channelCount', g.channels.length), look: channelGroupLook(project, g) }))"
        @reorder="(id, to) => send({ op: 'moveChannelGroup', id, to })"
      >
        <template #item="{ item }">
          <component :is="item.look.icon" :size="18" class="shrink-0" :style="{ color: item.look.color }" />
          <span class="flex-1 truncate">{{ item.label }}</span>
          <small class="text-xs text-fg-muted">{{ item.detail }}</small>
        </template>
      </RlList>
    </section>

    <FixturesGroupGrid v-if="fixtureGroup" :group="fixtureGroup" @selected="selectedId = $event" />
    <FixturesChannelGroup v-else-if="channelGroup" :group="channelGroup" @duplicated="selectedId = $event" />
    <section v-else class="panel justify-center text-center text-fg-muted">{{ t('groups.selectHint') }}</section>
  </div>
</template>

<style scoped>
.tree {
  min-height: 48px;
  flex-shrink: 0;
  overflow: auto;
  border-radius: 6px;
}

.tree.root {
  box-shadow: inset 0 0 0 2px var(--color-on);
}

li {
  position: relative;
  display: flex;
  align-items: center;
  gap: 6px;
  min-height: 48px;
  padding-right: 12px;
  border-bottom: 1px solid #000;
  box-shadow: inset 0 1px 0 rgb(255 255 255 / 0.03);
  cursor: pointer;
}

li.active {
  background: color-mix(in srgb, var(--color-select) 22%, transparent);
  box-shadow: inset 3px 0 0 var(--color-select);
}

li.dragging {
  opacity: 0.5;
}

li.into {
  box-shadow: inset 0 0 0 2px var(--color-on);
}

li.before::before,
li.after::after {
  content: '';
  position: absolute;
  right: 0;
  left: 0;
  height: 3px;
  background: var(--color-on);
}

li.before::before {
  top: -2px;
}

li.after::after {
  bottom: -2px;
}

.toggle {
  display: grid;
  width: 20px;
  flex-shrink: 0;
  place-items: center;
}
</style>

<script setup lang="ts">
import { DropdownMenuCheckboxItem, DropdownMenuContent, DropdownMenuItem, DropdownMenuPortal, DropdownMenuRoot, DropdownMenuTrigger } from 'reka-ui'
import { EllipsisVertical, Grid3x3 } from 'lucide-vue-next'
import type { Fixture } from '~/types/generated/Fixture'
import type { FixtureGroup } from '~/types/generated/FixtureGroup'

// Gaveta de seleção de fixtures, igual em todas as telas: abas Grupos (a árvore, com subgrupos) e
// Fixtures, e o menu de três pontos ao lado do fechar. Quem usa diz o que está ligado e o que tocar faz.
// `sameType` (v-model) só aparece no menu quando a tela usa; `clearable`, idem.
const props = defineProps<{
  title: string
  info?: string
  groups: { group: FixtureGroup; depth: number }[]
  fixtures: Fixture[]
  isGroupOn?: (id: string) => boolean
  isFixtureOn?: (id: string) => boolean
  // Fixture que já vale por um grupo escolhido (mostra o ícone de grupo)
  inGroup?: (id: string) => boolean
  clearable?: boolean
}>()
const emit = defineEmits<{ group: [id: string]; fixture: [id: string]; clear: [] }>()
const open = defineModel<boolean>('open', { default: false })
const sameType = defineModel<boolean | undefined>('sameType', { default: undefined })
const { t } = useI18n()
const { state } = useEngine()
const project = computed(() => state.value.project)
const tab = ref<'groups' | 'fixtures'>('groups')
const hasMenu = computed(() => props.clearable || sameType.value !== undefined)
const rows = computed(() =>
  props.fixtures.map((f) => ({
    id: f.id,
    label: f.name,
    color: f.color,
    universe: project.value.universes.find((u) => u.id === f.universe)?.name,
    address: f.address,
    end: f.address + channelCount(project.value, f) - 1,
  })),
)
</script>

<template>
  <RlDialog v-model:open="open" :title="title" :info="info" sheet wide>
    <template v-if="hasMenu" #actions>
      <DropdownMenuRoot>
        <DropdownMenuTrigger as-child>
          <RlButton size="sm" :aria-label="t('picker.menu')"><EllipsisVertical :size="18" /></RlButton>
        </DropdownMenuTrigger>
        <DropdownMenuPortal>
          <DropdownMenuContent align="end" :side-offset="4" class="sunken z-[60] flex min-w-52 flex-col gap-0.5 rounded-lg border border-black bg-surface-2 p-1 shadow-xl">
            <DropdownMenuCheckboxItem
              v-if="sameType !== undefined"
              :model-value="sameType"
              class="cursor-pointer rounded px-3 py-2 text-sm font-semibold uppercase outline-none data-[highlighted]:bg-surface-3"
              :class="sameType ? 'text-on' : ''"
              @update:model-value="sameType = $event === true"
              @select.prevent
            >
              {{ t('picker.sameType') }}
            </DropdownMenuCheckboxItem>
            <DropdownMenuItem
              v-if="clearable"
              class="cursor-pointer rounded px-3 py-2 text-sm font-semibold uppercase outline-none data-[highlighted]:bg-surface-3"
              @select="emit('clear')"
            >
              {{ t('picker.clear') }}
            </DropdownMenuItem>
          </DropdownMenuContent>
        </DropdownMenuPortal>
      </DropdownMenuRoot>
    </template>

    <div class="flex min-h-[50dvh] flex-col gap-3">
      <RlSegmented v-model="tab" :options="[{ value: 'groups', label: t('picker.groups') }, { value: 'fixtures', label: t('picker.fixtures') }]" />

      <div v-if="tab === 'groups'" class="flex flex-col gap-1.5">
        <p v-if="!groups.length" class="text-sm text-fg-muted">{{ t('picker.noGroups') }}</p>
        <RlButton
          v-for="{ group: g, depth } in groups"
          :key="g.id"
          highlight="led"
          class="justify-start pl-9"
          :style="{ marginLeft: `${depth * 16}px` }"
          :model-value="isGroupOn?.(g.id) ?? false"
          @press="emit('group', g.id)"
        >
          <Grid3x3 :size="14" />
          {{ g.name }}
        </RlButton>
      </div>

      <template v-else>
        <p v-if="!rows.length" class="text-sm text-fg-muted">{{ t('picker.noFixtures') }}</p>
        <!-- A lista só serve de botão: tocar na linha inclui ou tira a fixture (o LED mostra se está ligada) -->
        <RlList v-else class="min-h-0" :model-value="undefined" :items="rows" @update:model-value="(id) => id && emit('fixture', id)">
          <template #item="{ item }">
            <span class="rl-led" :style="{ '--led': isFixtureOn?.(item.id) ? 'var(--color-on)' : 'var(--color-fg-dim)', opacity: isFixtureOn?.(item.id) ? 1 : 0.3 }" />
            <span class="h-6 w-1.5 shrink-0 rounded-sm" :style="{ background: item.color }" />
            <span class="min-w-0 flex-1 truncate font-semibold">{{ item.label }}</span>
            <Grid3x3 v-if="inGroup?.(item.id)" :size="14" class="shrink-0 text-fg-muted" />
            <span class="shrink-0 text-xs text-fg-muted">{{ item.universe }}</span>
            <span class="w-16 shrink-0 text-right font-lcd text-sm text-lcd">{{ item.address }}–{{ item.end }}</span>
          </template>
        </RlList>
      </template>
    </div>
  </RlDialog>
</template>

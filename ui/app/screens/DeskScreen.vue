<script setup lang="ts">
import { RotateCcw } from 'lucide-vue-next'
import type { Fixture } from '~/types/generated/Fixture'

// Mesa simples (E12): um fader por canal DMX, os 512 do universo numa faixa só, e um por grupo de
// canais. Em tela larga, como no QLC+, os grupos ficam numa faixa embaixo dos canais; no celular,
// numa aba. A mesa fica por cima das criações rodando (o último a mexer vence); o medidor mostra a saída.
const STRIP = 88 // largura de uma RlChannelStrip
const { t } = useI18n()
const { state, output, send } = useEngine()
const project = computed(() => state.value.project)
const wide = useMediaQuery('(min-width: 1024px)')
const tab = ref(project.value.universes[0]?.id ?? 'groups')
const bank = ref<HTMLElement>()
const view = ref({ left: 0, width: 0 })

const tabs = computed(() => [
  ...project.value.universes.map((u) => ({ value: u.id, label: u.name })),
  ...(wide.value ? [] : [{ value: 'groups', label: t('groups.channelGroups') }]),
])

watch(tab, () => bank.value?.scrollTo(0, 0))
// Ao alargar a tela, a aba de grupos some (eles passam a ficar embaixo)
watch(wide, (isWide) => {
  if (isWide && tab.value === 'groups') tab.value = project.value.universes[0]?.id ?? ''
})

// Só as strips à vista (mais uma folga) são desenhadas: 512 faders de uma vez pesariam no celular
const measure = () => bank.value && (view.value = { left: bank.value.scrollLeft, width: bank.value.clientWidth })
let observer: ResizeObserver | undefined
watch(bank, (el) => el && (observer ??= new ResizeObserver(measure)).observe(el))
onBeforeUnmount(() => observer?.disconnect())
const start = computed(() => Math.max(0, Math.floor(view.value.left / STRIP) - 4))
const end = computed(() => Math.min(512, Math.ceil((view.value.left + (view.value.width || 1600)) / STRIP) + 4))

// Fixture dona de cada endereço do universo
const owners = computed(() => {
  const owner: (Fixture | undefined)[] = []
  for (const f of project.value.fixtures.filter((x) => x.universe === tab.value)) {
    for (let c = f.address; c < f.address + channelCount(project.value, f) && c <= 512; c++) owner[c] = f
  }
  return owner
})
const strips = computed(() =>
  Array.from({ length: end.value - start.value }, (_, i) => {
    const address = start.value + i + 1
    const fixture = owners.value[address]
    return { address, fixture, channel: fixture && modeChannels(project.value, fixture)[address - fixture.address] }
  }),
)
const level = (address: number) => state.value.desk.levels.find((l) => l.universe === tab.value && l.channel === address)?.value ?? 0
const groupLevel = (id: string) => state.value.desk.groups.find((g) => g.group === id)?.value ?? 0

// Ctrl + roda do mouse rola a faixa; sem Ctrl, a roda mexe no fader sob o ponteiro (E9)
function ctrlScroll(e: WheelEvent) {
  if (!e.ctrlKey) return
  e.preventDefault()
  e.stopPropagation()
  ;(e.currentTarget as HTMLElement).scrollLeft += (e.deltaY + e.deltaX) * (e.deltaMode ? 40 : 2)
}
</script>

<template>
  <div class="flex h-full flex-col gap-3 p-2">
    <RlTabs v-model="tab" :class="tab === 'groups' ? 'flex-none' : 'min-h-0 flex-3'" :tabs="tabs">
      <template #actions>
        <RlButton size="sm" @press="send({ op: 'resetDesk', universe: tab === 'groups' ? null : tab })">
          <RotateCcw :size="16" />
          {{ t('desk.reset') }}
        </RlButton>
      </template>

      <div v-if="tab !== 'groups'" ref="bank" class="strips" @scroll="measure" @wheel.capture="ctrlScroll">
        <div class="flex shrink-0" :style="{ width: `${512 * STRIP}px`, paddingLeft: `${start * STRIP}px` }">
          <RlChannelStrip
            v-for="s in strips"
            :key="s.address"
            plain
            :label="s.fixture ? `${s.address} · ${s.fixture.name} · ${s.channel?.name}` : String(s.address)"
            :color="s.fixture?.color"
            :meter="(output[tab]?.[s.address - 1] ?? 0) / 255"
            :model-value="level(s.address) / 255"
            @update:model-value="(v) => send({ op: 'setDeskLevel', universe: tab, channel: s.address, value: Math.round(v * 255) })"
          >
            <template #label>
              <span class="block font-lcd text-sm">{{ s.address }}</span>
              <span class="block truncate text-[10px] font-normal text-fg-muted normal-case">
                <component :is="attributeLook[s.channel.attribute].icon" v-if="s.channel" :size="10" class="inline" />
                {{ s.fixture ? s.fixture.name : '—' }}
              </span>
            </template>
          </RlChannelStrip>
        </div>
      </div>
    </RlTabs>

    <section v-if="wide || tab === 'groups'" class="flex min-h-0 flex-2 flex-col gap-2">
      <div v-if="wide" class="flex items-center justify-between gap-2">
        <h2 class="heading">{{ t('groups.channelGroups') }}</h2>
        <RlButton size="sm" @press="send({ op: 'resetDesk', universe: null })">
          <RotateCcw :size="16" />
          {{ t('desk.reset') }}
        </RlButton>
      </div>
      <div class="strips" @wheel.capture="ctrlScroll">
        <RlChannelStrip
          v-for="g in project.channelGroups"
          :key="g.id"
          plain
          :label="g.name"
          :color="channelGroupLook(project, g).color"
          :model-value="groupLevel(g.id) / 255"
          @update:model-value="(v) => send({ op: 'setDeskGroupLevel', group: g.id, value: Math.round(v * 255) })"
        >
          <template #label>
            <component :is="channelGroupLook(project, g).icon" :size="12" class="inline" />
            {{ g.name }}
          </template>
        </RlChannelStrip>
      </div>
    </section>
  </div>
</template>

<style scoped>
.strips {
  display: flex;
  flex: 1;
  height: 100%;
  min-height: 240px;
  overflow-x: auto;
  border: 1px solid #000;
  border-radius: 6px;
}
</style>

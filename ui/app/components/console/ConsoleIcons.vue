<script setup lang="ts">
import type { Component } from 'vue'

// Escolha do ícone de um botão do console na biblioteca inteira (lucide), com busca pelo nome
// e uma seleção sugerida para shows.
const icon = defineModel<string | null>({ required: true })
const open = defineModel<boolean>('open', { default: false })
const { t } = useI18n()
const { library, load } = useIconLibrary()
const query = ref('')
const MAX = 150

watch(open, (isOpen) => {
  if (!isOpen) return
  load()
  query.value = ''
})

const suggested = [
  'Lightbulb', 'LightbulbOff', 'Sun', 'SunDim', 'Moon', 'Star', 'Sparkles', 'Zap', 'Flame', 'Heart', 'Music', 'Mic', 'Speaker', 'Volume2',
  'Drum', 'Guitar', 'Piano', 'Camera', 'Palette', 'Droplet', 'Wind', 'Cloud', 'CloudFog', 'Snowflake', 'Rainbow', 'Play', 'Pause', 'Square',
  'SkipForward', 'SkipBack', 'Repeat', 'Shuffle', 'Power', 'OctagonX', 'Eye', 'EyeOff', 'Target', 'Crosshair', 'Move', 'RotateCw', 'Orbit',
  'LayoutGrid', 'Layers', 'Film', 'Timer', 'Bell', 'Flag', 'Megaphone', 'PartyPopper', 'Users', 'User', 'Church', 'Circle', 'Triangle',
  'Hexagon', 'ArrowUp', 'ArrowDown', 'ArrowLeft', 'ArrowRight',
]

// Um nome por ícone (a biblioteca também exporta apelidos do mesmo ícone)
const names = computed(() => {
  const seen = new Set<Component>()
  return Object.entries(library.value ?? {}).flatMap(([name, component]) => (seen.has(component) ? [] : (seen.add(component), [name])))
})
const results = computed(() => {
  const q = query.value.trim().toLowerCase().replace(/[\s-]/g, '')
  if (!q) return suggested.filter((name) => library.value?.[name])
  return names.value.filter((name) => name.toLowerCase().includes(q)).slice(0, MAX)
})

function pick(name: string) {
  icon.value = name
  open.value = false
}
</script>

<template>
  <RlDialog v-model:open="open" sheet :title="t('vc.icon')" :info="t('vc.info.icons')">
    <div class="flex flex-col gap-3">
      <input v-model="query" class="field" :placeholder="t('vc.searchIcon')">
      <h3 class="heading">{{ query ? t('vc.iconResults', { n: results.length }) : t('vc.suggested') }}</h3>
      <p v-if="!library" class="text-sm text-fg-muted">{{ t('vc.loadingIcons') }}</p>
      <div class="grid grid-cols-[repeat(auto-fill,minmax(48px,1fr))] gap-1.5">
        <RlButton
          v-for="name in results"
          :key="name"
          :model-value="name === icon"
          highlight="led"
          :title="name"
          :aria-label="name"
          @press="pick(name)"
        >
          <component :is="library![name]" :size="20" />
        </RlButton>
      </div>
      <p v-if="library && query && !results.length" class="text-sm text-fg-muted">{{ t('vc.noIcons') }}</p>
    </div>
  </RlDialog>
</template>

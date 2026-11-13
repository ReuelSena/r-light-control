<script setup lang="ts">
import type { AttributeKind } from '~/types/generated/AttributeKind'

// Update (P5): grava numa cena os ajustes feitos na Mesa simples; as cenas rodando vêm primeiro
const open = defineModel<boolean>('open', { default: false })
const { t } = useI18n()
const { state, send } = useEngine()
const scenes = computed(() =>
  state.value.project.functions
    .filter((f) => f.type === 'scene')
    .sort((a, b) => Number(state.value.running.includes(b.id)) - Number(state.value.running.includes(a.id))),
)
const target = ref<string>()
const kinds = ref<AttributeKind[]>([])
const error = ref('')

watch(open, (isOpen) => {
  if (!isOpen) return
  target.value = scenes.value.find((s) => state.value.running.includes(s.id))?.id
  error.value = ''
})

async function save() {
  if (!target.value) return
  try {
    await send({ op: 'updateScene', function: target.value!, kinds: kinds.value })
    open.value = false
  } catch (e) {
    error.value = (e as Error).message
  }
}
</script>

<template>
  <RlDialog v-model:open="open" :title="t('update.title')" :info="t('update.info')" sheet>
    <div class="grid gap-4">
      <div class="grid gap-2">
        <RlButton
          v-for="s in scenes"
          :key="s.id"
          highlight="led"
          class="justify-start!"
          :model-value="target === s.id"
          @press="target = s.id"
        >
          {{ s.name }}
          <span v-if="state.running.includes(s.id)" class="ml-auto text-xs text-[var(--color-on)]">{{ t('update.running') }}</span>
        </RlButton>
      </div>
      <StudioKindFilter v-model="kinds" />
      <p v-if="error" class="text-danger">{{ error }}</p>
      <div class="flex justify-end">
        <RlButton color="var(--color-on)" @press="save">{{ t('update.save') }}</RlButton>
      </div>
    </div>
  </RlDialog>
</template>

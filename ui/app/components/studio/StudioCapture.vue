<script setup lang="ts">
import type { AttributeKind } from '~/types/generated/AttributeKind'

// Gravar saída (P5): grava numa cena, nova ou existente, o que está saindo na luz agora
const open = defineModel<boolean>('open', { default: false })
const emit = defineEmits<{ created: [id: string] }>()
const { t } = useI18n()
const { state, send } = useEngine()
const scenes = computed(() => state.value.project.functions.filter((f) => f.type === 'scene'))
const target = ref('new')
const name = ref('')
const merge = ref(false)
const kinds = ref<AttributeKind[]>([])
const error = ref('')

// Cada abertura recomeça com um nome padrão
watch(open, (isOpen) => {
  if (!isOpen) return
  name.value = t('capture.defaultName', { n: scenes.value.length + 1 })
  error.value = ''
})

async function save() {
  const isNew = target.value === 'new'
  const id = uuidv7()
  try {
    await send({
      op: 'captureOutput',
      target: isNew ? { type: 'new', id, name: name.value.trim() || t('capture.defaultName', { n: scenes.value.length + 1 }) } : { type: 'scene', id: target.value },
      merge: !isNew && merge.value,
      kinds: kinds.value,
    })
    open.value = false
    if (isNew) emit('created', id)
  } catch (e) {
    error.value = (e as Error).message
  }
}
</script>

<template>
  <RlDialog v-model:open="open" :title="t('capture.title')" :info="t('capture.info')" sheet>
    <div class="grid gap-4">
      <div class="grid gap-2">
        <RlButton highlight="led" class="justify-start!" :model-value="target === 'new'" @press="target = 'new'">{{ t('capture.newScene') }}</RlButton>
        <input v-if="target === 'new'" v-model="name" class="field" :aria-label="t('capture.name')">
        <RlButton
          v-for="s in scenes"
          :key="s.id"
          highlight="led"
          class="justify-start!"
          :model-value="target === s.id"
          @press="target = s.id"
        >
          {{ s.name }}
        </RlButton>
      </div>
      <RlSegmented
        v-if="target !== 'new'"
        size="sm"
        class="w-fit!"
        :model-value="merge ? 'merge' : 'overwrite'"
        :options="[{ value: 'overwrite', label: t('capture.overwrite') }, { value: 'merge', label: t('capture.merge') }]"
        @update:model-value="(v) => (merge = v === 'merge')"
      />
      <StudioKindFilter v-model="kinds" />
      <p v-if="error" class="text-danger">{{ error }}</p>
      <div class="flex justify-end">
        <RlButton color="var(--color-on)" @press="save">{{ t('capture.save') }}</RlButton>
      </div>
    </div>
  </RlDialog>
</template>

<script setup lang="ts">
// Edição em grupo (3D View): a posição move o conjunto (os campos mostram o centro da caixa que o envolve),
// a rotação vale igual para todos, e cada eixo pode ser alinhado ou distribuído. `items` vem na ordem da escolha.
const props = defineProps<{ items: Placement[]; title: string }>()
const emit = defineEmits<{ change: [patches: Partial<Placement>[]] }>()
const { t } = useI18n()

const round = (v: number) => Math.round(v * 1000) / 1000
const values = (key: keyof Placement) => props.items.map((i) => i[key])
const center = (key: keyof Placement) => round((Math.min(...values(key)) + Math.max(...values(key))) / 2)
// A rotação só aparece quando é igual em todos
const common = (key: keyof Placement) => (values(key).every((v) => v === values(key)[0]) ? values(key)[0] : undefined)
const number = (e: Event) => Number((e.target as HTMLInputElement).value)

function setPosition(key: 'x' | 'y' | 'z', e: Event) {
  const value = number(e)
  if (Number.isNaN(value)) return
  // Nada desce abaixo do chão: o conjunto inteiro para onde o mais baixo toca
  const delta = Math.max(value - center(key), key === 'y' ? -Math.min(...values(key)) : -Infinity)
  emit('change', props.items.map((i) => ({ [key]: round(i[key] + delta) })))
}

function setRotation(key: keyof Placement, e: Event) {
  const value = number(e)
  const input = e.target as HTMLInputElement
  if (Number.isNaN(value) || input.value === '') return
  emit('change', props.items.map(() => ({ [key]: value })))
}

const arrange = (key: keyof Placement, mode: Arrange) =>
  emit('change', arrangeValues(values(key), mode, !key.startsWith('rotation')).map((v) => ({ [key]: v })))

const keys = ['x', 'y', 'z', 'rotationX', 'rotationY', 'rotationZ'] as const
</script>

<template>
  <h3 class="heading">{{ title }}</h3>
  <h3 class="heading">{{ t('view3d.groupPosition') }}</h3>
  <div class="grid grid-cols-3 gap-2">
    <label v-for="key in (['x', 'y', 'z'] as const)" :key="key" class="field-label">
      {{ t(`view3d.axis.${key}`) }}
      <input class="field w-full" type="number" step="0.1" :value="center(key)" @change="setPosition(key, $event)">
    </label>
  </div>
  <h3 class="heading">{{ t('view3d.rotation') }}</h3>
  <div class="grid grid-cols-3 gap-2">
    <label v-for="key in (['rotationX', 'rotationY', 'rotationZ'] as const)" :key="key" class="field-label">
      {{ t(`view3d.axis.${key}`) }}
      <input class="field w-full" type="number" step="5" placeholder="—" :value="common(key)" @change="setRotation(key, $event)">
    </label>
  </div>
  <h3 class="heading">{{ t('view3d.arrangeTitle') }}</h3>
  <div class="flex flex-col gap-1.5">
    <div v-for="key in keys" :key="key" class="flex items-center gap-1.5">
      <span class="w-24 shrink-0 text-xs text-fg-muted">{{ key.startsWith('rotation') ? `${t('view3d.rotation')} ` : '' }}{{ t(`view3d.axis.${key}`) }}</span>
      <RlButton v-for="mode in (['min', 'center', 'max', 'distribute'] as const)" :key="mode" size="sm" class="flex-1" @press="arrange(key, mode)">
        {{ t(`view3d.arrange.${mode}`) }}
      </RlButton>
    </div>
  </div>
</template>

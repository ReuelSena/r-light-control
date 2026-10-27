<script setup lang="ts">
import type { Function } from '~/types/generated/Function'
import type { MatrixPreview } from '~/types/generated/MatrixPreview'

// Prévia animada do RGB Matrix: o mapa de cada passo vem do servidor (o mesmo que sai na luz). Fica num
// componente à parte porque o relógio a redesenha a cada passo: no editor, isso repunha nos campos o valor
// salvo enquanto se digitava.
const props = defineProps<{ fn: Extract<Function, { type: 'rgbMatrix' }> }>()
const { t } = useI18n()
const { state, send } = useEngine()
const now = useClock(50)

const group = computed(() => state.value.project.fixtureGroups.find((g) => g.id === props.fn.group))

// O passo da prévia, na ordem e no sentido da criação
const preview = ref<MatrixPreview>({ steps: 1, map: [] })
const previewError = ref<string>()
const index = computed(() => matrixStep(props.fn, preview.value.steps, Math.floor(now.value / Math.max(props.fn.timing.holdMs, 50))))
watch(
  [index, () => props.fn],
  async () => {
    try {
      preview.value = (await send({ op: 'previewMatrix', id: props.fn.id, step: index.value })) as MatrixPreview
      previewError.value = undefined
    } catch (error) {
      // O script falhou ou a criação foi removida (o editor sai logo em seguida)
      previewError.value = (error as Error).message
    }
  },
  { immediate: true },
)
// Quando a criação rodando para por erro, o motivo vem do motor
const failure = computed(() => state.value.failures.find((f) => f.function === props.fn.id)?.message ?? previewError.value)

// Nos canais de um só valor, a cabeça mostra o cinza da cor
const colorAt = (x: number, y: number) => {
  const color = preview.value.map[y]?.[x]
  if (!group.value?.cells.some((c) => c.x === x && c.y === y) || color === undefined) return undefined
  const [r, g, b] = [color >> 16, (color >> 8) & 255, color & 255]
  const grey = Math.round(0.299 * r + 0.587 * g + 0.114 * b)
  return props.fn.control === 'rgb' ? `rgb(${r}, ${g}, ${b})` : `rgb(${grey}, ${grey}, ${grey})`
}
</script>

<template>
  <div class="flex flex-col gap-3">
    <div
      v-if="group"
      class="sunken grid max-w-xl gap-1 rounded-md p-2"
      :style="{ gridTemplateColumns: `repeat(${group.width}, minmax(0, 1fr))` }"
    >
      <template v-for="y in group.height" :key="y">
        <span v-for="x in group.width" :key="x" class="pixel" :style="{ '--px': colorAt(x - 1, y - 1) }" :class="{ empty: !colorAt(x - 1, y - 1) }" />
      </template>
    </div>
    <p v-if="failure" class="text-sm text-danger">⚠ {{ t('matrix.scriptError') }}: {{ failure }}</p>
  </div>
</template>

<style scoped>
.pixel {
  aspect-ratio: 1;
  border-radius: 3px;
  background: var(--px);
  box-shadow: 0 0 10px var(--px), inset 0 1px 1px rgb(255 255 255 / 0.3);
}

.pixel.empty {
  background: #111;
  box-shadow: inset 0 1px 3px rgb(0 0 0 / 0.8);
}
</style>

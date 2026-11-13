<script setup lang="ts">
import type { Attribute } from '~/types/generated/Attribute'
import type { AttributeKind } from '~/types/generated/AttributeKind'
import type { Channel } from '~/types/generated/Channel'

// Ferramenta de atributo de um tipo (intensidade, cor, posição ou efeitos) para os canais de um modo de
// fixture, usada pelo editor de cena. `get` lê o valor (0–255) de um canal e `set` devolve
// os canais mexidos, um a um, com o valor novo.
const props = defineProps<{ channels: Channel[]; kind: AttributeKind; get: (channel: number) => number }>()
const emit = defineEmits<{ set: [values: [channel: number, value: number][]] }>()
const { t } = useI18n()
const { format } = useValueFormat()

const channelsWith = (a: Attribute) => props.channels.flatMap((c, i) => (c.attribute === a && !c.fine ? [i] : []))
// Valor do atributo; undefined se o modo não o tem
const attr = (a: Attribute) => {
  const channel = channelsWith(a)[0]
  return channel === undefined ? undefined : props.get(channel)
}
const present = (list: Attribute[]) => list.filter((a) => attr(a) !== undefined)
const capsOf = (a: Attribute) => props.channels[channelsWith(a)[0]!]!.capabilities
const panel = computed(() => (props.kind === 'color' || props.kind === 'effects' ? panels[props.kind] : undefined))

// Grava por atributo, em todos os canais dele (ex.: o vermelho de cada pixel de uma barra)
function setAttributes(values: Partial<Record<Attribute, number>>) {
  emit('set', Object.entries(values).flatMap(([a, value]) => channelsWith(a as Attribute).map((channel): [number, number] => [channel, value])))
}

const color = computed({
  get: () => `#${(['red', 'green', 'blue'] as const).map((a) => (attr(a) ?? 0).toString(16).padStart(2, '0')).join('')}`,
  set: (hex) => {
    const [red, green, blue] = [1, 3, 5].map((i) => parseInt(hex.slice(i, i + 2), 16))
    setAttributes({ red, green, blue })
  },
})

const position = computed({
  get: () => ({ x: (attr('pan') ?? 0) / 255, y: (attr('tilt') ?? 0) / 255 }),
  set: ({ x, y }) => setAttributes({ pan: x * 255, tilt: y * 255 }),
})
</script>

<template>
  <div v-if="kind === 'intensity'" class="flex h-80 w-24 flex-col gap-2">
    <RlLcd>{{ format((attr('dimmer') ?? 0) / 255) }}</RlLcd>
    <RlFader class="flex-1" :model-value="(attr('dimmer') ?? 0) / 255" @update:model-value="(v) => setAttributes({ dimmer: v * 255 })" />
  </div>

  <RlXYPad v-else-if="kind === 'position'" v-model="position" class="aspect-square w-full max-w-xs" />

  <div v-else-if="panel" class="flex flex-wrap items-start gap-5">
    <RlColorPicker v-if="kind === 'color' && attr('red') !== undefined" v-model="color" class="w-full max-w-xs" />
    <div v-if="present(panel.knobs).length" class="flex flex-wrap gap-4">
      <div v-for="a in present(panel.knobs)" :key="a" class="flex flex-col items-center gap-1">
        <RlKnob :model-value="attr(a)! / 255" :color="attributeLook[a].color" @update:model-value="(v) => setAttributes({ [a]: v * 255 })" />
        <span class="text-xs text-fg-muted">{{ t(`attribute.${a}`) }} · {{ format(attr(a)! / 255) }}</span>
      </div>
    </div>
    <div v-for="a in present(panel.caps)" :key="a" class="grid gap-1.5">
      <h4 class="field-label">{{ t(`attribute.${a}`) }}</h4>
      <div class="flex flex-wrap gap-1.5">
        <RlButton
          v-for="cap in capsOf(a)"
          :key="cap.min"
          size="sm"
          highlight="led"
          :color="cap.color ?? 'var(--color-select)'"
          :model-value="attr(a)! >= cap.min && attr(a)! <= cap.max"
          @press="setAttributes({ [a]: cap.min })"
        >
          {{ cap.name }}
        </RlButton>
      </div>
    </div>
  </div>
</template>

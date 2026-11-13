<script setup lang="ts">
import type { AttributeKind } from '~/types/generated/AttributeKind'

// Filtro dos tipos de atributo ao gravar (P7): nenhum ligado vale como Tudo
const kinds = defineModel<AttributeKind[]>({ required: true })
const { t } = useI18n()
const toggle = (kind: AttributeKind) =>
  (kinds.value = kinds.value.includes(kind) ? kinds.value.filter((k) => k !== kind) : [...kinds.value, kind])
</script>

<template>
  <div class="flex flex-wrap gap-1.5">
    <RlButton size="sm" highlight="led" :model-value="!kinds.length" @press="kinds = []">{{ t('capture.all') }}</RlButton>
    <RlButton v-for="kind in attributeKinds" :key="kind" size="sm" highlight="led" :model-value="kinds.includes(kind)" @press="toggle(kind)">
      {{ t(`scene.${kind}`) }}
    </RlButton>
  </div>
</template>

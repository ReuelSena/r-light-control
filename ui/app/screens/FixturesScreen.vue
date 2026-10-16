<script setup lang="ts">
// Fixtures (E12): equipamentos do projeto, biblioteca (com importação .qxf e editor) e grupos.
const { t } = useI18n()
const tab = ref('project')
</script>

<template>
  <RlTabs
    v-model="tab"
    class="h-full p-2"
    :tabs="[
      { value: 'project', label: t('fixtures.project') },
      { value: 'library', label: t('fixtures.library') },
      { value: 'groups', label: t('fixtures.groups') },
    ]"
  >
    <template #project>
      <FixturesProject @add="tab = 'library'" />
    </template>
    <template #library>
      <FixturesLibrary @added="tab = 'project'" />
    </template>
    <template #groups>
      <FixturesGroups />
    </template>
  </RlTabs>
</template>

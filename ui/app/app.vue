<script setup lang="ts">
// Toque longo não abre menu de contexto (exceto em campos de texto).
onMounted(() => {
  document.addEventListener('contextmenu', (e) => {
    if (!(e.target as HTMLElement).closest('input, textarea')) e.preventDefault()
  })
})
</script>

<template>
  <NuxtPage />
</template>

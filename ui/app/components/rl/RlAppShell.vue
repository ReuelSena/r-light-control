<script setup lang="ts">
import { Menu } from 'lucide-vue-next'

// Moldura no estilo Ui24R: barra superior em metal escovado, área principal, seção master
// e painel lateral azul à direita, barra inferior.
// - `nav`: navegação principal; fica na barra superior e desce para baixo no celular (< 640 px).
//   `nav-bottom` substitui o conteúdo da barra do celular, quando ele precisa ser diferente.
// - `side`: abaixo de 1024 px vira um sheet, aberto pelo botão de menu.
defineProps<{ sideTitle?: string }>()
const { t } = useI18n()
const sideOpen = ref(false)
</script>

<template>
  <div class="rl-shell">
    <header class="topbar brushed">
      <slot name="toolbar" />
      <nav v-if="$slots.nav" class="nav-top">
        <slot name="nav" />
      </nav>
      <slot name="toolbar-end" />
      <RlButton v-if="$slots.side" class="side-toggle" :aria-label="t('rl.panel')" @press="sideOpen = true">
        <Menu :size="20" />
      </RlButton>
    </header>
    <div class="middle">
      <main class="main">
        <slot />
      </main>
      <aside v-if="$slots.master" class="master">
        <slot name="master" />
      </aside>
      <aside v-if="$slots.side" class="side">
        <slot name="side" />
      </aside>
    </div>
    <footer v-if="$slots.bottom" class="bar">
      <slot name="bottom" />
    </footer>
    <nav v-if="$slots['nav-bottom'] || $slots.nav" class="bar nav-bottom">
      <slot name="nav-bottom">
        <slot name="nav" />
      </slot>
    </nav>
    <RlDialog v-if="$slots.side" v-model:open="sideOpen" sheet :title="sideTitle ?? t('rl.panel')">
      <slot name="side" />
    </RlDialog>
  </div>
</template>

<style scoped>
.rl-shell {
  display: flex;
  flex-direction: column;
  height: 100dvh;
  padding-bottom: env(safe-area-inset-bottom);
  background: #0b0c0e;
}

.topbar {
  display: flex;
  flex-shrink: 0;
  align-items: center;
  gap: 6px;
  padding: max(6px, env(safe-area-inset-top)) 10px 6px;
  overflow-x: auto;
}

.nav-top {
  display: flex;
  gap: 6px;
  margin-right: auto;
}

.middle {
  flex: 1;
  display: flex;
  min-height: 0;
}

.main {
  flex: 1;
  min-width: 0;
  overflow: auto;
}

.master {
  display: flex;
  flex-shrink: 0;
  background: linear-gradient(#1b1d21, #141619);
  border-left: 2px solid #000;
}

.side {
  flex-shrink: 0;
  width: 240px;
  padding: 10px;
  overflow: auto;
  background: linear-gradient(var(--color-panel), color-mix(in srgb, var(--color-panel), black 45%));
  border-left: 1px solid #000;
  box-shadow: inset 1px 0 0 rgb(255 255 255 / 0.15);
}

.bar {
  display: flex;
  flex-shrink: 0;
  gap: 4px;
  padding: 6px 10px;
  overflow-x: auto;
  background: #0b0c0e;
  border-top: 1px solid #000;
}

/* Barra do celular: ícone em cima, texto pequeno embaixo */
.nav-bottom :deep(.rl-button) {
  flex: 1;
  flex-direction: column;
  gap: 2px;
  min-height: 52px;
  padding: 4px 2px;
  font-size: 9px;
}

@media (max-width: 639px) {
  .nav-top {
    display: none;
  }
}

@media (min-width: 640px) {
  .nav-bottom {
    display: none;
  }
}

@media (max-width: 1023px) {
  .side {
    display: none;
  }
}

@media (min-width: 1024px) {
  .side-toggle {
    display: none;
  }
}
</style>

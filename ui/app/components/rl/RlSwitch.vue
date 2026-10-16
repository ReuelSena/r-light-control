<script setup lang="ts">
// Interruptor de alavanca (liga/desliga), com o texto ao lado.
const on = defineModel<boolean>({ default: false })
</script>

<template>
  <label class="rl-switch">
    <input v-model="on" type="checkbox" role="switch">
    <span class="lever" />
    <slot />
  </label>
</template>

<style scoped>
.rl-switch {
  display: inline-flex;
  align-items: center;
  gap: 10px;
  min-height: 44px;
  cursor: pointer;
}

input {
  position: absolute;
  opacity: 0;
  pointer-events: none;
}

.lever {
  position: relative;
  flex-shrink: 0;
  width: 48px;
  height: 26px;
  border: 1px solid #000;
  border-radius: 13px;
  background: #08090a;
  box-shadow: inset 0 2px 4px rgb(0 0 0 / 0.8);
  transition: background 0.15s;
}

.lever::after {
  content: '';
  position: absolute;
  top: 2px;
  left: 2px;
  width: 20px;
  height: 20px;
  border-radius: 50%;
  background: linear-gradient(#f1f2f3, #9da2a7);
  box-shadow: 0 2px 3px rgb(0 0 0 / 0.6);
  transition: translate 0.15s;
}

input:checked + .lever {
  background: color-mix(in srgb, var(--color-on) 70%, #000);
}

input:checked + .lever::after {
  translate: 22px 0;
}

input:focus-visible + .lever {
  outline: 2px solid var(--color-select);
  outline-offset: 2px;
}
</style>

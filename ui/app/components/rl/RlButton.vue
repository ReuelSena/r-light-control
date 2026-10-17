<script setup lang="ts">
// Botão físico. Modos: `action` (clique), `toggle` (liga/desliga) e `flash`
// (ligado só enquanto pressionado; solta ao levantar o dedo, cancelar o toque ou perder a captura).
// Destaque quando ativo: `face` (o botão inteiro acende) ou `led` (só o filete acende e o botão fica afundado).
// `press` traz o evento (ex.: `ctrlKey`). Com `@longpress`, segurar 500 ms dispara `longpress` no lugar de `press`.
const props = withDefaults(defineProps<{
  mode?: 'action' | 'toggle' | 'flash'
  highlight?: 'face' | 'led'
  /** Cor do botão aceso e do filete de LED (qualquer cor CSS). */
  color?: string
  size?: 'sm' | 'md' | 'lg'
  // Declarado como prop (e não como evento) para o botão saber se alguém escuta o toque longo
  onLongpress?: () => void
}>(), { mode: 'action', highlight: 'face', color: 'var(--color-select)', size: 'md', onLongpress: undefined })

const active = defineModel<boolean>({ default: false })
const emit = defineEmits<{ press: [event: MouseEvent] }>()
let holdTimer: ReturnType<typeof setTimeout> | undefined
let held = false
let touched = false // o último disparo veio de um toque

function down(e: PointerEvent) {
  if (props.onLongpress) {
    held = false
    holdTimer = setTimeout(() => {
      held = true
      props.onLongpress!()
    }, 500)
  }
  if (props.mode !== 'flash') return
  ;(e.currentTarget as HTMLElement).setPointerCapture(e.pointerId)
  active.value = true
}

function release() {
  if (props.mode === 'flash') active.value = false
}

function activate(e: MouseEvent) {
  if (props.mode === 'toggle') active.value = !active.value
  emit('press', e)
}

// Toggle e ação disparam ao soltar o dedo, e não no `click`: o navegador não gera `click`
// para um toque feito enquanto outro dedo está na tela (ex.: segurando um fader e tocando no FINO).
// Se o gesto virar rolagem, chega `pointercancel` em vez de `pointerup` e nada dispara.
// Rolagem (pointercancel) ou soltar antes de 500 ms cancelam o toque longo
const cancelHold = () => clearTimeout(holdTimer)

function up(e: PointerEvent) {
  cancelHold()
  if (props.mode !== 'flash' && !held) activate(e)
  touched = e.pointerType === 'touch'
}

// Depois do toque, o navegador manda um clique de compatibilidade no mesmo ponto. Se o botão abriu
// algo ali (painel, diálogo), o clique cairia nele; cancelar o `touchend` impede esse clique.
function touchend(e: TouchEvent) {
  if (touched && e.cancelable) e.preventDefault()
  touched = false
}

// O `click` só conta quando vem do teclado (detail 0); o do ponteiro já foi tratado no `pointerup`.
function click(e: MouseEvent) {
  if (e.detail === 0) activate(e)
}

// Espaço/Enter seguram o flash como o dedo.
function key(e: KeyboardEvent, pressed: boolean) {
  if (props.mode !== 'flash' || e.repeat || (e.key !== ' ' && e.key !== 'Enter')) return
  active.value = pressed
}
</script>

<template>
  <button
    type="button"
    class="rl-button raised"
    :class="[mode, size, highlight, { lit: active }]"
    :style="{ '--lit': color }"
    @pointerdown="down"
    @pointerup="up"
    @touchend="touchend"
    @pointercancel="cancelHold"
    @lostpointercapture="release"
    @contextmenu="onLongpress && $event.preventDefault()"
    @click="click"
    @keydown="key($event, true)"
    @keyup="key($event, false)"
  >
    <slot />
  </button>
</template>

<style scoped>
.rl-button {
  position: relative;
  display: inline-flex;
  flex-shrink: 0;
  align-items: center;
  justify-content: center;
  gap: 6px;
  min-width: 44px;
  min-height: 44px;
  padding: 0 12px;
  border-radius: 6px;
  color: var(--color-fg);
  font-size: 12px;
  font-weight: 600;
  letter-spacing: 0.05em;
  text-transform: uppercase;
  white-space: nowrap;
}

/* Flash: o dedo que escorrega não pode virar rolagem e soltar o flash sem querer */
.flash {
  touch-action: none;
}

.rl-button:active,
.lit {
  transform: translateY(1px);
}

.rl-button:active {
  box-shadow: inset 0 2px 4px rgb(0 0 0 / 0.6);
}

.rl-button:focus-visible {
  outline: 2px solid var(--color-select);
  outline-offset: 2px;
}

/* Filete de LED: mostra a cor do botão mesmo apagado */
.toggle::after,
.flash::after,
.led::after {
  content: '';
  position: absolute;
  left: 25%;
  right: 25%;
  bottom: 4px;
  height: 2px;
  border-radius: 1px;
  background: var(--lit);
  /* Apagado bem fraco, para o aceso se destacar */
  opacity: 0.3;
}

/* Destaque "face": o botão inteiro acende */
.face.lit {
  background: linear-gradient(color-mix(in srgb, var(--lit), white 25%), var(--lit));
  color: #111;
  box-shadow: 0 0 14px color-mix(in srgb, var(--lit) 55%, transparent), inset 0 1px 0 rgb(255 255 255 / 0.4);
}

.face.lit::after {
  opacity: 0;
}

/* Destaque "led": o botão fica afundado e só o filete acende */
.led.lit {
  background: linear-gradient(#17191c, #25282c);
  box-shadow: inset 0 2px 5px rgb(0 0 0 / 0.8), 0 1px 0 rgb(255 255 255 / 0.06);
}

.led.lit::after {
  height: 3px;
  opacity: 1;
  box-shadow: 0 0 4px var(--lit), 0 0 10px var(--lit), 0 0 18px color-mix(in srgb, var(--lit) 50%, transparent);
}

.sm {
  min-width: 36px;
  min-height: 36px;
  padding: 0 6px;
  font-size: 10px;
  letter-spacing: 0.02em;
}

.lg {
  min-height: 56px;
  padding: 0 18px;
  font-size: 15px;
}
</style>

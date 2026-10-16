<script setup lang="ts">
import { renderSVG } from 'uqr'

// Página provisória da F0: valida o caminho UI ↔ servidor Rust no desktop e no celular.
const { t } = useI18n()
const { serverPort } = useRuntimeConfig().public

const connected = ref(false)
const ticks = ref(0)
const rate = ref(0)
const lanIp = ref('')

// Em dev a página vem do servidor do Nuxt (porta 3000), mas o WebSocket é sempre o do Rust.
const wsUrl = import.meta.dev ? `ws://${location.hostname}:${serverPort}/ws` : `ws://${location.host}/ws`
const lanUrl = computed(() => lanIp.value && `http://${lanIp.value}:${location.port}`)
const qrSvg = computed(() => lanUrl.value && renderSVG(lanUrl.value, { border: 2 }))

let ws: WebSocket | undefined
let lastTicks = 0
let rateTimer: ReturnType<typeof setInterval> | undefined

function connect() {
  ws = new WebSocket(wsUrl)
  ws.onopen = () => (connected.value = true)
  ws.onclose = () => {
    connected.value = false
    setTimeout(connect, 1000)
  }
  ws.onmessage = (e) => {
    const msg = JSON.parse(e.data)
    if (msg.t === 'hello') lanIp.value = msg.data.lanIp ?? ''
    else if (msg.t === 'evt' && msg.topic === 'tick') ticks.value = msg.data
  }
}

onMounted(() => {
  connect()
  rateTimer = setInterval(() => {
    rate.value = ticks.value - lastTicks
    lastTicks = ticks.value
  }, 1000)
})

onUnmounted(() => {
  clearInterval(rateTimer)
  if (ws) {
    ws.onclose = null
    ws.close()
  }
})
</script>

<template>
  <main>
    <p :class="['status', { on: connected }]">
      {{ connected ? t('status.connected') : t('status.disconnected') }}
    </p>
    <h1>{{ t('home.ticks') }}</h1>
    <p class="ticks">{{ ticks }}</p>
    <p>{{ t('home.rate', { rate }) }}</p>
    <section v-if="qrSvg">
      <p>{{ t('home.phone') }}</p>
      <!-- eslint-disable-next-line vue/no-v-html -- SVG gerado localmente pelo uqr -->
      <div class="qr" v-html="qrSvg" />
      <code>{{ lanUrl }}</code>
    </section>
  </main>
</template>

<style scoped>
main {
  display: grid;
  justify-items: center;
  gap: 0.5rem;
  padding: 2rem 1rem;
  text-align: center;
}
.status { color: var(--color-danger); }
.status.on { color: var(--color-on); }
.ticks {
  margin: 0;
  font-size: 4rem;
  font-variant-numeric: tabular-nums;
}
.qr {
  width: 200px;
  background: #fff;
}
</style>

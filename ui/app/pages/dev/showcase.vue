<script setup lang="ts">
// Catálogo do design system (F1): todos os componentes, em todos os estados, dentro do shell.
// Os dados são fictícios.
const { t } = useI18n()
const { mode, format } = useValueFormat()
const dmx = computed({ get: () => mode.value === 'dmx', set: (on) => (mode.value = on ? 'dmx' : 'percent') })

const nav = ['console', 'programmer', 'functions', 'patch', 'visualizer', 'devices']
const page = ref('console')
const tab = ref('strips')
const sideTab = ref('banks')
const bank = ref(1)
const viewPage = ref(1)

const strips = ref([
  { label: 'PAR 1', color: '#ff0000', value: 0.75 },
  { label: 'PAR 2', color: '#00ff00', value: 0.5 },
  { label: 'PAR 3', color: '#0000ff', value: 1 },
  { label: 'PAR 4', color: '#ffbf00', value: 0.3 },
  { label: 'MH 1', color: '#00bfff', value: 0.6 },
  { label: 'MH 2', color: '#00bfff', value: 0.6 },
  { label: 'MH 3', color: '#00bfff', value: 0 },
  { label: 'MH 4', color: '#00bfff', value: 0 },
  { label: 'LED BAR 1', color: '#8000ff', value: 0.85 },
  { label: 'LED BAR 2', color: '#8000ff', value: 0.85 },
  { label: 'STROBE', color: '#e6e8eb', value: 0 },
  { label: 'HAZER', color: '#5f666d', value: 0.2 },
].map((strip) => ({ ...strip, selected: false, flash: false })))

const gm = ref(1)
const blackout = ref(false)
// Saída simulada: o flash leva a 100%; depois aplicam-se GM e blackout
const output = (value: number, flash: boolean) => (blackout.value ? 0 : (flash ? 1 : value) * gm.value)

const editing = ref<number>()
const editingStrip = computed(() => (editing.value === undefined ? undefined : strips.value[editing.value]))
function confirmEdit(value: number) {
  if (editingStrip.value) editingStrip.value.value = value
  editing.value = undefined
}

const knobs = ref([
  { key: 'zoom', value: 0.4, color: 'var(--color-beam)' },
  { key: 'focus', value: 0.7, color: 'var(--color-beam)' },
  { key: 'iris', value: 1, color: 'var(--color-beam)' },
  { key: 'speed', value: 0.25, color: 'var(--color-speed)' },
])
const position = ref({ x: 0.5, y: 0.5 })
const color = ref('#ff6a00')

const demo = ref({ on: true, select: false, danger: false, flash: false })
const demoLed = ref({ on: true, select: true, danger: false, flash: false })
const form = ref({ name: 'PAR 1', address: 1, universe: 1 })
const functions = [
  { id: 'f1', label: 'Palco quente', detail: 'Cena' },
  { id: 'f2', label: 'Abertura', detail: 'Chaser' },
  { id: 'f3', label: 'Círculo MH', detail: 'EFX' },
  { id: 'f4', label: 'Arco-íris nas barras', detail: 'RGB Matrix' },
  { id: 'f5', label: 'Final', detail: 'Coleção' },
]
const selectedFunction = ref('f1')
const dialogOpen = ref(false)
const sheetOpen = ref(false)
const numpadValue = ref<number>()

// Tap tempo: média dos intervalos dos toques dos últimos 3 s
const bpm = ref(120)
let taps: number[] = []
function tap() {
  const now = performance.now()
  taps = [...taps.filter((time) => now - time < 3000), now]
  if (taps.length > 1) bpm.value = Math.round(60000 / ((now - taps[0]!) / (taps.length - 1)))
}
</script>

<template>
  <RlAppShell :side-title="t('dev.side.title')">
    <template #toolbar>
      <span class="brand">R·LIGHT</span>
    </template>
    <template #nav>
      <RlButton v-for="item in nav" :key="item" :model-value="page === item" @press="page = item">
        {{ t(`dev.nav.${item}`) }}
      </RlButton>
    </template>
    <template #toolbar-end>
      <RlLcd>{{ bpm }} BPM</RlLcd>
      <RlButton v-model="dmx" mode="toggle" size="sm" color="var(--color-lcd)">DMX</RlButton>
      <RlButton size="sm" @press="navigateTo('/dev/faders')">{{ t('dev.fadersTest') }}</RlButton>
      <RlButton size="sm" @press="navigateTo('/dev/beams')">3D</RlButton>
    </template>

    <RlTabs
      v-model="tab"
      class="h-full p-2"
      :tabs="[
        { value: 'strips', label: t('dev.tabs.strips') },
        { value: 'controls', label: t('dev.tabs.controls') },
        { value: 'buttons', label: t('dev.tabs.buttons') },
      ]"
    >
      <template #strips>
        <div class="flex h-full overflow-x-auto rounded-md border border-black">
          <RlChannelStrip
            v-for="(strip, i) in strips"
            :key="strip.label"
            v-model="strip.value"
            v-model:selected="strip.selected"
            v-model:flash="strip.flash"
            :label="strip.label"
            :color="strip.color"
            :meter="output(strip.value, strip.flash)"
            @edit="editing = i"
          />
        </div>
      </template>

      <template #controls>
        <div class="grid gap-3 md:grid-cols-2 xl:grid-cols-3">
          <section class="card">
            <h2>{{ t('dev.knobs') }}</h2>
            <div class="flex flex-wrap gap-5">
              <div v-for="knob in knobs" :key="knob.key" class="flex flex-col items-center gap-1.5">
                <RlKnob v-model="knob.value" :color="knob.color" />
                <RlLcd class="w-16">{{ format(knob.value) }}</RlLcd>
                <span class="caption">{{ t(`dev.${knob.key}`) }}</span>
              </div>
            </div>
          </section>
          <section class="card">
            <h2>{{ t('dev.xyPad') }}</h2>
            <RlXYPad v-model="position" />
          </section>
          <section class="card">
            <h2>{{ t('dev.color') }}</h2>
            <RlColorPicker v-model="color" />
          </section>
        </div>
      </template>

      <template #buttons>
        <div class="grid gap-3 md:grid-cols-2 xl:grid-cols-3">
          <section class="card">
            <h2>{{ t('dev.buttons') }}</h2>
            <h3>{{ t('dev.faceHighlight') }}</h3>
            <div class="flex flex-wrap gap-2">
              <RlButton>{{ t('dev.action') }}</RlButton>
              <RlButton v-model="demo.on" mode="toggle" color="var(--color-on)">{{ t('dev.on') }}</RlButton>
              <RlButton v-model="demo.select" mode="toggle">{{ t('rl.select') }}</RlButton>
              <RlButton v-model="demo.danger" mode="toggle" color="var(--color-danger)">{{ t('dev.blackout') }}</RlButton>
              <RlButton v-model="demo.flash" mode="flash" color="var(--color-flash)">{{ t('rl.flash') }}</RlButton>
            </div>
            <h3>{{ t('dev.ledHighlight') }}</h3>
            <div class="flex flex-wrap gap-2">
              <RlButton v-model="demoLed.on" mode="toggle" highlight="led" color="var(--color-on)">{{ t('dev.on') }}</RlButton>
              <RlButton v-model="demoLed.select" mode="toggle" highlight="led">{{ t('rl.select') }}</RlButton>
              <RlButton v-model="demoLed.danger" mode="toggle" highlight="led" color="var(--color-danger)">
                {{ t('dev.blackout') }}
              </RlButton>
              <RlButton v-model="demoLed.flash" mode="flash" highlight="led" color="var(--color-flash)">{{ t('rl.flash') }}</RlButton>
            </div>
            <h3>{{ t('dev.sizes') }}</h3>
            <div class="flex flex-wrap items-center gap-2">
              <RlButton size="sm">{{ t('dev.small') }}</RlButton>
              <RlButton>{{ t('dev.medium') }}</RlButton>
              <RlButton size="lg">{{ t('dev.large') }}</RlButton>
            </div>
            <h3>{{ t('dev.displays') }}</h3>
            <div class="flex flex-wrap gap-2">
              <RlLcd>{{ format(0.75) }}</RlLcd>
              <RlLcd>{{ bpm }} BPM</RlLcd>
              <RlLcd color="var(--color-flash)">CUE 12</RlLcd>
              <RlLcd color="var(--color-danger)">BLACKOUT</RlLcd>
            </div>
          </section>

          <section class="card">
            <h2>{{ t('dev.fields') }}</h2>
            <label class="field-row">
              {{ t('dev.name') }}
              <input v-model="form.name" class="field">
            </label>
            <label class="field-row">
              {{ t('dev.address') }}
              <input v-model.number="form.address" type="number" min="1" max="512" class="field">
            </label>
            <label class="field-row">
              {{ t('dev.universe') }}
              <select v-model="form.universe" class="field">
                <option v-for="n in 4" :key="n" :value="n">{{ t('dev.universeN', { n }) }}</option>
              </select>
            </label>
            <RlSwitch v-model="dmx">{{ t('dev.dmxDisplay') }}</RlSwitch>
          </section>

          <section class="card">
            <h2>{{ t('dev.list') }}</h2>
            <RlList v-model="selectedFunction" :items="functions" />
            <h3>{{ t('dev.dialogs') }}</h3>
            <div class="flex flex-wrap gap-2">
              <RlButton @press="dialogOpen = true">{{ t('dev.openDialog') }}</RlButton>
              <RlButton @press="sheetOpen = true">{{ t('dev.openSheet') }}</RlButton>
            </div>
          </section>

          <section class="card">
            <h2>{{ t('dev.numpad') }}</h2>
            <RlNumpad @confirm="numpadValue = $event" />
            <p class="caption">{{ t('dev.lastValue', { value: numpadValue === undefined ? '—' : format(numpadValue) }) }}</p>
          </section>
        </div>
      </template>
    </RlTabs>

    <RlDialog
      :open="editing !== undefined"
      :title="t('dev.editValue', { name: editingStrip?.label ?? '' })"
      @update:open="editing = undefined"
    >
      <RlNumpad @confirm="confirmEdit" />
    </RlDialog>
    <RlDialog v-model:open="dialogOpen" :title="t('dev.openDialog')">
      <p>{{ t('dev.dialogText') }}</p>
    </RlDialog>
    <RlDialog v-model:open="sheetOpen" sheet :title="t('dev.openSheet')">
      <p>{{ t('dev.dialogText') }}</p>
    </RlDialog>

    <template #master>
      <div class="master">
        <RlLcd>{{ format(gm) }}</RlLcd>
        <RlButton v-model="blackout" mode="toggle" size="sm" color="var(--color-danger)">{{ t('dev.blackout') }}</RlButton>
        <RlButton size="sm">{{ t('dev.stopAll') }}</RlButton>
        <RlFader v-model="gm" :default="1" cap="red" class="flex-1" :meter="blackout ? 0 : gm" />
        <div class="master-label">{{ t('dev.master') }}</div>
      </div>
    </template>

    <template #side>
      <RlTabs
        v-model="sideTab"
        :tabs="[
          { value: 'banks', label: t('dev.side.banks') },
          { value: 'functions', label: t('dev.side.functions') },
        ]"
      >
        <template #banks>
          <div class="flex flex-col gap-1.5">
            <RlButton
              v-for="n in 6"
              :key="n"
              :model-value="viewPage === n"
              color="var(--color-lcd)"
              @press="viewPage = n"
            >
              {{ t('dev.page', { n }) }}
            </RlButton>
            <RlButton class="mt-4" size="lg" @press="tap">{{ t('dev.tapTempo') }}</RlButton>
          </div>
        </template>
        <template #functions>
          <RlList v-model="selectedFunction" :items="functions" />
        </template>
      </RlTabs>
    </template>

    <template #bottom>
      <RlButton v-for="n in 4" :key="n" size="sm" :model-value="bank === n" @press="bank = n">
        {{ t('dev.bank', { from: n * 8 - 7, to: n * 8 }) }}
      </RlButton>
    </template>
  </RlAppShell>
</template>

<style scoped>
.brand {
  margin-right: 8px;
  font-weight: 800;
  letter-spacing: 0.14em;
  text-shadow: 0 1px 0 #000;
  white-space: nowrap;
}

.card {
  display: flex;
  flex-direction: column;
  gap: 10px;
  padding: 12px;
  border: 1px solid #000;
  border-radius: 8px;
  background: linear-gradient(#1b1d21, #15171a);
  box-shadow: inset 0 1px 0 rgb(255 255 255 / 0.05);
}

h2,
h3 {
  color: var(--color-fg-muted);
  font-size: 12px;
  font-weight: 700;
  letter-spacing: 0.08em;
  text-transform: uppercase;
}

h3 {
  margin-top: 6px;
  font-size: 11px;
}

.caption {
  color: var(--color-fg-muted);
  font-size: 11px;
  text-transform: uppercase;
}

.field-row {
  display: grid;
  gap: 4px;
  color: var(--color-fg-muted);
  font-size: 12px;
}

.master {
  display: flex;
  flex-direction: column;
  gap: 6px;
  width: 104px;
  padding: 6px 6px 0;
}

.master-label {
  margin: 0 -6px;
  padding: 8px 4px;
  background: #0b0c0e;
  border-top: 3px solid var(--color-danger);
  color: var(--color-flash);
  font-size: 11px;
  font-weight: 700;
  text-align: center;
  text-transform: uppercase;
}
</style>

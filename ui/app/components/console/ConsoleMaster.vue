<script setup lang="ts">
import { LightbulbOff, OctagonX } from 'lucide-vue-next'

// Seção master do console (E12): Grand Master, Blackout, Stop All e o BPM global com tap.
// Coluna ao lado do console; em espaço estreito, uma faixa em cima com o GM num knob.
const { t } = useI18n()
const { state, send } = useEngine()
const { format } = useValueFormat()
const master = computed(() => state.value.master)
const gm = computed({
  get: () => master.value.grandMaster / 255,
  set: (v: number) => send({ op: 'setGrandMaster', value: Math.round(v * 255) }),
})
const blackout = computed({ get: () => master.value.blackout, set: (on: boolean) => send({ op: 'setBlackout', on }) })

const vc = useConsole()
const prefs = usePrefs()
</script>

<template>
  <div class="flex shrink-0">
    <section class="flex flex-1 items-center gap-2 @3xl:hidden">
      <RlButton v-model="blackout" mode="toggle" color="var(--color-danger)" size="sm" :aria-label="t('vc.blackout')"><LightbulbOff :size="18" /></RlButton>
      <RlButton size="sm" :aria-label="t('vc.stopAll')" @press="send({ op: 'stopAllFunctions' })"><OctagonX :size="18" /></RlButton>
      <RlKnob v-model="gm" :default="1" :size="40" color="var(--color-danger)" />
      <RlLcd>GM {{ format(gm) }}</RlLcd>
      <RlLcd class="ml-auto">{{ master.bpm }} BPM</RlLcd>
      <RlButton size="sm" @press="vc.tapBpm">{{ t('vc.tap') }}</RlButton>
    </section>

    <section class="master hidden w-24 shrink-0 flex-col items-stretch gap-2 @3xl:flex">
      <h2 class="heading flex items-center justify-center gap-1">{{ t('vc.master') }} <RlInfo :text="t('vc.info.master')" /></h2>
      <RlFader v-model="gm" class="min-h-0 flex-1" cap="red" :default="1" />
      <RlLcd class="text-center">GM {{ format(gm) }}</RlLcd>
      <!-- As etiquetas mostram os atalhos fixos do teclado numérico (somem com as dicas de atalho desligadas) -->
      <RlButton v-model="blackout" mode="toggle" color="var(--color-danger)" class="flex-col! gap-1! py-2!">
        <span v-if="prefs.keyHints" class="rl-key">Num -</span>
        <LightbulbOff :size="18" />
        {{ t('vc.blackout') }}
      </RlButton>
      <RlButton class="flex-col! gap-1! py-2!" @press="send({ op: 'stopAllFunctions' })">
        <span v-if="prefs.keyHints" class="rl-key">Num +</span>
        <OctagonX :size="18" />
        {{ t('vc.stopAll') }}
      </RlButton>
      <RlLcd class="text-center">{{ master.bpm }} BPM</RlLcd>
      <RlButton class="flex-col! gap-1! py-2!" @press="vc.tapBpm">
        <span v-if="prefs.keyHints" class="rl-key">Num Enter</span>
        {{ t('vc.tap') }}
      </RlButton>
    </section>
  </div>
</template>

<style scoped>
.master {
  padding: 8px;
  border: 1px solid #000;
  border-radius: 8px;
  background: linear-gradient(#1b1d21, #15171a);
  box-shadow: inset 0 1px 0 rgb(255 255 255 / 0.05);
}
</style>

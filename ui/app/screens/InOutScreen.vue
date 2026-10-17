<script setup lang="ts">
import { KeyboardMusic, Plus, Trash2, Usb } from 'lucide-vue-next'
import type { Universe } from '~/types/generated/Universe'

// In/Out (E12): saídas (universos e interfaces DMX) e entradas (controladores MIDI conectados).
// O que cada controle MIDI ou tecla faz é definido no próprio elemento do console (E11), como no QLC+.
const { t } = useI18n()
const { state, send } = useEngine()
const project = computed(() => state.value.project)

const fixturesIn = (id: string) => project.value.fixtures.filter((f) => f.universe === id).length
const interfaceOf = (u: Universe) => state.value.interfaces.find((i) => i.id === u.output)

function updateUniverse(u: Universe, changes: Partial<Universe>) {
  send({ op: 'updateUniverse', universe: { ...u, ...changes } })
}
</script>

<template>
  <div class="flex h-full flex-col gap-3 overflow-auto p-2">
    <h2 class="text-lg font-bold">{{ t('io.outputs') }}</h2>
    <div class="grid grid-cols-1 gap-3 @5xl:grid-cols-2">
      <section class="panel">
        <h3 class="heading">{{ t('io.universes') }}</h3>
        <table class="rl-table table-fixed">
          <thead>
            <tr>
              <th class="w-2/5">{{ t('io.name') }}</th>
              <th>{{ t('io.output') }}</th>
              <th class="w-14" />
            </tr>
          </thead>
          <tbody>
            <tr v-for="u in project.universes" :key="u.id">
              <td>
                <input class="field w-full min-w-28" :value="u.name" @change="updateUniverse(u, { name: ($event.target as HTMLInputElement).value })">
                <small class="text-fg-dim">{{ t('io.fixturesIn', { n: fixturesIn(u.id) }) }}</small>
              </td>
              <td>
                <span class="flex items-center gap-2">
                  <span class="rl-led" :style="{ '--led': interfaceOf(u)?.connected ? 'var(--color-on)' : 'var(--color-fg-dim)' }" />
                  <select class="field min-w-0 flex-1" :value="u.output ?? ''" @change="updateUniverse(u, { output: ($event.target as HTMLSelectElement).value || null })">
                    <option value="">{{ t('io.none') }}</option>
                    <option v-for="i in state.interfaces" :key="i.id" :value="i.id">{{ i.name }}</option>
                  </select>
                </span>
              </td>
              <td>
                <RlButton
                  size="sm"
                  :aria-label="t('common.remove')"
                  :title="fixturesIn(u.id) ? t('io.universeInUse') : t('common.remove')"
                  @press="!fixturesIn(u.id) && send({ op: 'removeUniverse', id: u.id })"
                >
                  <Trash2 :size="16" :class="{ 'opacity-30': fixturesIn(u.id) }" />
                </RlButton>
              </td>
            </tr>
          </tbody>
        </table>
        <RlButton size="sm" class="self-start" @press="send({ op: 'addUniverse' })">
          <Plus :size="16" />
          {{ t('io.addUniverse') }}
        </RlButton>
      </section>

      <section class="panel">
        <h3 class="heading">{{ t('io.interfaces') }}</h3>
        <div v-for="i in state.interfaces" :key="i.id" class="sunken flex flex-wrap items-center gap-3 rounded-md p-3">
          <Usb :size="22" class="text-fg-muted" />
          <!-- basis-48: em painel estreito os medidores descem para a linha de baixo -->
          <div class="min-w-0 flex-1 basis-48">
            <p class="truncate font-semibold">{{ i.name }}</p>
            <p class="text-xs text-fg-muted">{{ t(`interfaceKind.${i.kind}`) }}<template v-if="i.serial"> · {{ t('io.serial') }} {{ i.serial }}</template></p>
          </div>
          <span class="flex items-center gap-1.5 text-xs">
            <span class="rl-led" :style="{ '--led': i.connected ? 'var(--color-on)' : 'var(--color-danger)' }" />
            {{ i.connected ? t('io.connected') : t('io.disconnected') }}
          </span>
          <RlLcd>{{ i.fps.toFixed(0) }} fps</RlLcd>
          <RlLcd>{{ i.jitterMs.toFixed(1) }} ms</RlLcd>
        </div>
      </section>
    </div>

    <h2 class="mt-2 text-lg font-bold">{{ t('io.inputs') }}</h2>
    <div class="grid grid-cols-1 gap-3 @5xl:grid-cols-2">
      <section class="panel">
        <h3 class="heading">{{ t('io.controllers') }}</h3>
        <div v-for="d in state.midiDevices" :key="d.id" class="sunken flex items-center gap-3 rounded-md p-3">
          <KeyboardMusic :size="22" class="text-fg-muted" />
          <p class="min-w-0 flex-1 truncate font-semibold">{{ d.name }}</p>
          <span class="flex items-center gap-1.5 text-xs">
            <span class="rl-led" :style="{ '--led': d.connected ? 'var(--color-on)' : 'var(--color-danger)' }" />
            {{ d.connected ? t('io.deviceConnected') : t('io.deviceDisconnected') }}
          </span>
        </div>
        <p v-if="!state.midiDevices.length" class="text-sm text-fg-muted">{{ t('io.noControllers') }}</p>
      </section>
    </div>
  </div>
</template>

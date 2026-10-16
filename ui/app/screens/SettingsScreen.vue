<script setup lang="ts">
import { renderSVG } from 'uqr'

// Configurações: preferências deste aparelho (formato de valores, modo modular), acesso remoto
// do servidor, e sobre/licenças.
const { t } = useI18n()
const prefs = usePrefs()
const { mode } = useValueFormat()
const { state, send } = useEngine()
const wide = useMediaQuery('(min-width: 1024px)')
const dmx = computed({ get: () => mode.value === 'dmx', set: (on) => (mode.value = on ? 'dmx' : 'percent') })
const remote = computed(() => state.value.remote)
const pin = ref(remote.value.pin ?? '')
const qr = computed(() => (remote.value.enabled && remote.value.lanUrl ? renderSVG(remote.value.lanUrl, { border: 2 }) : ''))
</script>

<template>
  <div class="grid content-start gap-3 p-2 @3xl:grid-cols-2 @6xl:grid-cols-3">
    <section class="panel">
      <h2 class="heading flex items-center gap-2">
        {{ t('settings.display') }}
        <RlInfo :text="t('settings.displayInfo')" />
      </h2>
      <RlSwitch v-model="dmx">{{ t('settings.dmxValues') }}</RlSwitch>
      <RlSwitch v-if="wide" v-model="prefs.modular">{{ t('settings.modular') }}</RlSwitch>
    </section>

    <section class="panel">
      <h2 class="heading">{{ t('settings.remote') }}</h2>
      <RlSwitch :model-value="remote.enabled" @update:model-value="send({ op: 'setRemoteAccess', enabled: $event, pin: remote.pin })">
        {{ t('settings.allowLan') }}
      </RlSwitch>
      <label class="field-label">
        {{ t('settings.pin') }}
        <span class="flex gap-2">
          <input v-model="pin" inputmode="numeric" maxlength="8" class="field min-w-0 flex-1" :placeholder="t('settings.noPin')">
          <RlButton @press="send({ op: 'setRemoteAccess', enabled: remote.enabled, pin: pin || null })">{{ t('common.save') }}</RlButton>
        </span>
      </label>
      <template v-if="qr">
        <h3 class="heading flex items-center gap-2">
          {{ t('settings.scan') }}
          <RlInfo :text="t('settings.scanInfo')" />
        </h3>
        <div class="flex items-center gap-3">
          <!-- eslint-disable-next-line vue/no-v-html -- SVG gerado localmente pelo uqr -->
          <div class="w-32 shrink-0 bg-white" v-html="qr" />
          <code class="text-sm break-all text-lcd">{{ remote.lanUrl }}</code>
        </div>
      </template>
      <h3 class="heading">{{ t('settings.clients') }}</h3>
      <ul class="flex flex-col gap-2">
        <li v-for="c in remote.clients" :key="c.id" class="flex items-center gap-2 text-sm">
          <span class="rl-led" />
          <span class="font-semibold">{{ c.name }}</span>
          <span class="text-fg-muted">{{ c.address }}</span>
          <span v-if="c.thisDevice" class="text-xs text-fg-dim">· {{ t('settings.thisDevice') }}</span>
        </li>
      </ul>
    </section>

    <section class="panel">
      <h2 class="heading">{{ t('settings.about') }}</h2>
      <p class="font-semibold">R Light Control 0.1.0</p>
      <p class="text-sm text-fg-muted">{{ t('settings.qlcNotice') }}</p>
    </section>
  </div>
</template>

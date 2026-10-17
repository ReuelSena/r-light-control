<script setup lang="ts">
import { renderSVG } from 'uqr'

// Configurações (E12): painel lateral aberto pelo ⚙ sobre a tela atual, para ajustar algo sem sair do que
// se está fazendo. Preferências deste aparelho (zoom, formato de valores, modo modular, formato do console),
// acesso remoto do servidor, e sobre/licenças.
const open = defineModel<boolean>('open', { default: false })
const { t } = useI18n()
const prefs = usePrefs()
const { mode } = useValueFormat()
const { state, send } = useEngine()
const wide = useMediaQuery('(min-width: 1024px)')
// O zoom geral só existe no app desktop (zoom nativo da janela); no navegador, vale o do navegador
const nativeZoom = '__TAURI_INTERNALS__' in window
const dmx = computed({ get: () => mode.value === 'dmx', set: (on) => (mode.value = on ? 'dmx' : 'percent') })
const remote = computed(() => state.value.remote)
const pin = ref(remote.value.pin ?? '')
const qr = computed(() => (remote.value.enabled && remote.value.lanUrl ? renderSVG(remote.value.lanUrl, { border: 2 }) : ''))
</script>

<template>
  <RlDialog v-model:open="open" sheet :title="t('screens.settings')">
    <div class="flex flex-col gap-3">
      <h2 class="heading flex items-center gap-2">
        {{ t('settings.display') }}
        <RlInfo :text="t('settings.displayInfo')" />
      </h2>
      <div class="field-label">
        <span class="flex items-center gap-1">{{ t('settings.zoom') }} <RlInfo :text="t('settings.zoomInfo')" /></span>
        <RlStepper v-if="nativeZoom" v-model="prefs.uiZoom" :steps="zoomSteps" />
        <p v-else class="text-sm text-flash">{{ t('settings.zoomBrowser') }}</p>
      </div>
      <RlSwitch v-model="dmx">{{ t('settings.dmxValues') }}</RlSwitch>
      <RlSwitch v-if="wide" v-model="prefs.modular">{{ t('settings.modular') }}</RlSwitch>
      <label class="field-label">
        {{ t('settings.consoleFormat') }}
        <select v-model="prefs.consoleFormat" class="field">
          <option value="auto">{{ t('settings.formatAuto') }}</option>
          <option v-for="f in layoutFormats" :key="f" :value="f">{{ t(`vc.formats.${f}`) }}</option>
        </select>
      </label>

      <h2 class="heading divider">{{ t('settings.remote') }}</h2>
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
        <div class="flex flex-col items-start gap-2">
          <!-- eslint-disable-next-line vue/no-v-html -- SVG gerado localmente pelo uqr -->
          <div class="w-32 shrink-0 bg-white" v-html="qr" />
          <code class="text-sm break-all text-lcd">{{ remote.lanUrl }}</code>
        </div>
      </template>
      <h3 class="heading">{{ t('settings.clients') }}</h3>
      <ul class="flex flex-col gap-2">
        <li v-for="c in remote.clients" :key="c.id" class="flex flex-wrap items-center gap-x-2 text-sm">
          <span class="rl-led" />
          <span class="font-semibold">{{ c.name }}</span>
          <span class="text-fg-muted">{{ c.address }}</span>
          <span v-if="c.thisDevice" class="text-xs text-fg-dim">· {{ t('settings.thisDevice') }}</span>
        </li>
      </ul>

      <h2 class="heading divider">{{ t('settings.about') }}</h2>
      <p class="font-semibold">R Light Control 0.1.0</p>
      <p class="text-sm text-fg-muted">{{ t('settings.qlcNotice') }}</p>
    </div>
  </RlDialog>
</template>

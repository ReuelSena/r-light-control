import tailwindcss from '@tailwindcss/vite'
import tresCompiler from '@tresjs/core/template-compiler-options'

export default defineNuxtConfig({
  compatibilityDate: '2026-10-01',
  // SPA: a UI é servida como arquivos estáticos pelo servidor Rust (E1/E2).
  ssr: false,
  // O botão flutuante do DevTools cobre a navegação inferior no celular.
  devtools: { enabled: false },
  modules: ['@nuxt/eslint', '@nuxtjs/i18n'],
  // Fontes embutidas: a UI precisa funcionar sem internet.
  css: ['@fontsource-variable/inter', '@fontsource/share-tech-mono', '~/assets/css/main.css'],
  // 0.0.0.0 para o celular acessar o servidor de dev pela rede local.
  devServer: { host: '0.0.0.0', port: 3000 },
  // Tela de "Conectando…" enquanto o app espera o estado do servidor (plugins/engine.client.ts)
  spaLoadingTemplate: 'spa-loading-template.html',
  runtimeConfig: {
    // Porta do servidor Rust, usada só em dev (em produção a UI vem dele mesmo).
    public: { serverPort: 8090 },
  },
  // Tags <Tres*> são elementos do three.js, não componentes Vue.
  vue: { compilerOptions: tresCompiler.template.compilerOptions },
  vite: {
    plugins: [tailwindcss()],
    // Evita o reload da página quando o Vite descobre a dependência em tempo de execução.
    optimizeDeps: { include: ['uqr', 'three', '@tresjs/core', 'reka-ui', 'lucide-vue-next', 'dockview-vue'] },
  },
  i18n: {
    defaultLocale: 'pt-BR',
    strategy: 'no_prefix',
    locales: [{ code: 'pt-BR', language: 'pt-BR', file: 'pt-BR.json' }],
  },
  app: {
    head: {
      title: 'R Light Control',
      htmlAttrs: { lang: 'pt-BR' },
      meta: [{ name: 'viewport', content: 'width=device-width, initial-scale=1, maximum-scale=1, user-scalable=no, viewport-fit=cover' }],
    },
  },
})

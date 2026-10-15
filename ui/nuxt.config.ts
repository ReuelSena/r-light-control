export default defineNuxtConfig({
  compatibilityDate: '2026-10-01',
  // SPA: a UI é servida como arquivos estáticos pelo servidor Rust (E1/E2).
  ssr: false,
  modules: ['@nuxt/eslint', '@nuxtjs/i18n'],
  // 0.0.0.0 para o celular acessar o servidor de dev pela rede local.
  devServer: { host: '0.0.0.0', port: 3000 },
  runtimeConfig: {
    // Porta do servidor Rust, usada só em dev (em produção a UI vem dele mesmo).
    public: { serverPort: 8090 },
  },
  // Evita o reload da página quando o Vite descobre a dependência em tempo de execução.
  vite: { optimizeDeps: { include: ['uqr'] } },
  i18n: {
    defaultLocale: 'pt-BR',
    strategy: 'no_prefix',
    locales: [{ code: 'pt-BR', language: 'pt-BR', file: 'pt-BR.json' }],
  },
  app: {
    head: {
      title: 'R Light Control',
      htmlAttrs: { lang: 'pt-BR' },
      meta: [{ name: 'viewport', content: 'width=device-width, initial-scale=1, viewport-fit=cover' }],
    },
  },
})

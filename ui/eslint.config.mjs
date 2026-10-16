import withNuxt from './.nuxt/eslint.config.mjs'

// Os tipos gerados pelo ts-rs seguem o estilo do gerador.
export default withNuxt({ ignores: ['app/types/generated/**'] })

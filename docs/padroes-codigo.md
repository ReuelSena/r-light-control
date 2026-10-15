# Padrões de código

## 1. Princípio principal: código enxuto e legível (G7)

- Escreva a menor quantidade de código que resolve o problema **e** continua fácil de ler. Não minifique: nomes claros e uma ideia por linha.
- **Sem helpers de uso único.** Se o código direto cabe em menos linhas que a definição da função, deixe-o direto.
- Extraia uma função, um componente ou um composable quando houver **repetição real** (3 usos ou mais) ou quando isso isolar uma fronteira do projeto (ex.: `EngineClient`, drivers DMX).
- Sem camadas "para o futuro". Crates, pastas e abstrações nascem na fase em que passam a ser necessários.
- Comentários explicam **por quê**, nunca o que o código já diz.

```rust
// ✗ helper que não paga a própria definição
fn is_full(v: u8) -> bool { v == 255 }
if is_full(value) { … }

// ✓ direto
if value == 255 { … }
```

## 2. Geral

- Identificadores em inglês; comentários, documentação e textos da UI em pt-BR (G6).
- Tipos compartilhados entre Rust e TypeScript são definidos no Rust e gerados para TS (ts-rs, a partir da F2). Nunca edite os tipos gerados à mão.
- `pnpm check` precisa passar antes de entregar uma fase.

## 3. Rust

- Edition 2024, `rustfmt` padrão, `clippy -D warnings`.
- Erros:
  - `thiserror` nas bibliotecas quando o chamador precisa distinguir os erros.
  - `anyhow` nos binários e no código de borda.
- Nada de `unwrap()` fora de testes. Use `expect("motivo")` só para invariantes que nunca deveriam falhar.
- O motor (`rlc-engine`) não usa async: threads e canais. Tokio só no servidor e em I/O de arquivos.
- Testes como especificação: nomes que descrevem o comportamento (`htp_keeps_highest_intensity`), num `mod tests` no próprio arquivo ou em `tests/`.
- Um crate novo só quando houver uma fronteira real (I/O isolado, binário separado, tempo de compilação).

## 4. UI (Nuxt / Vue / TypeScript)

- Nuxt 4 em modo SPA. O código fica em `ui/app/` (`pages/`, `components/`, `composables/`, `stores/`).
- Componentes com `<script setup lang="ts">`.
- Componentes do design system com prefixo `Rl` (`RlFader`, `RlButton`) em `ui/app/components/rl/`.
- Estado:
  - local no componente por padrão;
  - store Pinia por domínio só quando o estado é compartilhado entre telas.
- Composables só quando reutilizados.
- Todo texto visível passa pelo i18n (`t('chave')`). Os textos ficam em `ui/i18n/locales/pt-BR.json`.
- Dados de alta frequência (valores DMX, preview) ficam fora da reatividade do Vue (`shallowRef`, `markRaw`, `Uint8Array`).
- Lint: `@nuxt/eslint`. Typecheck: `nuxt typecheck`.

## 5. Comunicação UI ↔ servidor

- Só por WebSocket (`/ws`) e HTTP (E2). O desktop e o celular usam exatamente o mesmo código.
- Mensagens JSON com campo `t` (tipo): `hello`, `cmd`, `ack`, `err`, `evt`. Dados de alta taxa vão em frames binários.
- A especificação completa vai para `docs/protocolo.md` na F2.

## 6. Git

- Commits só quando o usuário pedir (G5).
- Conventional Commits com descrição em pt-BR, por exemplo `feat(dmx): saída Enttec Pro` ou `fix(ui): fader solta ao perder o toque`.

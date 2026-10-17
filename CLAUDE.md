# r-light-control

Controlador de iluminação DMX: app desktop que também serve a própria interface na rede local para celular e tablet. Usa o QLC+ como referência.

Stack: Tauri 2 + Rust (motor, drivers, servidor) + Nuxt 4 SPA (UI).

@DECISOES.md
@PLANO.md

## Ao iniciar cada sessão

- Ver no `PLANO.md` a fase em andamento (🟡) e o que falta no checklist dela.
- Seguir o `DECISOES.md`, principalmente G1 (fases com validação), G2 (interface antes de recursos) e G7 (código enxuto).
- Padrões de código: [docs/padroes-codigo.md](docs/padroes-codigo.md).
- Decisão nova ou pedido novo do usuário → registrar no `DECISOES.md` ou no `PLANO.md`.

## Estrutura

- `crates/rlc-core`: contrato de dados (só tipos). `cargo test -p rlc-core` gera os tipos TypeScript em `ui/app/types/generated/` (nunca editar à mão).
- `crates/rlc-server`: servidor axum (UI embutida + `/ws`), configurações e logs.
- `app/src-tauri`: casca Tauri. Sobe o servidor, abre a janela e transforma `window.open` em janela nativa (E13).
- `ui/`: Nuxt 4 em modo SPA (código em `ui/app/`, textos em `ui/i18n/locales/`).
  - `screens/`: telas do app; o registro fica em `utils/screens.ts`. A mesma tela vira página (`pages/[screen].vue`), painel do modo modular (`components/app/AppDock.vue`) ou janela externa (`pages/janela/[screen].vue`).
  - `composables/useEngine.ts`: a única porta das telas para o motor. `engine/mock.ts` é o motor de mentira da F2. Protocolo em [docs/protocolo.md](docs/protocolo.md).
  - `utils/functions.ts`: ícones dos tipos de criação e as contas do EFX, do RGB Matrix e dos passos do chaser, usadas pelas prévias do Studio e pela saída simulada.
  - `utils/console.ts`: formatos e grade do console, tipos de elemento e cópia de layout entre formatos (VC-2). `composables/useConsole.ts`: o que cada elemento faz em operação (toque e teclas).
  - Componentes do design system em `ui/app/components/rl/`; regras de uso em [docs/design-system.md](docs/design-system.md).
  - Páginas de desenvolvimento em `/dev/*`: catálogo (`showcase`), `faders`, `beams` e `conexao`.
- `docs/`: padrões, design system, protocolo, mini-planos de fase (`docs/fases/`).
- `reference/qlcplus`: código do QLC+ (tag `QLC+_5.3.0`, fora do git). Só leitura; nunca compilar nem executar.

## Comandos

- `pnpm install`: dependências da UI e do CLI do Tauri.
- `pnpm dev`: app desktop com a UI em hot reload. O celular abre a URL do QR, que em dev aponta para a porta 3000.
- `pnpm check`: rustfmt, clippy `-D warnings`, `cargo test`, typecheck e lint da UI. Precisa passar antes de entregar uma fase.
- `pnpm build`: gera o instalador.

## Arquivos do usuário (Windows)

- Configurações: `%APPDATA%\r-light-control\settings.json` (porta e bind).
- Logs: `%LOCALAPPDATA%\r-light-control\logs\`.

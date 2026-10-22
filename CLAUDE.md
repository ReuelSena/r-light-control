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
- `crates/rlc-engine`: motor (A1). Thread a 50 Hz com prazo absoluto: roda as criações (cenas, coleções, os chasers de `chaser.rs`, no tempo ou nas batidas do BPM, e os EFX de `efx.rs`), mistura as cenas acesas e o pan/tilt dos EFX (pela prioridade e, nela, HTP/LTP, com delays e fades nas curvas; os EFX relativos por cima da prévia), segura parado o que está pausado e publica sem trava o quadro de cada universo e o passo dos chasers.
- `crates/rlc-dmx`: saídas DMX, cada interface na sua thread: Dummy e, em `ftdi.rs`, Enttec Pro e Open DMX pelo D2XX da FTDI (carregado do driver instalado).
- `crates/rlc-qxf`: importador de fixtures `.qxf` do QLC+ para o formato próprio, com os avisos do relatório. Os testes usam os `.qxf` do rig do usuário (`tests/qxf/`) e a biblioteca inteira da referência.
- `crates/rlc-server`: servidor axum (UI embutida + `/ws`), configurações, logs, a biblioteca de fixtures (`library.rs`) e os arquivos de projeto e de recuperação, com as migrações de versão (`project.rs`). É a autoridade do estado (A2): `apply.rs` tem as regras de cada operação e traduz para o motor o patch e as criações rodando (com os tempos do elemento "Tempos", a cena dividida pelos tempos por tipo de atributo, a prioridade e a pausa) e o que a cue list e os botões fazem (`console_op`), `hub.rs` o estado, o histórico do desfazer, o motor e as saídas, `ws.rs` a conversa com cada cliente. O show de exemplo é `src/exemplo.json` (gravado na pasta dos projetos na primeira vez; os testes de Rust abrem com ele).
- `crates/rlc-headless`: o mesmo servidor sem janela (`rlc-headless --port N`), para testes.
- `app/src-tauri`: casca Tauri. Sobe o servidor, abre a janela e transforma `window.open` em janela nativa (E13).
- `ui/`: Nuxt 4 em modo SPA (código em `ui/app/`, textos em `ui/i18n/locales/`).
  - `screens/`: telas do app; o registro fica em `utils/screens.ts`. A mesma tela vira página (`pages/[screen].vue`), painel do modo modular (`components/app/AppDock.vue`) ou janela externa (`pages/janela/[screen].vue`).
  - `composables/useEngine.ts`: a única porta das telas para o motor. `engine/ws.ts` é o cliente WebSocket do servidor. Protocolo em [docs/protocolo.md](docs/protocolo.md). O app só abre com o estado do servidor (`plugins/engine.client.ts`).
  - `utils/functions.ts`: ícones dos tipos de criação, as contas do EFX (as mesmas do motor, em `efx.rs`) e do RGB Matrix (usadas pelas prévias do Studio e pelo console), os grupos de fixtures aplicados na cena e no EFX (as mesmas contas do servidor, `scene_values` em `apply.rs`) e a regra de uma criação não conter a si mesma. O passo atual dos chasers vem do motor (`chaserSteps`).
  - `utils/console.ts`: formatos e grade do console, tipos de elemento e cópia de layout entre formatos (VC-2). `composables/useConsole.ts`: o que cada elemento faz em operação (toque e teclas).
  - Componentes do design system em `ui/app/components/rl/`; regras de uso em [docs/design-system.md](docs/design-system.md).
  - Páginas de desenvolvimento em `/dev/*`: catálogo (`showcase`), `faders`, `beams` e `dmx` (padrões de teste da saída).
- `docs/`: padrões, design system, protocolo, mini-planos de fase (`docs/fases/`), as telas congeladas da F2 (`docs/telas/`) e a validação final (`validacao-final.md`: os testes com os moving beams, adiados até o usuário poder fazê-los).
- `reference/qlcplus`: código do QLC+ (tag `QLC+_5.3.0`, fora do git). Só leitura; nunca compilar nem executar.

## Comandos

- `pnpm install`: dependências da UI e do CLI do Tauri.
- `pnpm dev`: app desktop com a UI em hot reload. O celular abre a URL do QR, que em dev aponta para a porta 3000.
- `pnpm check`: rustfmt, clippy `-D warnings`, `cargo test`, typecheck e lint da UI. Precisa passar antes de entregar uma fase.
- `pnpm build`: gera o instalador (`target/release/bundle/nsis/`).
- O `pnpm dev` usa as pastas de verdade do usuário (projetos, biblioteca, recuperação e `settings.json`). Nos testes de navegador, guarde uma cópia antes e desfaça no fim.

## Arquivos do usuário (Windows)

- Configurações: `%APPDATA%\r-light-control\settings.json` (porta, bind, acesso pela rede local, PIN e o tipo escolhido para cada interface USB).
- Projetos: `Documentos\R Light Control\` (`.rlc`, com o `.rlc.bak` da versão anterior). O último aberto fica em `lastProject` no `settings.json`.
- Recuperação: `%APPDATA%\r-light-control\recuperacao.rlc`, enquanto há alterações não salvas.
- Biblioteca de fixtures: `%APPDATA%\r-light-control\fixtures\`, um `.json` por definição. Sem a pasta, o app a cria com as genéricas do exemplo. O `pnpm dev` grava nela de verdade.
- Logs: `%LOCALAPPDATA%\r-light-control\logs\`.

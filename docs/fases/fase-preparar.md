# Modo Preparar (VC-6) — mini-plano e checklist de execução

Pedido do usuário (2026-11-15, G4): preparar a luz da próxima música sem afetar a luz real. Plano completo aprovado: `C:\Users\Eliseu\.claude\plans\eu-queria-criar-um-jaunty-swing.md` (cópia resumida abaixo).
Exceção ao G1: executado numa rodada só, a pedido do usuário. Agentes: Sonnet 5.5 (médio) para tarefas simples/médias, Opus 5.5 (médio) para as muito complexas.

## Comportamento (decidido)
- **Ctrl+Enter** liga/desliga o modo Preparar (por aparelho, `useState`). Botões "Preparar" (toggle) e "Executar" (ativo só com algo preparado) ao lado de "Editar" na barra do console.
- Com o modo ligado, as ações dos elementos (botões, sliders, XY, speed dial, cue list, teclas/MIDI) vão como `Staged`: não tocam a luz real; os elementos mostram o estado preparado.
- Desligar mantém a seleção preparada (fica no servidor, única para o show, vale para todos os aparelhos).
- **Enter** (fileira de cima; só com o modo ligado e algo preparado) = Executar: `CommitStaged` — o preparado vira real e é limpo. `Num Enter` continua sendo o tap.
- Sempre reais, mesmo no modo: pânico, Blackout, Stop All, GM, BPM/tap.
- 3D View: chave "Dividido / Só preview" (preferência do aparelho): dividido = real e preview lado a lado; só preview = tela toda com o preview. Faixa "PREVIEW".
- Nome: "Preparar". Flash/Temp preparados: ignorados (só liga/desliga e níveis).

## Contrato (os agentes seguem exatamente isto)
- `rlc-core`: `Snapshot.staged: StagedState { running: Vec<Id>, paused: Vec<Id>, widget_values: Vec<WidgetValue>, loaded_steps: Vec<CurrentStep> }` (topic `"staged"`, fora do projeto e do desfazer; zerado no Novo/Abrir).
- `Op::Staged { op: Box<Op> }` (aplica a op de console interna só no `staged`), `Op::CommitStaged`, `Op::ClearStaged`. Todas "live" (`is_live`).
- Motor: `Command::Shadow(Box<Command>)` + `Engine.shadow_frames` (**feito**, ver abaixo).
- Transporte: frame binário `PREVIEW_FRAME` (mesmo formato do `UNIVERSE_FRAME`, tipo novo), enviado só a quem mandou a mensagem `{t:"previewSub", on:bool}`; `ws.ts` guarda em `previewOutput` (`shallowRef<Record<universeId,Uint8Array>>`) e expõe `subscribePreview(on)`.

## Checklist (marque [x] a cada tarefa e anote o estado; outro agente continua daqui)
- [x] 0. Registro: este arquivo.
- [x] 1. Motor: `Command::Shadow`, segundo `State` na mesma thread, `Engine.shadow_frames` (editado em `crates/rlc-engine/src/lib.rs`; falta compilar e testar) — _Opus/eu_
- [x] 1b. Motor: `cargo check -p rlc-engine` passa (2026-11-15). Falta o teste unitário do sombra (o agente da tarefa 2 pode acrescentá-lo).
- [x] 2. Core + servidor (Opus) — concluída em 2026-11-15: testes (`staged_ops_stay_out_of_the_live_state_until_committed`, `clear_staged_and_new_project_drop_what_was_prepared`, `only_console_ops_are_staged` em `hub.rs`; o do sombra no motor), `cargo test`, clippy `-D warnings` e `cargo fmt` passam. Itens originais: `rlc-core` (StagedState, Ops, regenerar TS com `cargo test -p rlc-core`), `apply.rs` (Topic::Staged, regras), `hub.rs` (`Staged` aplica a op interna sobre uma visão com os campos `staged` trocados e devolve, sem publish/sync do real; envia ao motor `Command::Shadow(Running(running_programs(staged visão)))` e Patch/Bpm/Tempo/Step/Cut espelhados; `CommitStaged`/`ClearStaged`; Novo/Abrir limpam), testes de Rust.
  - [x] 2a. `rlc-core`: `StagedState`, `Snapshot.staged`, `Op::Staged { inner }`, `CommitStaged`, `ClearStaged`; TS regenerado. **Desvio:** o campo é `inner` (não `op`), porque `op` é a etiqueta do enum no serde: `{ op: "staged", inner: {...} }`.
  - [x] 2b. `apply.rs`: `Topic::Staged` ("staged"), `is_live` das 3 ops, Novo/Abrir zeram `staged`; regras de `CommitStaged` (soma ao ao vivo: criações acesas continuam; valores e passos carregados do preparado substituem os do mesmo elemento/chaser) e `ClearStaged`. Motor: teste `shadow_publishes_its_own_frame_without_touching_the_output`.
  - [x] 2c. `hub.rs`: `Hub::staged` (troca com `swap_staged`, `console_op` + `apply`, troca de volta, publica só `staged`), sombra recebe Patch, Running (do preparado, quando há algo ou ele muda), Bpm, Tempo, Cut e o Step preparado; `Hub::shadow_frames()` (com `#[allow(dead_code)]` até a tarefa 3 usá-lo no `ws.rs`: **tirar o allow**).
- [x] 3. Transporte (Sonnet) — concluída em 2026-11-15: `ws.rs` (`PREVIEW_FRAME` = byte `2`, `{"t":"previewSub","on"}` por conexão, `sent_preview` limpo ao ligar; `AppState.shadow_frames`; `send_frames` recebe a fonte e o tipo), `ws.ts` (`previewOutput`, `subscribePreview`, reenvia no `onopen` se ligado, limpa `previewOutput` ao desligar), `EngineClient` em `useEngine.ts`, `docs/protocolo.md`. `allow(dead_code)` removido. Sem teste de Rust novo (não há testes de ws no estilo existente). Clippy, `cargo test -p rlc-server`, typecheck e lint da UI passam.
- [x] 4. UI console (Sonnet) — concluída em 2026-11-15; typecheck e lint passam. `useConsole` tem `prepare` (`useState("consolePrepare")`), `op` (embrulha em `staged`), `hasStaged`, `staged(w)`; botões Preparar/Executar/Descartar em `ConsoleScreen.vue`; destaque tracejado ciano (`--color-lcd`) em `ConsoleCanvas.vue`. Itens: estado do modo, `useConsole.ts` embrulha em `Staged`, leitura do estado preparado nos elementos, botões "Preparar" e "Executar" na barra, atalhos Ctrl+Enter e Enter (reservados), i18n pt-BR.
- [x] 5. UI 3D (Sonnet) — concluída em 2026-11-15; typecheck e lint passam. `View3dStage.vue` ganhou a prop `source: 'live' | 'preview'` (lê `output` ou `previewOutput` no `onLoop`); `View3dScreen.vue` põe dois `View3dStage` (o do preview sem edição) quando o Preparar está ligado, com a chave Dividido/Só preview (`prefs.previewView`, em `usePrefs`), etiquetas AO VIVO (vermelho) e PREVIEW (ciano), e `subscribePreview` ligado ao `useState('consolePrepare')` enquanto a tela está montada. Falta ver no navegador (tarefa 7).
- [x] 6. Registro (feito: PLANO.md VC-6, DECISOES.md E11 e registro; `pnpm check` passou em 2026-11-15). Registro em `DECISOES.md` (E11) e `PLANO.md` (VC-6), checklist de validação do usuário, `pnpm check`.
- [x] 8. Pré-carregar o preview (2026-11-15, pedido do usuário): ao ligar o modo com o preparado vazio, `seedStaged` copia o ao vivo e marca `StagedState.full`; o Executar de um preparado `full` substitui o ao vivo (o que foi desligado no preview para pelo caminho normal); Backspace (modo ligado, fora de campo, diálogo e edição) faz `clearStaged`. Testes em `hub.rs`; `setPrepare` em `useConsole.ts` (botão do console, Ctrl+Enter e botão da 3D View).
- [ ] 7. Verificação no navegador (copiar projetos do usuário antes; ver memória browser-testing-setup).

## Validação com você (preencher na entrega)
- [ ] Ctrl+Enter liga; tocar um botão acende só o preview na 3D (dividida e só preview).
- [ ] Desligar e religar mantém a seleção; a luz real não muda.
- [ ] Enter executa: a luz real/Monitoramento muda e o preparado limpa.
- [ ] Dois aparelhos veem a mesma seleção preparada.

## Decisões tomadas sem perguntar (informar no fim)
- Transporte: o `previewSub` não tem resposta (sem `id`); o servidor o reconhece por `t == "previewSub"` antes de tentar como comando.
- Enter executa só com o modo ligado; Enter da fileira de cima fica reservado nesse modo.
- Após Executar, o preparado é limpo.
- O motor sombra não tem GM/blackout/Mesa/Localizar/prévia (mostra a criação "pura").
- `Op::Staged` usa o campo `inner` (o `op` colide com a etiqueta do serde).
- (Tarefa 8) Ao ligar o modo com o preparado vazio, ele copia o ao vivo; limpo (Backspace/Descartar), fica vazio e só soma. Os sliders de nível e XY Pads reais entram na cópia (`widgetValues`).
- Executar de um preparado `full` (da cópia) substitui o ao vivo; `Executar` fica habilitado com `full` mesmo vazio (para parar tudo). Sem `full`:
- Executar soma ao ao vivo: as criações acesas continuam, as preparadas entram depois (vencem no LTP); valores de elemento e passos carregados do preparado substituem os do mesmo elemento/chaser. Um "Só esta" preparado não para as criações reais. O chaser preparado começa do passo 0 na luz real (a posição do sombra não passa).
- Botões de Blackout e Stop All apertados no modo Preparar agem na luz real; Flash, Temp e Macro preparados são ignorados (resposta OK, nada muda). Outras ops dentro de `Staged` são recusadas com erro.
- 3D: cada viewport é um `View3dStage` (duas cenas TresJS: custo dobrado só com o Preparar ligado; a qualidade segue a preferência). A câmera NÃO é compartilhada (cada viewport tem a sua; a vista 3D/topo/frente/lateral e a escolha valem para os dois). O preview não se edita nem escolhe pelo palco.
- 3D: a tela da 3D View tem o seu próprio botão "Preparar" (o mesmo `useState('consolePrepare')`), para ligar o preview também na janela externa (estado próprio); no painel do modular o estado é o do aparelho. A chave Dividido/Só preview só aparece com o modo ligado.
- O pânico, o Novo e o Abrir mandam `Cut` também ao sombra; o pânico não limpa o preparado (só Novo/Abrir).
- Tarefa 4: o "Descartar" (`clearStaged`) é um botão só com ícone (borracha) ao lado do Executar, visível só com o modo ligado. Ctrl+Enter vale com foco em campo (como o Alt); Enter simples só executa fora de campo, de diálogo e da edição. O passo do chaser preparado mostra 0 se o chaser real está parado (a sombra não publica o passo). Blackout/Stop All (botões) mandam direto, sem `staged`.

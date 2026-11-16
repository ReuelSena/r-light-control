# Seletor e Alvo (VC-7) — mini-plano

Pedido do usuário (2026-11-15, G4), inspirado na grandMA 3: o operador **seleciona um grupo ou subgrupo** de fixtures (um ou vários) e, a partir daí, escolhe **cor, efeito e animação** para o que está selecionado. Hoje há um botão de cor por equipamento; o objetivo é um seletor e botões que valem para a seleção.

Status: **entregue em 2026-11-15, aguardando a validação** (checklist no fim).

## Decisões do usuário
- Escolher uma cor nova **para a cor anterior** nas fixtures selecionadas (uma "categoria" por botão: cor, efeito e animação).
- A ação vale para **todas as fixtures** do grupo ou subgrupo selecionado.
- O **Seletor é um elemento do console** (o usuário o posiciona no layout).

## Ideia central
As criações que já existem (cenas de cor por modelo, EFX, Phaser, RGB Matrix) **não mudam**. O botão ganha o modo **"Alvo: seleção"**: ao tocar, o app manda ao servidor a criação mais a lista de fixtures selecionadas, e o servidor liga a criação **só nessas fixtures** (máscara). Nada de criações novas por grupo.

- **A seleção fica no aparelho** (`useState`, como o modo Preparar), e não no servidor. O servidor recebe a lista já resolvida (grupo/subgrupo → fixtures de agora). Assim, dois aparelhos não brigam pela seleção e o servidor não guarda estado de tela.
- Valores de cor continuam vindo das **cenas por grupo e exceções por fixture** (F10b): cada modelo recebe a sua cor, e o Strobo verde demais não volta.

## Contrato (a confirmar no estudo da rodada 1)
- `Project.selections: Vec<SelectionGroup { id, name, members: Vec<Member> }>`, com `Member` = fixture ou grupo de fixtures. É o **subgrupo**: pode sobrepor os outros (uma fixture em vários), ao contrário do grupo de fixtures (cada cabeça uma vez). Resolve na hora de tocar (N1). Projeto na **versão 21** (a migração só sobe a versão). Ops: `saveSelection`, `removeSelection`, `moveSelection` (arrastar, como os grupos).
- `Snapshot.targeted: Vec<TargetedRun { id, function, fixtures, category }>` (ao vivo, fora do projeto e do desfazer; zerado no Novo, Abrir, Stop All e pânico). `category`: Cor, Efeito ou Animação.
- Ops: `targetOn { function, fixtures, category }` e `targetOff { function, fixtures }`. O botão de Alvo é `Widget` com `target: bool` e `category`.
- Widget novo `Selector` (só na UI e no contrato do console): lista de grupos de fixtures e subgrupos, mais os atalhos Todos e Limpar.

## Regras
1. **Parar a cor anterior:** ao ligar uma criação com alvo na categoria C para as fixtures F, tira-se F de todas as outras execuções alvo da categoria C. A execução que fica sem fixtures para. O que sai faz o **fade out** da criação (sem corte seco; ver Risco 1).
2. **Sem categoria** ("Soma"): não tira nada; vale a mistura de sempre (HTP/LTP e prioridade).
3. **Toque em um botão já aceso na seleção:** desliga a criação nessas fixtures.
4. **LED do botão:** aceso quando a criação roda alvo em todas as fixtures da seleção atual; meio-aceso (âmbar) quando roda só em parte.
5. **Mistura, prioridade, tempos ("Tempos"), pausa e BPM** são os de sempre: a execução alvo é mais um `Run` do motor.
6. **Stop All e pânico** param também as execuções alvo.
7. **Seleção vazia:** o botão de Alvo não faz nada (avisa com um flash no Seletor).

## Máscara nas criações (rodada 2)
O motor não conhece fixtures: a máscara é aplicada na **tradução do servidor** (`program()` e `scene_values`, em `apply.rs`), e a execução alvo vira um `Run` com id próprio (`função@n`).
- **Cena:** só os valores (por grupo, por fixture e exceções) das fixtures da seleção entram.
- **EFX:** só as fixtures da seleção andam; a ordem, a propagação e a defasagem contam pela lista inteira (a seleção de 4 entre 12 anda como as 4 delas faziam).
- **Phaser e RGB Matrix:** o desenho e a onda continuam os do grupo inteiro; **só as saídas das fixtures selecionadas** passam (escolher 4 de 12 mostra o pedaço da animação que cai nelas). Pode pedir um filtro pequeno de cabeças no motor; confirmar no estudo.
- **Chaser e coleção:** a máscara desce para o que eles disparam.

## Rodadas
1. **Contrato e servidor:** `selections` (v21), `targeted`, `targetOn`/`targetOff`, a regra de parar a anterior, Stop All e pânico, testes em `apply.rs` e `hub.rs`.
2. **Máscara** em cena, EFX, Phaser, RGB Matrix, chaser e coleção; testes de cada.
3. **UI:** elemento Seletor (grupos e subgrupos, multi, Todos e Limpar), o modo Alvo e a categoria nas propriedades do botão, LED da seleção, i18n, protocolo e design system.
4. **Subgrupos e divisões:** editor de seleções em Fixtures (ao lado dos grupos, com arrastar e duplicar) e as **divisões do P3** da F12c no Seletor: metades, asas, pares e ímpares, uma por vez (anterior e próxima). _O P3 sai da F12c e entra aqui; o P2 e o P9 ficam lá._
5. **Preparar e exemplo (G8):** `targeted` também no `StagedState` (o preview mostra o alvo); exemplo pronto no `exemplo.json` e no "Show de exemplo" do usuário.

## Checklist de execução (aprovado em 2026-11-15; exceção ao G1: executado numa rodada só, a pedido do usuário)
Agentes: Opus 5.5 (médio) para as rodadas 1 e 2 (servidor e motor), Sonnet 5.5 (médio) para as de UI. Cada agente marca [x] e anota o estado ao terminar cada tarefa, para outro continuar daqui. `pnpm check` (ou `cargo test`, clippy e fmt; typecheck e lint da UI) deve passar a cada rodada. Nada de commit (G5).
- [x] 0. Aprovação e registro (DECISOES.md, PLANO.md e este arquivo).
- [x] 1. Contrato e servidor (Opus): `selections` (v21), `targeted`, `targetOn`/`targetOff`, regra de parar a anterior, Stop All, pânico, Novo/Abrir; TS regenerado; testes.
  - **Estado (2026-11-15):** feito. `rlc-core`: `Project.selections: Vec<SelectionGroup { id, name, members: Vec<Member> }>` (`Member::Fixture(Id) | Member::Group(Id)`, `#[serde(default)]`); `Snapshot.targeted: Vec<TargetedRun { id, function, fixtures, category: Option<Category> }>`; `Category { Color, Effect, Animation }`; `VcButton.target: bool` e `VcButton.category: Option<Category>` (`#[serde(default)]`); widget `VcWidgetKind::Selector(VcSelector { groups, selections })`; ops `SaveSelection { selection }`, `RemoveSelection { id }`, `MoveSelection { id, to }`, `TargetOn { function, fixtures, category }`, `TargetOff { function, fixtures }` (as duas de alvo em `is_live`). Tópicos `project.selections` (desfaz) e `targeted` (ao vivo). TS regenerado.
  - `apply.rs`: regras do `TargetOn`/`TargetOff`; Stop All, pânico, Novo (e Abrir) zeram `targeted`; `removeFixtures` limpa `selections` e `targeted`; `removeFixtureGroup` congela o grupo nas seleções (vira as fixtures dele) e o tira dos Seletores; `removeSelection` tira dos Seletores; `removeFunction` tira as execuções alvo dela. O `pressWidget` num botão com `target` e ação `function` não faz nada (o aparelho manda `targetOn`). **Ponto de extensão da rodada 2:** comentário em `running_programs` (cada `TargetedRun` vira um `Run` com `id` = `TargetedRun::id` e a máscara); `Topic::Targeted` já está na lista `running` do `sync_engine` (hub.rs). O `staged` recusa `targetOn` (rodada 5).
  - **Desvio:** projeto na **versão 22**, não 21 (a 21 já era a dos subgrupos de fixtures, `FixtureGroup.parent`). A migração só sobe a versão.
  - UI: só o mínimo para compilar (`ConsoleScreen.vue`: padrões do botão e do Seletor; `utils/console.ts`: `selector` em `widgetKinds`, fora de `widgetTypes` até a rodada 3).
  - Testes: 5 em `apply.rs`, 1 em `hub.rs`, 1 em `project.rs` (v21 → v22). `cargo fmt`, `clippy -D warnings`, `cargo test`, typecheck e lint da UI passam.
- [x] 2. Máscara nas criações (Opus): cena, EFX, Phaser, RGB Matrix, chaser, coleção; fade out sem corte (Risco 1); testes.
  - **Estado (2026-11-15):** feito. `running_programs` traduz cada `TargetedRun` num `Run` com `id` = `TargetedRun::id`, depois das criações rodando e antes dos sliders; Flash, slider de playback, "Tempos", velocidade, intensidade, prioridade e pausa são os da criação (`function`). A máscara vai no `Play` (`mask: Option<&[Id]>`, com `lit(fixture)`) e desce pelo `program()` recursivo (chaser, sequência, coleção). Cena e passo de sequência: `scene_values`/`step.values` filtrados. EFX: a fixture fora da máscara fica na lista com `heads = [None, None]` (ordem, propagação e defasagem iguais). Phaser: `phaser_targets(…, mask)` põe o canal 0 (que o motor pula, `is_valid`) no lugar da fixture fora da máscara, e o índice na onda não muda. RGB Matrix: `matrix_program(…, mask)`/`matrix_cells(…, mask)` só com as células das fixtures da máscara, na mesma grade (largura e altura do grupo).
  - **Risco 1, no motor (`Ctx::update`, cena):** o canal que sai de uma cena rodando passa a um fader parado logo acima dela (mesmo dono, prioridade, tempos e intensidade), que faz o fade out e sai sozinho no fim; a cor nova, iniciada depois, fica por cima e parte do que está saindo. O servidor não precisa de execução "saindo": a execução que encolhe guarda o id. EFX já fazia o fade out da fixture retirada; Phaser e RGB Matrix recomeçam os faders (o que sai faz o fade out, o que fica entra partindo do valor de agora; o Phaser segue o ciclo, o RGB Matrix volta ao primeiro passo).
  - Testes: motor `a_channel_that_leaves_a_running_scene_fades_out_under_the_new_one`; servidor `a_targeted_scene_lights_only_its_fixtures`, `a_targeted_efx_moves_only_its_fixtures_in_their_place`, `a_targeted_phaser_and_matrix_keep_the_whole_group_and_light_only_their_fixtures`, `the_mask_goes_down_to_what_chasers_and_collections_light`, `a_color_that_loses_fixtures_keeps_its_run_and_the_new_one_comes_after`, `a_targeted_run_follows_the_function_controls`. `Topic::Targeted` já reenvia o `running` (hub); Stop All tira as execuções (fade out) e o pânico manda `Cut` (na hora). `cargo fmt`, clippy, `cargo test` e `pnpm check` passam.
  - **Para a rodada 5:** o motor sombra (`swap_staged`) ainda vê o `targeted` ao vivo, porque o `StagedState` não o tem.
- [x] 3. UI (Sonnet): elemento Seletor, modo Alvo e categoria no botão, LED, seleção no aparelho, i18n, protocolo.
  - **Estado (2026-11-15):** feito, `pnpm check` passa; **não verificado no navegador**. `composables/useSelection.ts` (seleção do aparelho em `useState`, resolve grupo e seleção salva para as fixtures de agora, `flash` para o Seletor piscar); `useConsole.ts` (`isTarget`, `targetState` on/part/off a partir de `snapshot.targeted`, toque de alvo manda `targetOn`/`targetOff`; também vale para tecla e MIDI, que passam por `press`); `ConsoleWidget.vue` (Seletor com botões LED, Todos, Limpar, contagem; LED do alvo cheio ou amarelo); `ConsoleProperties.vue` (chave "Alvo: seleção" + categoria no botão de ação "Rodar criação", lista de grupos e seleções salvas no Seletor); `utils/console.ts` (`selector` na paleta); i18n, `design-system.md`. `protocolo.md` já estava em dia.
- [x] 4. Subgrupos e divisões P3 (Sonnet): editor de seleções em Fixtures; metades, asas, pares/ímpares, uma por vez.
  - **Estado (2026-11-15):** feito, `pnpm check` passa; **não verificado no navegador**. `FixturesSelection.vue` (nome, duplicar "… (cópia)", remover com confirmação, membros por botões: grupos de fixtures em árvore e fixtures) e, em `FixturesGroups.vue`, a seção "Seleções" abaixo dos grupos de canais (criar, reordenar arrastando com `RlList`). Divisões: contas puras em `utils/selection.ts` (`divide`, `splitSteps`, conferidas à mão no node, sem framework de teste de UI), estado em `useSelection.ts` (`split`, `walk`, `restore`; `full` guarda a seleção inteira) e barra "Dividir" no Seletor (`ConsoleWidget.vue`). Contrato: `VcSelector.splits: bool` (padrão ligado, também nos arquivos de antes), chave "Barra de divisões" nas propriedades do Seletor.
- [x] 5. Preparar e exemplo G8 (Sonnet): `targeted` no `StagedState`; exemplo no `exemplo.json` e no "Show de exemplo" do usuário (cópia dos arquivos antes).
  - **Estado (2026-11-15):** feito. `StagedState.targeted` (TS regenerado); `swap_staged` troca também o `targeted`, então o `targetOn`/`targetOff` preparados valem sobre o preparado (a mesma regra, agora em `target_on`, `apply.rs`) e o motor sombra os roda. `SeedStaged` copia o `targeted`; `CommitStaged` o substitui com `full` (mesmos ids, sem reiniciar) e, sem `full`, soma cada execução por `target_on` (tira a anterior da categoria). Descartar, Novo e Abrir já zeravam o preparado inteiro. UI: o toque de alvo vai por `op` (embrulhado em `staged` no Preparar), o LED lê o `view` (preparado ou ao vivo), `hasStaged` e o destaque `staged(w)` contam o `targeted`. Testes em `hub.rs`: `a_staged_target_waits_for_the_commit_and_then_replaces_the_previous_color` e `a_seeded_staged_copies_the_targets_and_its_commit_replaces_them`.
  - **Exemplo:** pasta "Seletor" (cena, Phaser e RGB Matrix): cenas Vermelho, Azul, Verde e Âmbar com valores por modelo (ParLeds `g4` e PARs RGBW da frente `g1`; o Âmbar usa o canal âmbar do ParLed e vermelho com verde no PAR), "Onda de dimmer" (Phaser, prioridade Alta, para mandar no dimmer da cor) e "Onda ciano" (RGB Matrix com o script "Onda", Alta). O subgrupo é o "Esquerda" dos ParLeds (`g7`, já existia), e a seleção salva "Pontas + PARs frente" mistura fixtures e grupo. Página "Seleção" do console: o Seletor (ParLeds, Esquerda, Direita, PARs frente e a seleção) e os botões de alvo das três categorias. O `exemplo.json` foi regravado com `JSON.stringify` (indentação de 1): alguns trechos escritos à mão ficaram no formato normal. O "Show de exemplo" salvo (`R:\Documentos\R Light Control\`, da versão 14) ganhou o mesmo, mais os subgrupos `g7` e `g8`; cópia de antes em `Show de exemplo.rlc.bak-seletor`, na mesma pasta. A recuperação era do projeto "Palco" e não foi mexida. O teste de migração para a 22 tira do exemplo o que veio nela.
- [x] 6. Registro final: PLANO.md, DECISOES.md, `pnpm check`, decisões tomadas sem perguntar (abaixo).
  - **Estado (2026-11-15):** `pnpm check` passa. Verificado no navegador (Edge, contra a UI do `pnpm dev` na porta 3000 ligada a um servidor só na memória na porta 8095, sem tocar o app aberto nem as pastas do usuário): Esquerda + Vermelho acende só os ParLeds 1–6; Azul na mesma seleção tira o Vermelho; Direita + Verde soma; com os 12 selecionados, Azul e Verde ficam amarelos (parcial); Onda de dimmer e Onda ciano somam (outras categorias) e o segundo toque desliga; Metade 1 = 6, Asas = 2, Próxima anda, Ímpares = 6, Tudo = 12, a seleção salva soma; Âmbar nas pontas e PARs tira as pontas do Azul e do Verde; Preparar: o Vermelho vai só para o preparado (botão aceso com o tracejado do Preparar), e o Executar (a cópia do ao vivo, `full`) o leva à luz tirando dos ParLeds o Âmbar, o Azul e o Verde (o Âmbar fica nos PARs); Num + (Stop All) zera o ao vivo (o preparado não muda, como no VC-6). O "Show de exemplo" salvo também abriu (numa cópia) e acendeu o alvo. Não verificado: a luz de verdade e a 3D View (só o estado do servidor), o pânico de dois toques e o toque no celular.

## Decisões tomadas sem perguntar (informar no fim)
- Seleção por aparelho (a resposta do usuário não cobriu essa pergunta).
- Rodada 1: o projeto vai para a **versão 22** (a 21 já é a dos subgrupos de fixtures).
- Rodada 1: `targetOn` da mesma criação na mesma categoria não amplia a execução que já existe: cria outra só com as fixtures que faltam. A máscara de uma execução nunca cresce (só encolhe), para cada execução entrar com o próprio fade in; o encolher é o Risco 1, da rodada 2.
- Rodada 1: a categoria só tira fixtures de execuções de **outras** criações; tocar de novo a mesma criação acesa é o `targetOff` (o aparelho decide pelo LED).
- Rodada 1: fixtures que não existem são ignoradas no `targetOn` (sem recusa); seleção vazia não muda nada. Criação que não existe é recusada.
- Rodada 1: remover um grupo de fixtures **congela** as seleções salvas que o usavam (ficam as fixtures dele), como nos outros lugares (F10b). Remover a criação tira as execuções alvo dela.
- Rodada 1: o `pressWidget` (tecla, MIDI) num botão de alvo não faz nada, porque o servidor não conhece a seleção do aparelho.
- Rodada 1: o Seletor guarda `groups` e `selections` em duas listas (como o XY Pad com `fixtures` e `groups`), sem ordem misturada.
- Rodada 2: a máscara fica no servidor; o motor só ganhou o fade out do canal que sai de uma cena rodando (Risco 1). Isso vale também ao editar uma cena rodando e tirar um valor (antes cortava): ajuste registrado em A2 do `DECISOES.md`.
- Rodada 2: Phaser fora da máscara = canal 0 no lugar da fixture (o motor já pula canais inválidos), sem tipo novo no motor.
- Rodada 2: Phaser e RGB Matrix que perdem fixtures recomeçam os faders sem corte; o RGB Matrix volta ao primeiro passo nas fixtures que ficam (manter o passo pediria mais código no motor).
- Rodada 2: pausa, intensidade, prioridade, "Tempos", velocidade e Flash da execução alvo são os da criação (a pausa de uma criação pausa também as execuções alvo dela).
- Rodada 3: tocar num grupo do Seletor soma as fixtures dele; se todas já estão na seleção, tira (o LED do botão é "todas dentro"). Todos seleciona todas as fixtures do projeto.
- Rodada 3: o modo (liga/desliga, Flash, Temp) some das propriedades do botão de alvo, que só tem toque; "Alvo" só aparece na ação "Rodar criação" (não em "Só esta").
- Rodada 3: o LED de alvo usa o `targeted` ao vivo (não o preparado); com Preparar ligado o toque de alvo é ignorado sem erro. O Seletor novo nasce sem grupos (aviso na caixa); as seleções salvas só aparecem nas propriedades quando existirem (editor na rodada 4).
- Rodada 4: as divisões partem da seleção inteira (a guardada, se já dividida) na ordem das fixtures do projeto; Ímpares e Pares contam pela posição (1ª, 3ª… e 2ª, 4ª…); Metade 1 fica com a fixture do meio quando o total é ímpar. Asas e "Uma por vez" andam com Anterior e Próxima (que dão a volta); sem divisão ativa, Anterior e Próxima começam em "Uma por vez". Escolher grupos à mão, Todos ou Limpar acaba a divisão; "Tudo" restaura a inteira. A divisão é do aparelho, como a seleção.
- Rodada 5: no Preparar, o alvo segue o mesmo caminho das outras ops (vale sobre o preparado e só vai à luz no Executar); o Stop All de um botão continua só na luz real e não mexe no alvo preparado.
- Rodada 5: o Executar sem cópia soma as execuções alvo preparadas uma a uma pela regra da categoria; com a cópia, substitui (as execuções que já estavam ficam com o mesmo id, sem reiniciar).
- Rodada 5: no exemplo, o subgrupo é o "Esquerda" que já existia (filho de "ParLeds"), e não um grupo novo "ParLeds esquerda"; os efeitos usam prioridade Alta para aparecer sobre a cor (o dimmer da cor está cheio).
- Rodada 4: o editor de seleções vive na aba Grupos (seção abaixo dos grupos de canais), não numa aba nova.
- **Para o usuário decidir:** os subgrupos de fixtures (`FixtureGroup.parent`, entregues no mesmo dia) já cobrem boa parte do "subgrupo" deste plano, e grupos diferentes já podem repetir fixtures. As `selections` só acrescentam misturar fixtures e grupos sem grade. Se não fizerem falta, saem na rodada 4 (G7).

## Riscos
1. **Corte ao trocar a cor em parte das fixtures:** o motor corta quando o programa de um `Run` encolhe. A solução é mover as fixtures retiradas para uma execução "saindo", parada pelo caminho normal (com fade out), e ligar a nova por cima. Validar no teste da rodada 2.
2. **Prévia e Localizar** continuam por cima de tudo (como hoje); a execução alvo fica abaixo.
3. **Máscara no Matrix/Phaser** pode exigir mexer no motor (o resto é só no servidor).

## Exemplo (G8)
Pasta "Seletor" no show de exemplo: o grupo "ParLeds", o subgrupo "ParLeds esquerda", cenas de cor por modelo, um Phaser, um RGB Matrix e a página "Seleção" do console com o Seletor e os botões de cor, efeito e animação (categorias diferentes).

## Validação com você
Abra o "Show de exemplo", Console → página "Seleção": o Seletor (ParLeds, Esquerda, Direita, PARs frente e "Pontas + PARs frente") e os botões Vermelho, Azul, Verde e Âmbar (Cor), Onda de dimmer (Efeito) e Onda ciano (Animação). As criações ficam nas pastas "Seletor" do Studio.
- [ ] Selecionar um grupo e tocar numa cor: só as fixtures dele mudam.
- [ ] Selecionar um subgrupo e tocar outra cor: só ele muda, o resto fica, sem corte.
- [ ] Trocar a cor na mesma seleção: a anterior sai (fade out), a nova entra.
- [ ] Cor, efeito e animação ao mesmo tempo na mesma seleção (categorias diferentes).
- [ ] Selecionar dois grupos juntos e aplicar.
- [ ] O LED do botão acompanha a seleção (cheio, parcial e apagado).
- [ ] Stop All e pânico param tudo.
- [ ] As divisões (metades, asas, uma por vez) percorrem a seleção.
- [ ] Preparar: a seleção e o alvo aparecem só no preview até o Executar.
- [ ] Efeitos de pan e tilt: na [validação final](../validacao-final.md), com os Beams.

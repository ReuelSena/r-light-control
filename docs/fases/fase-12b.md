# F12b — Gravação (P5 e P7)

**Status:** entregue em 2026-11-13, aguardando a validação (junto com a F9, a F9b, a F10c, a F10d e a F11, exceção ao G1 a pedido do usuário). **As paletas (P1) foram tentadas e removidas inteiras no mesmo dia (2026-11-13, pedido do usuário):** não fizeram sentido na prática. A diferença de cor entre fixtures (o Strobo com vermelho e verde cheios fica muito verde; o ParLed tem âmbar intenso) se resolve com cenas de cada cor para cada fixture.

## Objetivo

- **P5, gravar a saída e "Update":** gravar numa cena o que está saindo na luz agora; gravar de volta numa cena os ajustes feitos na Mesa simples.
- **P7, filtros de atributo:** escolher os tipos de atributo (Intensidade, Cor, Posição, Efeitos) ao gravar e no Update.

## Decisões

Do usuário (2026-11-13):
1. **A captura grava os canais controlados:** só os que alguma coisa controla agora (criações, prévia, Mesa simples), sem os que estão no valor padrão. O Localizar, o Grand Master e o blackout ficam de fora.
2. **Update com a cena escolhida:** o Update pergunta em qual cena gravar.
3. **Sem paletas (P1):** removidas inteiras (decisão do usuário, 2026-11-13). Projeto na versão 16: os campos das paletas das versões 14 e 15 são ignorados ao abrir.

Recomendações seguidas (para o usuário ajustar):
4. **Gravar a saída:** botão "Gravar saída" no Studio, ao lado de "Parar todas", com o destino (cena nova ou uma existente), o modo (Sobrescrever: tira da cena os tipos do filtro antes de gravar; Mesclar: grava por cima) e o filtro. Os valores entram como valores próprios das fixtures. Os atributos sem tipo (velocidade, manutenção…) só entram com o filtro "Tudo".
5. **Update:** botão na Mesa simples (aparece quando ela segura canais). Escolhe-se a cena (as rodando primeiro) e o filtro; os canais da Mesa simples das fixtures entram na cena como valores próprios, e a Mesa simples solta os canais gravados. Canais sem fixture ficam na Mesa.

## Contrato (rlc-core)

```rust
Op::CaptureOutput { target: CaptureTarget (New { id, name } | Scene { id }), merge: bool, kinds: Vec<AttributeKind> }   // kinds vazio = tudo
Op::UpdateScene { function: Id, kinds: Vec<AttributeKind> }
```

A captura pede /ao motor (comando com resposta) o quadro antes do Localizar, do Grand Master e do blackout, com a máscara dos canais controlados (faders das criações, prévia, relativo e Mesa simples).

## Exemplos (G8)

Sem exemplo no `exemplo.json`: a gravação é uma ação (botão "Gravar saída" e "Update"), e não uma criação.

## Checklist de execução

- [x] 1. Perguntas ao usuário e mini-plano.
- [x] 2. Backend da gravação: comando de captura no motor, `CaptureOutput` e `UpdateScene` no servidor, filtro por tipo, testes.
- [x] 3. UI da gravação: "Gravar saída" no Studio e "Update" na Mesa simples, i18n.
- [x] 4. Documentação.
- [x] 5. Paletas (P1) removidas (2026-11-13), `pnpm check` passando.

## Registro de progresso

- 2026-11-13: perguntas respondidas; mini-plano escrito.
- 2026-11-13: item 3 (backend da gravação). Motor: `Command::Capture(Sender<Vec<Capture>>)`, respondido no fim do próximo tick com o quadro de antes do Localizar, do Grand Master e do blackout (`unlit`, guardado a cada tick) e a máscara dos canais controlados (faders acesos, sem os que apagam ou esperam o delay de entrada; prévia; Mesa simples; os relativos já são faders). Com o padrão de teste, nada é capturado. Servidor: o hub pede a captura (espera até 1 s) e a converte, em `captured_scene` (`apply.rs`), num `saveFunction` (validação e desfazer de sempre); `updateScene` fica no `apply` (a cena entra no desfazer, a Mesa simples não). Decisões da implementação: os dois recusam quando não há nada para gravar ("Nada controla as fixtures desses tipos agora." / "A Mesa simples não segura canais de fixtures desses tipos."); o Sobrescrever mantém os grupos da cena (só tira os valores dos tipos do filtro); o Update solta também o nível do grupo de canais da Mesa cujos canais saíram todos; a cena nova nasce sem pasta. `pnpm check` passa.
- 2026-11-13: item 5 (UI da gravação). "Gravar saída" no Studio (botão com ícone ao lado de "Parar todas"; `StudioCapture.vue`: cena nova com nome padrão "Gravação n" ou cena existente, Sobrescrever/Mesclar só com cena existente, filtro; escolhe a cena criada) e "Update" na Mesa simples (`StudioDeskUpdate.vue`, botão só quando a Mesa segura níveis ou grupos; cenas rodando primeiro e marcadas, a primeira rodando já vem escolhida). O filtro de tipos é o componente `StudioKindFilter.vue`, dos dois diálogos (nenhum ligado = Tudo). O erro do servidor aparece no diálogo. Typecheck e lint da UI passam, sem teste no navegador.
- 2026-11-13: as paletas (P1) foram implementadas (versões 14 e 15) e removidas por inteiro a pedido do usuário. Saíram `Palette`, `PaletteModel`, `FixturePalette`, `Project.palettes`, `SceneGroup.palettes`, `Scene.fixturePalettes`, as operações `savePalette` e `removePalette`, a prévia de paleta, `StudioPalette.vue`, a seção na lista do Studio, a fileira "Paletas" no editor de cena e a pasta "Paletas" do `exemplo.json`. Ficaram a captura, o Update e o filtro. O "Show de exemplo" do usuário perdeu a pasta "Paletas" (com cópia antes); a recuperação não foi mexida e abre pela versão 16. Projeto na versão 16, sem migração de dados (o serde ignora os campos).

## Validação com você

- [ ] "Gravar saída" com duas cenas rodando e a Mesa simples: a cena nova acende igual; com o filtro "Cor", só as cores.
- [ ] "Update": um ajuste na Mesa simples gravado na cena escolhida, e a Mesa simples solta os canais.
- [ ] Projetos antigos abrem (versões 13, 14 e 15 → 16).

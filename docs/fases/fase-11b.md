# F11b — Coleção com linha do tempo

**Status:** 🟡 entregue em 2026-11-13, aguardando a validação (iniciada no mesmo dia). Pedido do usuário VC-5 (G4). O usuário respondeu as perguntas e mandou executar a fase inteira, sem parar para aprovar o mini-plano (exceção ao G1); as decisões abaixo são as recomendadas, para ele ajustar na entrega.

## Objetivo

A coleção vira uma linha do tempo: cada item (criação) tem **início**, **duração**, **camada** e **prioridade**; a coleção tem modo **Laço**, **Uma vez** ou **Vai e volta**. Para a operação ao vivo; mais enxuta que o Show/timeline com áudio (fora da v1). O editor é visual, como o de um editor de vídeo: trilhas, clipes que se arrastam e redimensionam, régua, zoom, encaixe e a posição ao vivo.

## Decisões (do usuário, 2026-11-13)

1. **Unidade:** segundos e batidas, com a chave "Seguir o BPM" (ligada por padrão, como no elemento "Tempos"): régua e encaixe em batidas; desligada, em segundos.
2. **Vai e volta:** a ordem dos clipes inverte na volta (os inícios espelham), e cada criação toca para a frente.
3. **Prioridade por item** (Baixa, Normal, Alta ou "da criação"); a macro (E6) continua separada.
4. **Etapas 1 e 2 juntas:** motor, lista e editor visual na mesma fase.

## Decisões de projeto (recomendadas, para ajustar)

5. **Compatível com o que existe:** a migração (versão 17) transforma cada `functions[i]` em item com início 0, sem duração (dura até a coleção parar), camada = posição na lista, prioridade "da criação"; modo Laço. Uma coleção antiga acende e roda exatamente como antes.
6. **Duração vazia = até parar.** Itens sem duração nunca terminam sozinhos; com eles não há laço nem "Uma vez" terminando (a coleção só acaba quando o usuário a para). O comprimento do laço é o fim do último item com duração (ou o campo `length`, se preenchido).
7. **Camada é a trilha visual** (linha do editor); a prioridade é o campo que decide a mistura. O editor mostra a prioridade por cor/etiqueta no clipe.
8. **Item termina com o fade out da criação** (Stop normal, como o Stop All); entra com o fade in dela. Os tempos do elemento "Tempos" valem como hoje.
9. **Sem arrastar o cursor** para ver a luz (o motor guarda estado); a posição ao vivo (playhead) vem do motor.
10. **Edição no desktop, operação no celular:** no toque, o editor funciona, mas o arrastar de clipes é pensado para o mouse; a lista de campos numéricos (Etapa 1) fica sempre disponível sob a linha do tempo.

## Contrato (rlc-core) — proposta inicial (a versão final está em "Decisões da implementação")

```rust
pub struct Collection {
    pub items: Vec<TimelineItem>,        // substitui `functions`
    pub mode: TimelineMode,              // Loop | Once | PingPong
    pub beats: bool,                     // Seguir o BPM (padrão true; início e duração são Span em batidas)
    pub length: Option<Span>,            // comprimento do laço; None = fim do último item com duração
}
pub struct TimelineItem {
    pub function: Id,
    pub start: Span,                     // a partir do começo da coleção
    pub duration: Option<Span>,          // None = até a coleção parar
    pub layer: u8,                       // trilha visual
    pub priority: Option<Priority>,      // None = a da criação
}
```

Motor: `Program::Collection` vira (ou ganha) um programa de linha do tempo com `Node::Timeline`, que liga cada item no início e o para no fim (fade out da criação), em segundos ou batidas (segue o BPM master e o `shift` como o chaser), com pausa, e publica a posição para o playhead (`timelineClock`, como `chaserSteps`).

## Exemplos (G8)

No `exemplo.json` e no "Show de exemplo" do usuário, pasta "Linha do tempo": uma coleção com três itens em tempos diferentes e sobrepostos, com camadas de prioridades diferentes, em Laço; e uma em "Vai e volta". Os valores exatos ficam registrados aqui na entrega.

## Checklist de execução (atualizar a cada tarefa; outro agente pode continuar daqui)

- [x] 1. Perguntas ao usuário e este mini-plano.
- [x] 2. Backend (agente Opus): contrato `rlc-core` + migração v17 + `cargo test -p rlc-core` regenera os tipos TS.
- [x] 3. Backend: motor (`Program`/`Node` de linha do tempo, laço, uma vez, vai e volta, prioridade por item, pausa, BPM, posição ao vivo) + testes.
- [x] 4. Backend: servidor (`apply.rs`: tradução, validação, remover criação tira itens, não conter a si mesma; evento da posição; `docs/protocolo.md`) + testes.
- [x] 5. UI (agente Sonnet): `StudioCollection.vue` com linha do tempo (régua, trilhas, clipes arrastáveis e redimensionáveis, zoom, encaixe nas batidas, playhead), lista de campos, i18n, `useEngine`. Typecheck e lint passam; sem teste visual no navegador ainda.
- [x] 6. Exemplos (G8) no `exemplo.json` e no "Show de exemplo" salvo (com a cópia `.rlc.bak-f11b`).
- [x] 7. Documentação: `DECISOES.md` (decisão, versão 17, registro), `PLANO.md`, `CLAUDE.md` (estrutura), `docs/design-system.md` se houver componente novo.
- [x] 8. `pnpm check` passando; checklist de validação para o usuário abaixo.

## Validação com você

- [ ] Uma coleção com três itens em tempos diferentes, sobrepostos, com camadas de prioridades diferentes.
- [ ] Laço, Uma vez e Vai e volta.
- [ ] A coleção disparada por um botão do console, pelo "Tempos" e pelo BPM.
- [ ] Arrastar e redimensionar clipes, zoom e encaixe nas batidas; o cursor mostra a posição ao vivo.
- [ ] Uma coleção antiga abre e acende como antes.

## Decisões da implementação

**Contrato final (backend, 2026-11-13; versão 17 do projeto).** Tipos gerados em `ui/app/types/generated/` (`Collection`, `TimelineItem`, `TimelinePosition`, `Snapshot`):

```rust
pub struct Collection {
    pub items: Vec<TimelineItem>,
    pub run_order: RunOrder,       // loop | singleShot ("Uma vez") | pingPong; random é recusado
    pub beats: bool,               // "Seguir o BPM": tempos em milésimos de batida; senão, ms
    pub length: Option<u32>,       // None = fim do último item com duração
}
pub struct TimelineItem {
    pub function: Id,
    pub start: u32,                // na unidade da coleção
    pub duration: Option<u32>,     // None = até a coleção parar
    pub layer: u8,                 // trilha visual
    pub priority: Option<Priority>,// None = a da coleção
}
// No Snapshot: timeline_clock (`timelineClock`), evento `timelineClock`
pub struct TimelinePosition { pub collection: Id, pub position: u32 }
```

1. **Sem o tipo `Span` no contrato:** o `rlc-core` não tem `Span` (é do motor); segui o padrão do chaser e do Phaser: números `u32` mais a chave `beats` (milésimos de batida ou ms).
2. **`runOrder` no lugar de `mode`:** reusa o `RunOrder` (como o EFX e o RGB Matrix): `loop`, `singleShot` = "Uma vez", `pingPong` = "Vai e volta"; o `random` é recusado no `saveFunction`.
3. **Itens sem duração (muda a decisão 6):** acendem uma vez, no início deles (contado do começo da coleção), e ficam até a coleção parar, sem se repetir com a passada; os itens com duração continuam em laço por baixo deles (ex.: uma cena de base sem duração e efeitos em laço). Sem nenhum item com duração (nem `length`), não há passada: é a coleção de antes.
4. **Passada:** vai até `length` ou até o fim do último item com duração; o item que passa do fim é cortado nele, e o que começa depois não acende. Um item que cobre a passada inteira recomeça a cada passada (o clipe toca de novo, como um chaser Single Shot dentro dele).
5. **"Uma vez" termina** quando a passada acabou e os itens sem duração terminaram sozinhos (o Single Shot de dentro). Uma coleção antiga (tudo sem duração) termina como antes: quando tudo de dentro terminou.
6. **Vai e volta:** na passada de volta, a janela de cada item é `[length − fim, length − início)`; cada criação toca para a frente.
7. **Em batidas,** o começo da coleção cai na grade da batida (como o primeiro passo do chaser), para os itens trocarem nas batidas; o BPM e o fader de BPM valem na hora. A velocidade (Rate) só muda os tempos em ms.
8. **Prioridade do item:** vale para tudo o que ele acende (também os passos de um chaser dentro dele) e não segue a da coleção quando ela muda (fader `pinned` no motor). Mudar a prioridade de um item aceso o recomeça.
9. **Edição rodando:** os itens são casados pela posição na lista; o aceso recebe os valores na hora; janelas novas valem no quadro seguinte; o item que saiu apaga com o fade out.
10. **Posição ao vivo:** `timelineClock` no `Snapshot` e o evento `timelineClock` (`[{ collection, position }]`), só das coleções da lista `running`. Na unidade da coleção, dentro da passada; no Vai e volta, volta do fim para o começo; no fim do "Uma vez", fica no fim; sem passada, para no último início (a coleção antiga fica em 0 e não gera eventos). Muda a cada quadro enquanto anda: o servidor confere a cada 40 ms (até 25 eventos/s por coleção rodando).
11. **Validação:** criações que existem (a mesma pode aparecer mais de uma vez, em tempos diferentes), duração e `length` maiores que zero, sem conter a si mesma. Remover uma criação tira os itens dela.
12. **Motor:** `Program::Timeline` / `Node::Timeline` (`crates/rlc-engine/src/timeline.rs`); o `Program::Collection` continua para a cena dividida por tipo de atributo. No passo de um chaser, a entrada conta só os itens que começam no zero.
13. **UI mínima (para o typecheck):** `StudioCollection.vue` continua a lista simples (adicionar põe o item no zero, sem duração, numa trilha nova); `contains` em `utils/functions.ts` e a coleção nova do `StudioScreen.vue` usam `items`. O `exemplo.json` só teve a "Abertura" (k1) migrada.

**Editor visual (UI, 2026-11-13).** `StudioCollection.vue`:

14. **Unidade da régua:** início, duração e comprimento já são milésimos da unidade (ms ou de batida), então 1000 é uma unidade inteira da régua nos dois casos. O encaixe vai às unidades inteiras; Alt solta o encaixe (passo de 10).
15. **Arraste:** eventos de ponteiro na janela (mouse e toque); o rascunho fica local e a operação `saveFunction` sai ao soltar. Mover muda o início (horizontal) e a trilha (vertical); a borda direita muda a duração e a esquerda, início e duração juntos (num clipe "até parar", só o início). Zoom por −/+ e Ctrl + roda (20 a 180 px por unidade).
16. **Clipe e prioridade:** a cor do clipe é a da prioridade (Baixa azul, Normal âmbar, Alta rosa); "Da coleção" usa a prioridade da própria coleção, como no contrato. Clipe sem duração se estende até a borda, com "∞→".
17. **Comprimento:** a linha do laço é arrastada pela alça na régua e tem campo ao lado; cinza = automático (fim do último clipe com duração), laranja = fixado; "Automático" volta ao automático.
18. **Lista sob a linha do tempo:** cada clipe tem campos de início, duração (com ∞), trilha; o escolhido mostra também a prioridade, duplicar e remover. Serve no celular, onde arrastar clipes é ruim.
19. **Limitações:** sem arrastar o cursor, sem desfazer próprio do arraste (usa o Ctrl + Z do servidor), sem seleção múltipla; novo clipe entra no zero, sem duração, em trilha nova.

**Exemplos (G8, 2026-11-13).** Pasta "Linha do tempo" (`d6`, tipo coleção) no `exemplo.json` e no "Show de exemplo" salvo. Unidade em batidas (`beats` ligado; 1000 = 1 batida). Abrir e observar a luz com o cursor andando:

1. **"Laço em camadas"** (`k2`, Laço, comprimento fixado em 16 batidas):
   - `s10` Base dos ParLeds: início 0, sem duração (acende uma vez e fica), trilha 0, prioridade Baixa;
   - `ph1` Onda de dimmer nos ParLeds: início 0, duração 8, trilha 1, prioridade da coleção (Normal);
   - `ph3` Arco-íris nos ParLeds: início 4, duração 8, trilha 2, prioridade Alta (sobrepõe a onda entre 4 e 8).
2. **"Vai e volta em camadas"** (`k3`, Vai e volta, comprimento automático = 12 batidas):
   - `c1` Troca de cores: início 0, duração 6, trilha 0, prioridade da coleção;
   - `e1` Círculo nos MHs: início 4, duração 8, trilha 1, prioridade da coleção;
   - `ph2` Pulso nos ParLeds: início 8, duração 4, trilha 2, prioridade Alta. Na volta a ordem inverte: o pulso entra primeiro e a troca de cores por último.

No "Show de exemplo" do usuário (`DocumentosR Light ControlShow de exemplo.rlc`, versão 14 no disco, migra ao abrir) as duas coleções e a pasta foram acrescentadas direto no JSON, com a cópia `Show de exemplo.rlc.bak-f11b` ao lado. A recuperação (`%APPDATA%-light-controlecuperacao.rlc`) estava como "Novo projeto" vazio e não foi mexida.

# F2 — Protótipo navegável de todas as telas

**Status:** aprovado em 2026-10-16 (E11–E13). Rodada 1 aprovada em 2026-10-16, depois dos ajustes da validação. Rodada 2 aprovada em 2026-10-16, depois dos ajustes da validação. Rodada 3 aprovada em 2026-10-16, depois dos ajustes da validação. Fase concluída e validada em 2026-10-17, com as telas congeladas em [`docs/telas/`](../telas/).

**Rodada 3 entregue**
- **Console, operação:** páginas em abas e os elementos botão, slider (nível em grupos de canais, playback ou submaster; fader ou knob), cue list, XY pad, Tempos (VC-3), moldura (com opção solo), relógio (hora, cronômetro ou contagem), animation (RGB Matrix) e texto. Seção master fixa: Grand Master, Blackout, Stop All e BPM com tap. As teclas definidas nos elementos funcionam.
- **Console, edição:**
  - paleta (o toque adiciona no primeiro lugar livre) e grade com encaixe;
  - mover e redimensionar com mouse e toque (no toque, depois de escolher), destino em verde ou vermelho, entrar e sair de molduras;
  - multisseleção (Ctrl + clique ou toque longo), alinhamento, duplicar e remover;
  - propriedades (VC-1): texto, cor, ação (rodar criação, blackout geral, apagar fixtures ou grupos, parar tudo), modo, destaque e as entradas externas (MIDI com "aprender", simulado, e tecla);
  - páginas: nova, renomear e excluir.
- **VC-2:** o formato em edição aparece na barra; "Tirar deste formato", a lista dos elementos fora do formato e "Copiar layout de…" outro formato. No ⚙, o formato deste aparelho.
- **3D View:** palco com as fixtures acendendo pela saída simulada (feixes, pixels das barras, strobe), vistas 3D, topo, frente e lateral, arrastar a fixture na vista de topo, posição e rotação no painel, tamanho do palco, qualidade e fumaça (por aparelho).
- **Contrato:** `Console` e `Stage` no projeto; `master` e `widgetValues` no estado de execução; operações novas em [`docs/protocolo.md`](../protocolo.md). O `MockClient` simula o efeito do console na saída (playback, submaster, nível, XY pad, apagar fixtures, Grand Master e blackout).

**Ajustes pedidos na validação da rodada 3 (2026-10-16)**
- Na edição do console, o painel de propriedades fica sempre no lugar (com um aviso quando nada está escolhido): escolher ou soltar um elemento não mexe no layout.
- Grade do console mais fina: cada célula virou 2 × 2 (16, 32 e 48 colunas).
- Paleta dos elementos com a versão escura de cada cor e cinzas do branco ao preto, num botão que abre a paleta. O Texto ganhou cor de fundo e de borda e pode ter 1 célula de altura.
- Etiqueta com a tecla no canto dos elementos acionados por teclado, na operação e na edição.
- Correção: no celular, o toque que abria o painel de propriedades também abria a paleta de cor embaixo do dedo (clique fantasma); o `RlButton` passou a cancelar esse clique.
- 3D View: "Editar posição" ao lado das vistas; ligado, arrastar a fixture a move em qualquer vista (topo, frente, lateral e 3D), e no vazio a câmera continua mexendo.
- Atalhos padrão: Ctrl + − e Ctrl + + (zoom do sistema, ou do console quando ele está na frente); −, + e Enter do teclado numérico para blackout, parar tudo e tap do BPM, com etiquetas na seção master. Essas teclas são recusadas na captura dos elementos; as da fileira de cima ficam livres.
- 3D View: aba Elementos, com elementos de palco (box truss de 1, 1,5 e 3 m, praticável, escada, painel LED, tela, caixas de grave, ativa e retorno, pedestal, pessoa, bateria, teclado, púlpito e planta); Ctrl + D duplica ao lado e Del exclui. O toque e o arrasto valem só para o que é da aba aberta (fixtures ou elementos).
- 3D View com o mouse: o botão do meio gira ou desloca a câmera, o zoom é só na roda e o botão esquerdo só escolhe (e arrasta o escolhido com "Editar posição").
- Barras de rolagem no padrão do design system em todo o app (trilho afundado e cursor em relevo, sem setas).
- Configurações num painel lateral aberto pelo ⚙, sobre a tela atual (antes era uma tela): dá para ajustar algo sem sair de uma criação ou edição. No celular, o painel sobe de baixo, e o "Mais" ficou só com os menus.
- Botão de dicas de atalho ao lado do "Editar": mostra ou esconde as etiquetas das teclas (elementos e seção master). Atalhos Alt + E (editar) e Alt + I (dicas), também reservados.
- Zoom geral da interface no ⚙ (no app desktop; no navegador, o do navegador) e zoom do console ao lado do "Editar", guardados em cada aparelho.
- Botões sem ícone por padrão. Nas propriedades, "Mostrar ícone" liga o do tipo da criação, e "Trocar" abre a biblioteca inteira de ícones com busca.

**Pedidos depois da aprovação da rodada 3 (2026-10-16)**

- Celular e tablet: botão de tela cheia na barra de cima (onde o navegador permite) e zoom da interface no ⚙, pela largura do viewport.
- Celular (2026-10-17): o 100% do zoom passa a ser 75% da escala do navegador.

**Decisões do início da rodada 3 (2026-10-16)**
- **VC-2:** elementos compartilhados entre os formatos; cada formato guarda só a posição. Três formatos fixos pela largura da tela, com troca no ⚙.
- **VC-3:** o elemento "Tempos" ajusta as criações de uma lista, como o Speed Dial do QLC+.

**Ajustes pedidos na validação da rodada 2 (2026-10-16)**
- **Editor de cena:**
  - o toque escolhe uma fixture; Ctrl + clique (ou toque longo, no celular) soma outras do mesmo tipo, e o que se mexe vale para todas as escolhidas;
  - aba Canais primeiro, e as de atributo só aparecem quando a fixture tem o tipo de canal;
  - "Adicionar fixtures" (primeiro botão da fileira) abre uma lista com nome, universo e canais, com os atalhos "Desmarcar todos", "Selecionar do mesmo tipo" e "Selecionar grupo";
  - divisor entre a fixture escolhida e os grupos de canais; ligar um grupo traz as fixtures dele para a cena, e o fader só mexe nas fixtures da cena;
  - sem textos de ajuda na tela: explicações no ícone "i" (E6).

**Rodada 2 entregue**
- **Studio:** lista de todas as criações agrupada por tipo (com ícone), pastas, busca e rodar/parar em cada uma; criar (escolhendo o tipo), renomear, duplicar e excluir.
- **Editores:**
  - **Cena:** fixtures da cena, escolha das que recebem os valores e as ferramentas Intensidade, Cor (roda, branco/âmbar/UV, roda de cores e macros), Posição (pad XY), Efeitos (shutter/strobe, gobo, prisma, zoom, foco, íris) e Canais (um fader por canal); grupos de canais com fader.
  - **Chaser:** passos com fade in, hold e fade out, ordem, sentido, tempos em batidas e de onde vem cada tempo (padrão, comum ou por passo); passo atual aceso enquanto roda.
  - **Sequência:** o mesmo editor, preso a uma cena; cada passo tem os próprios valores, editados em faders.
  - **Coleção:** lista de criações disparadas juntas.
  - **EFX:** os 10 desenhos do QLC+, parâmetros em knobs, prévia animada com as fixtures, propagação, ordem, sentido, tempos e fixtures com ordem, inversão e deslocamento.
  - **RGB Matrix:** grupo de fixtures, algoritmos de amostra (os scripts do QLC+ entram na F9), cores, tempos e prévia animada da grade.
- **Mesa simples:** um fader por canal, os 512 do universo numa faixa só (Ctrl + roda do mouse rola), com a fixture e o atributo de cada canal e o medidor de saída; grupos de canais; "Zerar".
- **Monitoramento:** CPU, memória, tick do motor, tempo ligado, clientes conectados, saídas DMX por universo e a grade dos 512 valores de saída ao vivo.
- **Contrato:** criações do Studio, estado de execução (rodando, mesa, estatísticas) e as operações novas em [`docs/protocolo.md`](../protocolo.md). O `MockClient` simula a saída a partir das criações rodando.
- **Design system:** `RlSegmented`, `RlTimeField`, `RlChannelStrip` sem botões e com etiqueta própria, e a tabela de ícones dos tipos de criação.

**Rodada 1 entregue**
- Shell com menus, menu Projeto, status USB e ⚙. No celular, barra inferior com "Mais".
- Modo modular (dockview) com layout salvo por aparelho, e janela externa nativa no Tauri.
- Telas Fixtures (projeto, biblioteca, importação `.qxf`, editor, grupos), Patch, In/Out e Configurações. Console, Studio, 3D View, Mesa simples e Monitoramento mostram em que rodada chegam.
- Contrato v0 em `crates/rlc-core` (tipos TS gerados), `MockClient` e rascunho de [`docs/protocolo.md`](../protocolo.md).

**Verificado automaticamente**
- 12 fluxos no navegador:
  - adicionar fixtures;
  - indicador de alteração não salva e salvar;
  - relatório de importação;
  - endereçamento automático tirando o conflito;
  - modo modular e menu abrindo painel;
  - janela externa, com o aviso no dock;
  - mudança feita na janela externa chegando na principal;
  - painel voltando ao fechar a janela.
- No app Tauri: o botão abre uma janela nativa separada, com o título sincronizado.
- `pnpm check` e build de produção passam.

**Ajustes durante a rodada**
- As telas usam container queries, para se adaptar a painéis estreitos (regra registrada no design system).
- Abaixo de 1536 px, a barra esconde o BPM, o nome R·LIGHT e o texto USB (o LED fica), para tudo caber em 1280 px. O BPM vai morar no Console.

**Ajustes pedidos na validação (2026-10-16)**
- **Grupos de canais**, além dos grupos de fixtures (E12).
  - Ficam na aba Grupos, numa lista com as duas seções.
  - O ícone na lista vem do atributo quando todos os canais têm o mesmo; senão, é genérico.
  - Editor: toque nos canais de cada fixture, ou use os atalhos por atributo, que pegam todas as fixtures de uma vez.
- **Abas agrupadas** num trilho afundado, como um controle segmentado, no lugar dos botões soltos. Valem para todas as telas, incluindo os universos do Patch.
- **Patch:** arrastar a fixture na grade para mudar o endereço.
- **Studio:** lista agrupada por tipo (o filtro de tipo saiu), com pastas por tipo; arrastar move a criação entre pastas do mesmo tipo.
- **Studio:** coluna da lista com 360 px; botão "Parar todas" ao lado da busca; ao criar uma coleção com criações rodando, pergunta se ela nasce vazia ou com elas.
- **Editor de cena:** fade in e fade out no fim, numa seção "Tempos" (o console ajusta os fades ao vivo, VC-3).
- **Barra superior:** botão de pânico (um toque para as criações; dois toques também zeram a Mesa simples).
- **Mesa simples:** em tela larga, canais em cima e grupos de canais embaixo (como no QLC+); no celular, grupos numa aba.
- **In/Out sem abas:** seções Saídas e Entradas. A seção Entradas só lista os controladores MIDI conectados. O mapeamento de MIDI e de teclado sai do In/Out e vai para cada elemento do console (rodada 3).

## Objetivo

Montar todas as telas da v1 com dados fictícios, navegáveis no desktop, no tablet e no celular, para você aprovar fluxos e layout antes de qualquer recurso (G2).

Ao fim da fase:
- as telas ficam "congeladas" como referência;
- o contrato de dados está definido.

Da F3 em diante, cada fase só liga uma tela já aprovada ao motor real.

## Menus (E12)

| Menu | O que tem |
|---|---|
| **Console** | Virtual Console (operação e edição) e a seção master: GM, Blackout, Stop All (só aqui) |
| **Fixtures** | Biblioteca, importação de `.qxf`, criação e edição de fixtures, fixtures do projeto, grupos |
| **Studio** | Todas as criações: cena, chaser, sequência, coleção, EFX, RGB Matrix. Cada tipo tem um ícone próprio, usado em todo o projeto |
| **Patch** | Endereçamento DMX das fixtures, grade dos universos, conflitos |
| **3D View** | Preview, configuração do palco e posicionamento dos equipamentos |
| **Mesa simples** | Banco de faders: canais DMX específicos ou grupos de canais, fora do console |
| **Monitoramento** | Universos mandando sinais (valores ao vivo) e dados do app e da máquina: CPU, memória, fps das saídas, clientes conectados |
| **In/Out** | Saídas (universos e interfaces DMX USB) e Entradas (controladores MIDI conectados) |
| **⚙ Configurações** | Formato de valores (% ou DMX), acesso remoto (PIN, QR), modo da interface, sobre/licenças |

O menu **Projeto** (novo, abrir, salvar, recentes) fica no canto esquerdo da barra superior.

## Interface modular (E13)

- **Modo simples:** uma tela por vez, trocada pelos menus. É o modo inicial e o único no celular.
- **Modo modular**, para telas de 1024 px ou mais:
  - cada menu vira um painel;
  - painéis viram abas, se encaixam lado a lado ou empilhados, e são redimensionados arrastando a divisa (biblioteca **dockview**);
  - um painel também pode **flutuar** dentro da janela;
  - o layout fica salvo por aparelho.
- **Janela externa:** "Abrir em janela" leva o painel para uma janela nova do Windows, que pode ir para outro monitor.
  - A janela externa é a mesma tela conectada ao motor como mais um cliente (E2). O preview 3D continua rodando mesmo com a janela principal minimizada.
  - No lugar do painel fica um aviso "aberto em janela externa", com o botão "Trazer de volta".
  - No navegador (celular ou outro PC), a mesma ação abre uma nova aba.

```
┌ Projeto ▾ │ Console  Fixtures  Studio  Patch  3D View  Mesa simples │ 120 BPM  ● USB  ⚙ ┐
├──────────────────────────────────────┬─────────────────────────────────────────────┤
│ Studio                       [⧉][×]  │ 3D View                             [⧉][×]  │
│  ◆ Cenas  ↻ Chasers  ≋ Sequências…   │                                             │
│                                      │          (preview)                          │
├──────────────────────────────────────┤                                             │
│ Mesa simples                 [⧉][×]  │                                             │
│  ┃ ┃ ┃ ┃ ┃ ┃ ┃ ┃                     │                                             │
└──────────────────────────────────────┴─────────────────────────────────────────────┘
                     ⧉ = abrir em janela externa (outro monitor)
```

## Rodadas (uma por vez, cada uma validada antes da próxima)

### Rodada 1: shell modular, Fixtures, Patch, In/Out e Configurações

1. **Shell**
   - Barra superior com menus, menu Projeto, status (USB, BPM) e ⚙.
   - Modo simples e modo modular: abas, encaixe, redimensionamento, painel flutuante, layout salvo.
   - Janela externa no Tauri e no navegador.
   - Navegação no celular.
2. **Fixtures**
   - Biblioteca com busca por fabricante e modelo, e detalhes (canais, modos, dados físicos).
   - Importação de `.qxf` (arquivo ou upload pelo celular), com relatório de conversão.
   - Fixtures do projeto: adicionar a partir da biblioteca, nome, modo e cor de etiqueta.
   - Editor de fixture: dados gerais, canais (grupo/preset e faixas de capabilities), modos, cabeças e dados físicos.
   - Grupos de fixtures: lista e editor de grade 2D, usada pelo RGB Matrix.
   - Grupos de canais: canais de quaisquer fixtures, para faders do console e da mesa simples e para cenas.
3. **Patch**
   - Lista por universo: endereço, canais e conflitos.
   - Grade do universo com 512 células mostrando a ocupação, conflitos em vermelho e valores ao vivo.
   - Endereçamento automático de várias fixtures.
   - Arrastar a fixture na grade para mudar o endereço, com prévia do destino (verde se livre, vermelho se invade outra).
4. **In/Out**, sem abas:
   - **Saídas:** universos e a interface DMX de cada um; interfaces detectadas com tipo, serial, status, fps e jitter.
   - **Entradas:** controladores MIDI conectados por USB. O mapeamento fica em cada elemento do console (rodada 3).
5. **⚙ Configurações e menu Projeto.**

**Teste técnico desta rodada:** abrir um painel numa janela externa do Tauri e confirmar que ela continua atualizando com a janela principal minimizada. Com o `MockClient`, as janelas se sincronizam pelo `BroadcastChannel` do navegador; com o motor real (F3), pelo WebSocket.

### Rodada 2: Studio, Mesa simples e Monitoramento

6. **Studio**
   - Lista de todas as criações com ícone por tipo e filtro.
   - Criar, rodar/parar, duplicar, renomear e excluir.
   - Os **ícones dos tipos** entram no design system.
7. **Editores**
   - **Cena:** fixtures e valores, com as ferramentas de atributo (intensidade, cor, posição, beam, gobo) e os grupos de canais.
   - **Chaser:** passos com fade in, hold e fade out; ordem Loop, SingleShot, PingPong ou Random; modos de velocidade.
   - **Sequência.**
   - **Coleção.**
   - **EFX:** algoritmo, parâmetros, fixtures com ordem e offset, preview animado do caminho.
   - **RGB Matrix:** grupo, algoritmo, cores, preview animado da grade.
8. **Mesa simples:** faders por canal DMX (os 512 do universo numa faixa, Ctrl + roda rola) e faders por grupos de canais.
9. **Monitoramento:** valores ao vivo dos universos, CPU, memória, fps e jitter das saídas, clientes conectados.

### Rodada 3: Console e 3D View

10. **Console, operação:** páginas, widgets em uso (botão, slider, cue list, XY pad, speed dial, frame, solo frame, relógio, animation, label) e a seção master. O slider pode controlar um grupo de canais.
11. **Console, edição**
   - Grade com snap e paleta de widgets.
   - Mover e redimensionar por toque e mouse; seleção múltipla e alinhamento.
   - Propriedades por elemento, incluindo o **VC-1**:
     - destaque e cor;
     - modo (liga/desliga ou flash);
     - ação: função, blackout geral, blackout de fixtures ou grupos, ou stop all;
     - entrada externa: controle MIDI (com "aprender") ou tecla do teclado.
   - **VC-2:** layouts por formato de tela. Cada aparelho abre o layout do seu formato. Os elementos são compartilhados e cada formato guarda só a posição (decidido no início da rodada).
   - **VC-3:** controle no console para ajustar os fades ao vivo.
12. **3D View:** layout do preview (com a cena de feixes da F1 como amostra), vistas, palco e posicionamento das fixtures.

## Contrato de dados v0 e dados fictícios

- **`crates/rlc-core`:** structs do domínio, só com dados, sem lógica, montadas rodada a rodada conforme as telas precisam.
  - Os tipos TypeScript são gerados com ts-rs em `ui/app/types/generated/`. Nunca se edita à mão, e o `pnpm check` confere se estão atualizados.
- **`EngineClient`:** a única porta das telas para o motor. Expõe o estado do projeto e de execução e um `send(operação)`.
  - **`MockClient` agora:** projeto fictício em memória, com uma simulação simples.
  - **`WsClient` na F3:** fala com o servidor Rust. As telas não mudam.
- **Projeto fictício:** cerca de 20 fixtures (PARs RGB, moving heads, barras de LED, strobe) em 2 universos, mais criações e um console de 2 páginas.

## Referências do QLC+ (`reference/qlcplus/qmlui/qml/`)

- Fixtures: `fixturesfunctions/{FixtureBrowser,FixtureProperties,FixtureGroupEditor,FixtureGroupManager}.qml` e `fixtureeditor/`
- Patch: `fixturesfunctions/{UniverseGridView,UniverseSummary,DMXView}.qml` e `inputoutput/`
- Studio: `fixturesfunctions/{FunctionManager,SceneEditor,ChaserEditor,SequenceEditor,CollectionEditor,EFXEditor,RGBMatrixEditor}.qml`
- Ferramentas de atributo: `fixturesfunctions/{IntensityTool,ColorTool,PositionTool,BeamTool,PresetsTool}.qml`
- Mesa simples: `SimpleDesk.qml`
- Console: `virtualconsole/`
- 3D View: `fixturesfunctions/3DView/` e `SettingsView2D.qml`

## Saída

- Prints das telas aprovadas em `docs/telas/`.
- Rascunho de `docs/protocolo.md`, tirado do que as telas precisam.

## Fora do escopo

Motor, USB, arquivos de verdade (abrir e salvar são simulados) e MIDI de verdade.

## Decisões do usuário (2026-10-16)

1. **Onde se programa:** as ferramentas de atributo ficam no editor de cena do Studio.
2. **Dispositivos:** dois menus próprios. **Monitoramento** mostra os universos e os dados do app e da máquina; **In/Out** configura as interfaces e o MIDI.
3. **Modo inicial:** simples; o modular é ativado opcionalmente.
4. **Celular:** Console · Studio · Mesa simples · Fixtures · Mais.

## Validação

- [x] Rodada 1 aprovada (shell modular, janela externa, Fixtures, Patch, In/Out, Configurações).
- [x] Rodada 2 aprovada (Studio, editores, Mesa simples, Monitoramento).
- [x] Rodada 3 aprovada (Console em operação e edição, VC-1, VC-2, 3D View).
- [x] Todos os fluxos percorridos no desktop, no tablet e no celular.
- [x] Contrato v0 e rascunho do `docs/protocolo.md` revisados (2026-10-17).
- [x] Telas congeladas, com prints em `docs/telas/` (2026-10-17).

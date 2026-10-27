# Plano — r-light-control

Controlador DMX próprio, com o QLC+ (Apache-2.0) como referência de arquitetura e protocolos. Tem só os recursos que o usuário usa, mais as melhorias que ele quer:
- UI touch inspirada na Soundcraft Ui24R;
- operação pelo celular;
- visualização 2D/3D;
- editor de fixture simples;
- recursos de Virtual Console sob demanda.

As regras e decisões estão em [DECISOES.md](DECISOES.md).

**Status:** ⬜ pendente · 🟡 em andamento · ✅ concluída

## Fluxo de cada fase

1. Estudar os arquivos de referência do QLC+ listados na fase.
2. Escrever o mini-plano em `docs/fases/fase-XX.md` → **o usuário aprova**.
3. Implementar com testes automatizados.
4. Entregar o checklist de validação → **o usuário valida** (com hardware real quando aplicável).
5. Ajustar → validar de novo.
6. Marcar a fase aqui e registrar as decisões novas no `DECISOES.md`. Commit se o usuário pedir.

## Pendências (2026-10-21)

**Onde paramos (2026-10-27, para retomar em outra conversa):** a [F10d](docs/fases/fase-10d.md) (EFX avançado) foi entregue e commitada em 2026-10-27, e falta o usuário validá-la junto com a F10c, numa rodada só ([checklist](docs/fases/fase-10d.md#validação-com-você), pedido dele). A [F9](docs/fases/fase-09.md) (RGB Matrix) foi aprovada e entregue no mesmo dia (2026-10-27), e vai na mesma rodada de validação; o "Show de exemplo" do usuário (a recuperação e o arquivo salvo) ganhou os exemplos dela, com cópia dos arquivos antes. Detalhes que não estão no código:
- Em 2026-10-22, o app abre o "Show de exemplo" pela recuperação: o arquivo salvo ainda é da versão 8, e o trabalho da F10c e os exemplos da F10d estão na recuperação (os da F10d também foram postos no arquivo salvo). Antes de qualquer teste que abra outro projeto, guardar uma cópia da recuperação e restaurá-la no fim (abrir um projeto descarta a recuperação).
- Os "botões acesos, sem LED" pedidos na F10c valeram para o Studio (Rodar, Pausar, Parar todas e a lista) e para a barra de cima. Os botões de dentro das telas (Prévia, Localizar, Editar do console etc.) continuam em LED: o usuário ainda não decidiu se quer trocá-los.
- A área do console tem a célula de tamanho fixo por formato (mudança do usuário, 2026-10-21, em [DECISOES.md](DECISOES.md), E11).

O usuário validou a F7 e a F8 em 2026-10-20. O que falta:

**Na [validação final](docs/validacao-final.md)**, antes do release:
- com os moving beams: 3 itens da F6, 1 da F7, 7 da F8 (o EFX), 3 da F10b (o EFX em grupo), 1 da F10c (a posição lenta com a cor rápida) e 4 da F10d (os exemplos do EFX avançado);
- adiados por escolha dele, sem precisar dos Beams: o tick com 4 universos (F6), a sequência e a coleção (F7).

**Na F16:** o instalador num PC limpo (adiado da F3).

**Pesquisa da grandMA (pedido do usuário, 2026-10-20):** para profissionalizar o app, o catálogo de recursos está em [docs/pesquisa-grandma.md](docs/pesquisa-grandma.md). Em 2026-10-21 o usuário escolheu os itens e aprovou a ordem, com a prioridade na operação ao vivo.

**Ordem de execução (aprovada em 2026-10-21; a F10d entrou no mesmo dia, pedido do usuário):** Ajustes rápidos → F10 → F10b → F10c → F10d → F9 → F9b → F11 → F12 → F12b → F12c → F13 → F13b → F14 → F15 → F16. Todos os itens escolhidos estão em alguma fase, sem fila à parte (o usuário pediu a fila de baixa prioridade dentro das fases, em 2026-10-21). As fases novas levam uma letra e os números das antigas não mudam. A F10 passa à frente da F9, mas a F9 continua antes da F11 (o elemento "Animation" controla um RGB Matrix). Os [ajustes rápidos](docs/fases/ajustes-rapidos.md), a [F10](docs/fases/fase-10.md) e a [F10b](docs/fases/fase-10b.md) foram validados em 2026-10-21. A [F10c](docs/fases/fase-10c.md) foi entregue no mesmo dia; em 2026-10-22 o usuário $1 A [F9](docs/fases/fase-09.md) foi aprovada e entregue em 2026-10-27, antes da validação da F10c e da F10d (G1: o usuário pediu para seguir).

**Em aberto, sem decisão:** trocar o texto "Tempos em batidas do BPM" do editor de chaser por "Seguir o BPM", para as duas telas falarem igual ao elemento "Tempos" (sugestão da F7).

**O que ficou de fora da F7 e da F8** está no [Backlog](#backlog-v2), para entrar só se o usuário pedir.

## Arquitetura alvo

```
r-light-control/
├─ CLAUDE.md · PLANO.md · DECISOES.md · NOTICE · README.md
├─ docs/  padroes-codigo.md · design-system.md · protocolo.md · fases/ · telas/
├─ crates/
│  ├─ rlc-core     modelo de dados + formatos JSON (serde + ts-rs → tipos TS), sem I/O
│  ├─ rlc-engine   tick 50 Hz, camadas, merge HTP/LTP, GM, fades, funções
│  ├─ rlc-dmx      Enttec Pro, Open DMX, Dummy; descoberta e hot-plug
│  ├─ rlc-qxf      importador .qxf → formato próprio
│  ├─ rlc-midi     entrada/feedback MIDI, learn
│  ├─ rlc-server   axum: UI embutida + /ws + upload/download
│  └─ rlc-headless servidor sem janela (testes, futuro Raspberry Pi)
├─ app/src-tauri/  casca: instância única, sobe o servidor, abre a janela
├─ ui/             Nuxt 4 SPA · Pinia · Tailwind v4 · Reka UI · TresJS
└─ reference/qlcplus/  (fora do git)
```

Os crates são criados na fase em que passam a ser necessários (G7).

---

## Etapa A — Fundação, padrões e interface (antes de qualquer recurso)

### F0 — Fundação e padrões · ✅

- **Ambiente:** rustup (MSVC), VS 2022 Build Tools (C++) e pnpm.
- **Repositório:**
  - git, `.gitignore`.
  - `CLAUDE.md`, `PLANO.md`, `DECISOES.md`, `NOTICE`.
  - Referência do QLC+ em `reference/qlcplus`.
- **Padrões:** `docs/padroes-codigo.md`.
- **Esqueleto ponta a ponta:**
  - Workspace Cargo + Tauri + Nuxt.
  - O axum serve a UI e expõe `/ws`, com handshake `hello` e contador de ticks a 50 Hz.
  - QR code com a URL da rede local.
  - Logs em arquivo e arquivo de configurações.
  - `pnpm check`.

Concluída e validada pelo usuário em 2026-10-15.

Validação:
- [x] A janela mostra o contador subindo (~50/s). _Teste automático: 50,2 ticks/s._
- [x] O celular abre a mesma página pelo QR e vê o mesmo contador.
- [x] `docs/padroes-codigo.md` aprovado.
- [x] `pnpm check` passa.

### F1 — Design system (Ui24R) · ✅

Mini-plano: [docs/fases/fase-01.md](docs/fases/fase-01.md). Concluída e validada pelo usuário em 2026-10-16 (60 fps no celular, 100 no desktop).

- **Entrada do usuário:** prints e referências da Ui24R (em `docs/design/`).
- **Tokens:** tema escuro, cores por atributo (intensidade, cor, posição, beam), tipografia, espaçamento e alvos de toque ≥44 px.
- **Componentes:**
  - `RlFader`: multitoque, modo fino, toque duplo para resetar, medidor.
  - `RlButton`: toggle/flash, solta em `pointercancel`.
  - `RlKnob`, `RlXYPad`, `RlColorPicker`, `RlChannelStrip`, `RlMeter`.
  - Navegação, Sheet/Dialog, teclado numérico, listas e formulários.
- **Páginas e layouts:** página `/dev/showcase` e layouts para celular, tablet e desktop.
- **Spikes:** 64 faders animados; 50 cones de feixe em TresJS no celular do usuário.
- **Documentação:** `docs/design-system.md`.

Validação:
- [x] Visual aprovado no desktop, tablet e celular.
- [x] 4 ou mais faders movidos ao mesmo tempo com vários dedos.
- [x] 60 fps no celular.
- [x] Sem zoom, rolagem ou seleção de texto acidentais.

### F2 — Protótipo navegável de todas as telas · ✅

Mini-plano: [docs/fases/fase-02.md](docs/fases/fase-02.md) (aprovado em 2026-10-16; rodadas 1, 2 e 3 aprovadas). Concluída e validada pelo usuário em 2026-10-17, com as telas congeladas em [docs/telas/](docs/telas/).

- **Menus (E12):** Console · Fixtures · Studio · Patch · 3D View · Mesa simples · Monitoramento · In/Out, com as Configurações num painel lateral aberto pelo ⚙.
- **Interface modular (E13):** painéis encaixáveis e redimensionáveis, janelas externas para outros monitores; começa no modo simples.
- **Três rodadas, uma por vez:**
  1. Shell modular, Fixtures, Patch, In/Out e Configurações.
  2. Studio (com ícone por tipo de criação), Mesa simples e Monitoramento.
  3. Console (com VC-1 e VC-2) e 3D View.
- **Contrato de dados v0:** structs em `rlc-core` → tipos TS via ts-rs, montados rodada a rodada. A UI usa um `EngineClient` (`MockClient` agora, `WsClient` depois).
- **Saída:** prints em `docs/telas/` e rascunho de `docs/protocolo.md`.

Validação:
- [x] Rodada 1 aprovada.
- [x] Rodada 2 aprovada.
- [x] Rodada 3 aprovada.
- [x] Todos os fluxos percorridos no desktop, tablet e celular.
- [x] Telas aprovadas e congeladas como referência.

## Etapa B — Núcleo

### F3 — Motor base + saída USB DMX · ✅

Mini-plano: [docs/fases/fase-03.md](docs/fases/fase-03.md) (aprovado em 2026-10-17). Concluída e validada pelo usuário em 2026-10-17, com a Numbstar e os ParLed. O teste do instalador num PC limpo foi para a F16.

- **Motor:** tick, N universos, camada de mesa, Grand Master, blackout e pânico.
- **`rlc-dmx`:**
  - Enttec Pro (label 6).
  - Open DMX via D2XX (sem reserva pela porta COM, como no QLC+).
  - Dummy.
- **Dispositivos:** descoberta pelo D2XX a cada segundo, tipo pelo nome, pela pergunta da Pro ou à mão, reconexão, estatísticas (fps, jitter, erros).
- **UI:** In/Out, Mesa simples e Monitoramento ligados ao motor. Primeiro instalador.
- **Protocolo:** cliente WebSocket no lugar do `MockClient`; `evt` com o pedaço do estado que mudou; frames binários dos universos; IDs UUIDv7 gerados pela UI (A2–A4).
- **Ref:**
  - `plugins/dmxusb/src/{dmxusbwidget,enttecdmxusbpro,enttecdmxusbopen,ftd2xx-interface,qtserial-interface}.cpp`
  - `engine/src/{outputpatch,inputoutputmap,grandmaster,mastertimer-win32}.cpp`

Validação (detalhada em [docs/fases/fase-03.md](docs/fases/fase-03.md)):
- [x] Rodada 1: servidor de verdade, aparelhos sincronizados, Monitoramento real.
- [x] A Open DMX do usuário (Numbstar) controla fixtures reais. A Pro fica nos testes automáticos até o usuário ter uma.
- [x] 10 min sem flicker.
- [x] A saída volta em até 3 s depois de replugar.
- [x] Blackout instantâneo.
- [x] O Dummy funciona sem hardware.
- [ ] O instalador funciona num PC limpo. _Adiado para a F16 (pedido do usuário)._

### F4 — Fixtures: formato e importador .qxf · ✅

Mini-plano: [docs/fases/fase-04.md](docs/fases/fase-04.md) (aprovado em 2026-10-17). Concluída e validada pelo usuário em 2026-10-17.

- **Formato:** schema JSON de fixture (`schemaVersion`) com taxonomia própria de atributos e o valor padrão de cada canal (o `Default` do `.qxf`; ex.: o Strobo do Beam 7R em 255, "luz ativa").
- **Importador:** `rlc-qxf`, com relatório de conversão.
- **Biblioteca:** do usuário, com busca e upload também pelo celular.
- **Decisão desta fase (2026-10-17):** a biblioteca do QLC+ não é embutida. As fixtures entram só importadas pelo usuário ou criadas à mão.
- **Ref:**
  - `engine/src/{qlcfixturedef,qlcfixturemode,qlcchannel,qlccapability,qlcphysical,qlcfixturehead}.cpp`
  - `resources/schemas/fixture.xsd`

- **Fixtures do usuário:** o ParLed 18x15w e o Beam 7R estão convertidos à mão no show de exemplo (`crates/rlc-server/src/exemplo.json`), e o importador dá o mesmo resultado (teste automático). Os três `.qxf` (`R:\Light\QLC+\Fixtures\RSena\`) o usuário importa pelo app.

Validação:
- [x] Os .qxf das fixtures reais importam sem erro: o usuário importa o Strobo 1000w pelo app.
- [x] A importação do ParLed 18x15w e do Beam 7R bate com a conversão feita à mão no exemplo (incluindo o valor padrão do Strobo do Beam).
- [x] Relatório correto.
- [x] Conferência manual contra o QLC+.

### F5 — Patch, projeto e programador · ✅

Mini-plano: [docs/fases/fase-05.md](docs/fases/fase-05.md) (aprovado em 2026-10-17). Concluída e validada pelo usuário em 2026-10-17.

- **Projeto:** fixtures embutidas; patch com conflitos e endereçamento automático (prontos desde a F2 e a F3).
- **Grupos de canais:** gravados no projeto e usados pelos faders da mesa simples, das cenas (F6) e do console (F10).
- **Arquivos:** `.rlc` numa pasta só (`Documentos\R Light Control`), salvar e "Salvar como" pelo nome, abrir pela lista, `.rlc.bak`, cópia de recuperação e o que abre ao iniciar (recuperação, último projeto ou em branco).
- **Programador (o editor de cena, E12):** as ferramentas de atributo da F2, a **Prévia** (a cena em edição na luz) e o **Localizar** (o highlight do QLC+).
- **Edição:** API por operações tipadas com revisão (pronta desde a F3).
- **Desfazer e refazer (2026-10-17, pedido do usuário):** Ctrl + Z e Ctrl + Shift + Z em todas as telas e menus. Com o console na frente em operação, as teclas vão para os elementos; na edição do console, desfazem.
- **Ref:**
  - `engine/src/{doc,fixture,channelmodifier}.cpp`
  - `qmlui/{fixturemanager,contextmanager}.cpp`
  - `qmlui/tardis/tardis.cpp`

Validação (detalhada em [docs/fases/fase-05.md](docs/fases/fase-05.md)):
- [x] 20 fixtures em 2 universos.
- [x] Conflito sinalizado.
- [x] Salvar → reabrir dá um projeto idêntico.
- [x] Recuperação após matar o processo.
- [x] Prévia: a cena em edição muda a luz na hora; desligada, nada muda.
- [x] Localizar acende a fixture certa no palco.
- [x] Celular e desktop sincronizados (inclusive a prévia e o Localizar).
- [x] Ctrl + Z e Ctrl + Shift + Z desfazem e refazem em todas as telas; com o console em operação, as teclas vão para os elementos.
- [x] `pnpm check` passa.

## Etapa C — Funções

### F6 — Merge, fades e Cena · ✅

Mini-plano: [docs/fases/fase-06.md](docs/fases/fase-06.md) (aprovado em 2026-10-17). Concluída em 2026-10-18, a pedido do usuário, com 8 itens validados. Os outros (o tick com 4 universos e o elemento "Tempos", ligado às cenas a pedido dele) ainda não foram testados e serão ajustados depois, se precisar. A posição entre cenas e os fades nos Beams estão na [validação final](docs/validacao-final.md) (2026-10-19).

- **Merge:** HTP na intensidade (dimmer e cores, como no QLC+) e LTP por ordem de início no resto. Ao parar, o LTP volta ao que está embaixo.
- **Fades:** lineares em inteiros (os números do QLC+), do valor atual, com canais finos de 16 bits. Rodas, gobo, prisma e strobo mudam de uma vez.
- **Grand Master:** só reduzir a intensidade (os modos Limit e All saíram do plano).
- **Cena:** gravada do programador e editável enquanto roda. Stop All.
- **Testes:** portados de `engine/test/{universe,genericfader,fadechannel,scene,grandmaster}`.
- **Camadas da saída (decididas no mini-plano):** valor padrão, criações rodando, prévia (a cena em edição), Mesa simples, Localizar, Grand Master e blackout (`rlc-engine`). O programador continua sendo o editor de cena com a Prévia (E12).
- **Tempos (VC-3), ajuste da validação:** o elemento do console troca o fade in e o fade out das cenas da lista, sem mexer nos tempos gravados.
- **Ref:** `engine/src/{universe,genericfader,fadechannel,function,scene}.cpp`

Validação:
- [x] Duas cenas sobrepostas: maior dimmer vence e as cores se misturam (HTP). _Na posição (vence a última iniciada): na [validação final](docs/validacao-final.md), com os moving beams._
- [x] GM a 50% reduz só a intensidade.
- [ ] Tick abaixo de 2 ms com 4 universos. _Na [validação final](docs/validacao-final.md) (o teste automático mede 0,68 ms no app compilado)._
- [x] O elemento "Tempos" troca o fade in e o fade out das cenas da lista (ajuste pedido na validação). _Validado em 2026-10-20._

### F7 — Chaser, Sequência, Coleção e BPM · ✅

Mini-plano: [docs/fases/fase-07.md](docs/fases/fase-07.md) (aprovado em 2026-10-18; entregue em 2026-10-18). Concluída em 2026-10-19, a pedido do usuário, sem a validação. Validada em 2026-10-20, junto com a F8, menos o deslizar dos Beams, a sequência e a coleção, que estão na [validação final](docs/validacao-final.md).

- **Chaser:** modos de velocidade Default/Common/PerStep, crossfade, next/prev/goto, run order Loop/SingleShot/PingPong/Random.
- **Sequência** e **Coleção.**
- **Tempo:** BPM global com tap tempo.
- **Fim do Single Shot (pedido do usuário, 2026-10-18):** opção do chaser: apagar depois do último passo ou manter o último passo aceso.
- **Cue list:** ligada ao chaser de verdade já nesta fase (play, stop, anterior, próximo e toque num passo).
- **Ajuste pedido na validação (2026-10-19):** o elemento "Tempos" segue o BPM master ("Seguir o BPM", ligada por padrão): tempos em batidas e um TAP só, o do master.
- **Ref:** `engine/src/{chaser,chaserrunner,chaserstep,sequence,collection,mastertimer}.cpp`

Validação (detalhada em [docs/fases/fase-07.md](docs/fases/fase-07.md)):
- [x] Chaser de 8 passos com crossfade de 2 s, suave. _Os Beams deslizando: na [validação final](docs/validacao-final.md)._
- [x] Tap a 120 BPM mantém o tempo.
- [x] PingPong e Random corretos.
- [x] O TAP master muda os tempos de todos os elementos "Tempos" que seguem o BPM, e os chasers da lista acompanham (ajuste pedido na validação).

### F8 — Grupos de fixtures e EFX · ✅

Mini-plano: [docs/fases/fase-08.md](docs/fases/fase-08.md) (aprovado em 2026-10-19; entregue em 2026-10-19; validada pelo usuário em 2026-10-20, sem os Beams). Os testes que faltam estão na [validação final](docs/validacao-final.md). Os testes com os moving beams foram para a [validação final](docs/validacao-final.md) (2026-10-19, pedido do usuário).

- **Grupos:** grupos 2D de cabeças.
- **EFX:** os 10 algoritmos, modo absoluto e relativo, spread/offset, preview do caminho.
- **Decisões do mini-plano (2026-10-19):** o EFX entra e sai como as cenas (fade a partir da posição de antes, volta ao que está embaixo); o relativo desenha em volta das criações rodando e da prévia; "Defasadas" a 1/n da volta; editor sem "Aleatória", com largura e altura até 127 e "Selecionar grupo" em "Adicionar fixture".
- **Ref:** `engine/src/{efx,efxfixture,fixturegroup,grouphead}.cpp`

Validação (detalhada em [docs/fases/fase-08.md](docs/fases/fase-08.md)):
- [x] Um grupo em "Adicionar fixture" inclui as fixtures na ordem da grade.
- [x] Os itens da F7, testados junto (pedido do usuário), menos o deslizar dos Beams, a sequência e a coleção, que estão na validação final.
- Na [validação final](docs/validacao-final.md), com os moving beams: o círculo em 4 moving heads com offset de 90°, o EFX relativo sobre uma posição do programador (a prévia) e de uma cena, e os outros testes do EFX.

### Ajustes rápidos · ✅

Mini-plano: [docs/fases/ajustes-rapidos.md](docs/fases/ajustes-rapidos.md) (aprovado e entregue em 2026-10-21). Validado pelo usuário em 2026-10-21. Dois pedidos do usuário, da pesquisa da grandMA, antes da F10.

- **Localizar (P6):** enquanto ligado, as outras fixtures ficam com a intensidade reduzida (padrão 20%, ajustável no ⚙; com 0% vira um "solo") e só as localizadas ficam em 100%.
- **Fader de BPM (E3):** na seção master, ao lado do TAP, com o FINO. O celular e o tablet ganham um knob.

Validação:
- [x] O Localizar deixa só a fixture escolhida em 100% e as outras na intensidade ajustada.
- [x] O fader de BPM ajusta (também o FINO), e o TAP move o fader.
- [x] `pnpm check` passa.

### F9 — RGB Matrix · 🟡 (entregue em 2026-10-27, aguardando a validação; mini-plano em [docs/fases/fase-09.md](docs/fases/fase-09.md))

- **Algoritmos nativos:** Plain, Text e Image em Rust.
- **Scripts do QLC+:** rodando em rquickjs, numa thread separada com limite de tempo e cache dos passos.
- **Ponto de partida (F8):** o RGB Matrix entra como mais um tipo da árvore `Program` do motor, como o EFX (`efx.rs`), traduzido em `running_programs` (`apply.rs`). Os grupos de fixtures já chegam conferidos pelo servidor (cada cabeça uma vez, dentro da grade).
- **Ref:**
  - `engine/src/{rgbmatrix,rgbalgorithm,rgbscript,rgbscriptscache}.cpp`
  - `resources/rgbscripts/`

Entregue em 2026-10-27, com as sete decisões do mini-plano (prévia pelo servidor, entrar e sair pela regra das cenas, scripts dentro do projeto, JS numa thread à parte com limites, sem Shutter, sem ordem Aleatória, Text e Image com fonte própria e imagem no projeto). Projeto na versão 11. Os 60 scripts do QLC+ abrem e rodam; os exemplos (G8) estão na pasta "RGB Matrix" do "Show de exemplo".

Validação (detalhada em [docs/fases/fase-09.md](docs/fases/fase-09.md#validação-com-você), junto com a da F10c e a da F10d):
- [ ] Cor fixa, degradê, onda, faíscas, texto e imagem acendem os ParLeds, as barras e o painel 8 × 8, e a prévia do editor mostra o mesmo.
- [ ] Importar scripts do QLC+ (vários `.js` de uma vez) e rodá-los; a prévia bate com a do QLC+.
- [ ] Um script em laço infinito é abortado sem derrubar o motor, e o editor mostra o erro.
- [ ] O TAP do BPM e o elemento "Tempos" mudam a velocidade do passo.

### F9b — Efeitos por passos · ⬜ (nova, 2026-10-21)

- **P4 (da pesquisa da grandMA):** um efeito em loop que mexe em **qualquer atributo** (dimmer, cor, pan, tilt, gobo, zoom). Escolhe-se a forma (seno, dente de serra, pulso, círculo), a velocidade (em BPM, seguindo o master) e a fase de cada fixture (o deslocamento dentro do ciclo). Cada passo pode ter valores absolutos ou relativos.
- **Como entra:** um tipo novo de criação do Studio, com ícone próprio, na árvore `Program` do motor e reaproveitando a defasagem do EFX. Não muda o desenho da F9. Aponta para grupos de fixtures (F10b).
- **Ref:** `engine/src/efx.rs` e a base do EFX do projeto (`efx.rs`); o catálogo em [docs/pesquisa-grandma.md](docs/pesquisa-grandma.md) (P4).

Validação:
- [ ] Uma onda de dimmer nos ParLeds, cada um deslocado no ciclo.
- [ ] O TAP do BPM muda a velocidade.
- [ ] Pan e tilt: na [validação final](docs/validacao-final.md), com os Beams.

## Etapa D — Operação ao vivo

### F10 — Virtual Console I · ✅

Mini-plano: [docs/fases/fase-10.md](docs/fases/fase-10.md) (aprovado e entregue em 2026-10-21). Validada pelo usuário em 2026-10-21.

- **Base:** registro de widgets com propriedades geradas por schema.
- **Edição:** grade com snap e edição por toque, modos Edição e Operação, páginas e frames.
- **Widgets:** Button (Toggle/Flash/Blackout/StopAll), Slider (Level/Playback/Submaster), Label.
- **Entradas externas:** modelo `InputRef` (controle MIDI ou tecla do teclado), definido na edição de cada elemento.
- **Da pesquisa da grandMA (2026-10-21):**
  - **E1:** mais funções nos botões (voltar, pausar, "Temp", "Kill").
  - **E2:** o fader de Master e o de **Rate**, que acelera ou desacelera todos os fades e delays ao vivo sem mudar o gravado.
  - **R7:** ligar e desligar a criação pelo fader (sai do zero, liga; volta a zero, desliga).
  - Os sliders (nível, playback e submaster) e o blackout de fixtures passam a acender de verdade.
- **Ref:**
  - `qmlui/virtualconsole/{vcwidget,vcbutton,vcslider,vcframe,vcpage,vclabel}.cpp`
  - `engine/src/qlcinputsource.cpp`

Validação:
- [x] Console de 2 páginas montado no desktop e operado no celular.
- [x] Flash solta ao levantar o dedo e ao cair a conexão.
- [x] Submaster escala uma cena.
- [x] Rate acelera e desacelera os fades de uma cena rodando, sem mudar os gravados.
- [x] Ligar e desligar uma criação pelo fader.

### F10b — Rig flexível · ✅ (nova, 2026-10-21)

Mini-plano: [docs/fases/fase-10b.md](docs/fases/fase-10b.md) (aprovado e entregue em 2026-10-21). Validada pelo usuário em 2026-10-21.

- **N1 (pedido do usuário):** as criações deixam de depender de quantas fixtures há. As cenas, os EFX e os grupos de canais passam a apontar para **grupos de fixtures**: o valor gravado para "ParLeds" vale para quem está no grupo agora. Um valor de uma fixture específica continua possível e vence o do grupo. O RGB Matrix já funciona assim.
- **Resultado:** ir de 8 para 12 ParLeds é patchar as 4 e pô-las no grupo, sem editar as criações.
- **Onde mexe:** o contrato (nova versão do projeto, com migração), a tradução em `running_programs` e `scene_program` (`apply.rs`) e os editores de cena e de EFX. O motor a 50 Hz não muda.
- **Na validação (pedidos do usuário):** a defasagem entre as fixtures de um grupo no EFX, a fixture sozinha como exceção do grupo no EFX e o topo do editor de cena reorganizado (ações numa linha; grupos e fixtures num painel, acesos). Os outros pedidos do EFX foram para a F10d.
- **Projetos do usuário:** o "RSena Palco" e o "Show de exemplo" foram convertidos para os grupos pela tela, com a luz de cada cena igual antes e depois.
- Detalhes: [docs/pesquisa-grandma.md](docs/pesquisa-grandma.md) (N1).

Validação:
- [x] De 8 para 12 ParLeds: tirar e pôr ParLeds no grupo muda as cenas, o slider e a Mesa simples, sem editar as criações.
- [x] Um valor de uma fixture específica vence o do grupo (na cena e no EFX).
- [x] Os projetos antigos abrem (migração).
- [x] `pnpm check` passa.
- Com os Beams de verdade, o EFX em grupo fica na [validação final](docs/validacao-final.md).

### F10c — Tempos, cue list e prioridade · ✅ (nova, 2026-10-21; a validação vai junto com a da F10d, pedido do usuário em 2026-10-22)

Mini-plano: [docs/fases/fase-10c.md](docs/fases/fase-10c.md) (aprovado e entregue em 2026-10-21, com as quatro recomendações; aguardando a validação). Os exemplos de cada recurso (G8) estão no "Show de exemplo": a pasta "Tempos e prioridade", o chaser "Roteiro manual (Go)" e a página "Tempos" do console.

- **R1:** cue list com Go, voltar, ir para, **pausar** e **carregar a próxima**; gatilhos por passo (manual, depois de X segundos, quando o anterior termina, nas batidas); **hold infinito** (o passo espera o Go); Off no fim e volta ao começo.
- **R2:** **delay** de entrada e de saída em cada passo e em cada cena, e curvas além da linear (lenta, rápida, S). Muda a decisão da F6 (fades lineares).
- **R3:** tempos por tipo de atributo na mesma cena ou passo (posição lenta, cor rápida).
- **R6:** **prioridade por criação**: a de nível maior ganha da menor quando disputam o mesmo canal, não importa quem começou por último (uma cena base nunca cobre um efeito). Sem prioridade escolhida, vale a regra de hoje (HTP e LTP).
- **Onde mexe:** o contrato (nova versão do projeto, com migração), o motor (`chaser.rs`, os fades e a mistura), o elemento Cue List e o elemento "Tempos".
- **Ref:** `engine/src/{chaser,chaserrunner,chaserstep,fadechannel}.cpp` e o catálogo ([docs/pesquisa-grandma.md](docs/pesquisa-grandma.md): R1, R2, R3 e R6).

Validação:
- [ ] Um passo espera o Go (hold infinito), e o Pause segura um fade.
- [ ] Delay de saída diferente do de entrada; a curva S se vê no fade.
- [ ] Cor e dimmer com tempos diferentes na mesma cena.
- [ ] Uma cena de prioridade baixa nunca cobre uma de prioridade alta no mesmo canal.
- [ ] Posição lenta com cor rápida: na [validação final](docs/validacao-final.md), com os Beams.

### F10d — EFX avançado · 🟡 (nova, 2026-10-21, pedido do usuário na validação da F10b; entregue, aguardando a validação)

Mini-plano: [docs/fases/fase-10d.md](docs/fases/fase-10d.md) (aprovado e entregue em 2026-10-22, com as quatro recomendações; aguardando a validação). A validação da F10c entra no fim desta, numa rodada só ([checklist](docs/fases/fase-10d.md#validação-com-você)). Os exemplos (G8) estão na pasta "EFX avançado" do "Show de exemplo".

- **Parâmetros por item:** o desenho, a posição, o tamanho e os outros parâmetros do EFX valem para todos, e cada item da lista (fixture ou grupo) pode trocar os que quiser, como a exceção na cena. Ex.: os Beams da esquerda num círculo pequeno e os da direita num oito grande, no mesmo EFX.
- **Desenho personalizado:** desenhar o caminho com o dedo ou o mouse na área da prévia; as fixtures percorrem o desenho.
- **Curva de velocidade:** como a velocidade varia ao longo da volta (constante, acelera, desacelera, rápida no meio e lenta nas pontas, ou uma curva desenhada), reaproveitando as curvas dos fades da F10c (R2).
- **Inverter o pan e o tilt (pedido do usuário, 2026-10-21):** cada item do EFX (fixture ou grupo) pode inverter o pan, o tilt ou os dois, e o desenho sai espelhado naquele eixo (ex.: Beams pendurados de cabeça para baixo, ou um de frente para o outro, desenhando para o mesmo lado). O mini-plano estuda também trocar um eixo pelo outro (a fixture montada de lado).
- **Já entregue na F10b:** a defasagem entre as fixtures de um grupo.
- **Onde mexe:** o contrato (nova versão do projeto, com migração), o motor (`efx.rs`), a tradução no servidor, o editor de EFX e a prévia (as mesmas contas em `utils/functions.ts`).

Validação:
- [ ] Dois itens do mesmo EFX com desenhos diferentes.
- [ ] Um desenho feito à mão percorrido pelas fixtures (na 3D View).
- [ ] A curva de velocidade se vê no movimento.
- [ ] Um item com o pan invertido desenha espelhado (na prévia do EFX e na 3D View).
- [ ] Os itens da F10c (pedido do usuário: as duas validações numa rodada só).
- [ ] Com os Beams de verdade: na [validação final](docs/validacao-final.md).

### F11 — Virtual Console II, multi-cliente e segurança · ⬜

- **Widgets:** XYPad, SoloFrame, SpeedDial, CueList, Clock, Animation.
- **Acesso:** papéis admin/operador/visualizador, PIN → token, checagem de Origin.
- **Da pesquisa da grandMA (2026-10-21):** **E6**, as macros: um botão que executa uma lista de ações em ordem, com espera entre elas.
- **Ref:**
  - `qmlui/virtualconsole/{vcxypad,vcsoloframe,vcspeeddial,vccuelist,vcclock,vcanimation}.cpp`
  - `webaccess/src/webaccessauth.cpp`

Validação:
- [ ] 3 aparelhos por 5 min com estado consistente.
- [ ] PIN errado recusado.
- [ ] Operador não edita.
- [ ] Reconexão recebe o estado completo.

### F12 — MIDI · ⬜

- **Base:** midir com hot-plug e learn.
- **Mapeamento:** na edição de cada elemento do console (E12), com "aprender", como no QLC+. O mesmo vale para as teclas do teclado.
- **Mensagens:** CC, Note, Program Change, Pitch Bend.
- **Da pesquisa da grandMA (2026-10-21):** **S6**, o MIDI Show Control (MSC): outro equipamento manda GO, STOP e RESUME por uma mensagem MIDI padrão, e o app os leva às criações e às cue lists.
- **Feedback:** LEDs e faders motorizados.
- **Opcional:** importar perfis .qxi.
- **Ref:**
  - `plugins/midi/src/common/{midiprotocol,midiinputdevice,midioutputdevice}.cpp`
  - `engine/src/{qlcinputprofile,inputpatch}.cpp`

Validação:
- [ ] Learn em menos de 10 s.
- [ ] LED acompanha um toggle acionado pelo celular.
- [ ] Mapeamentos sobrevivem a replugar.
- [ ] Um MSC "GO" de um programa de teste dispara a criação ligada.

### F12b — Paletas e gravação · ⬜ (nova, 2026-10-21)

- **P1, as paletas:** uma "receita" com nome (uma cor, uma posição, um gobo). As cenas guardam a referência: corrigir a paleta uma vez corrige todas as cenas que a usam. Paleta por fixture, por modelo e universal.
- **P5, gravar a partir da saída e "Update":** gravar numa cena o que está saindo na luz agora (criações, Mesa simples e o resto juntos), o "dump" que estava no Backlog. O "Update" grava de volta, na criação que está rodando, um ajuste feito ao vivo. Ao gravar sobre algo que já existe: perguntar, sobrescrever, mesclar ou remover.
- **P7, filtros de atributo:** ao gravar ou aplicar, escolher quais tipos de atributo valem (só a posição, só a cor). Sem os "Worlds", que dependiam do multiusuário.
- **Onde mexe:** o contrato (nova versão do projeto, com migração), o editor de cena e o Studio.
- Catálogo: [docs/pesquisa-grandma.md](docs/pesquisa-grandma.md) (P1, P5 e P7).

Validação:
- [ ] Mudar uma paleta muda todas as cenas que a usam.
- [ ] Gravar numa cena o que está saindo, com cenas e a Mesa simples juntas.
- [ ] O "Update" grava um ajuste ao vivo na cena que está rodando.
- [ ] Um filtro "só posição" não mexe nas cores da cena.

### F12c — Seleção · ⬜ (nova, 2026-10-21)

- **P2, a ordem de seleção e o leque:** a ordem em que as fixtures foram escolhidas passa a contar, e o leque distribui um valor (pan, intensidade, cor) entre dois extremos ao longo dessa ordem.
- **P3, a divisão da seleção:** "as duas metades", "asas" (de cada ponta para o centro), "uma por vez" (próxima e anterior), embaralhar de forma repetível e deslocar. Também espalhar tempo ou fase pela seleção (cada fixture começa um pouco depois da anterior).
- **P9, o ID numérico:** cada fixture ganha um número, e se escolhem fixtures por faixa ("1 até 12").
- **Onde mexe:** o editor de cena (a seleção) e o contrato (o número da fixture, com migração).
- Catálogo: [docs/pesquisa-grandma.md](docs/pesquisa-grandma.md) (P2, P3 e P9).

Validação:
- [ ] Um leque de intensidade e de cor nos ParLeds.
- [ ] As "asas" e o "uma por vez" percorrem a seleção.
- [ ] Escolher "1 até 12" pelo número.
- [ ] O leque de pan: na [validação final](docs/validacao-final.md), com os Beams.

## Etapa E — Ferramentas e visualização

### F13 — Editor de fixtures · ⬜

- Informações gerais, canais (grupo/preset e capabilities com validação de faixas), modos, cabeças e dados físicos.
- **Ref:** `qmlui/fixtureeditor/{channeledit,modeedit,physicaledit}.cpp`

Validação:
- [ ] Criar do zero um par RGBW de 7 canais, patchar e controlar.

### F13b — Intercâmbio: GDTF, MVR e importação parcial · ⬜ (nova, 2026-10-21)

- **S2, GDTF:** importar fixtures em **GDTF**, o formato aberto de fixture do mercado, para a biblioteca, além do `.qxf` (muda a decisão E3).
- **S2, MVR:** importar o patch e as posições de um desenho de palco (Vectorworks e similares): cria as fixtures, os endereços e as posições da 3D View. O MVR traz as fixtures em GDTF.
- **S4, importação parcial:** trazer só partes de outro show (patch, criações, grupos, console), com regras para o que já existe, no lugar do "Abrir", que troca o projeto inteiro.
- **Ref:** as páginas de GDTF, MVR e importação parcial da grandMA não foram lidas na pesquisa; o mini-plano estuda os formatos antes. Catálogo: [docs/pesquisa-grandma.md](docs/pesquisa-grandma.md) (S2 e S4).

Validação:
- [ ] Importar uma fixture em GDTF e controlá-la.
- [ ] Importar um MVR com fixtures e posições.
- [ ] Trazer só as cenas de outro show para o projeto aberto.

### F14 — Visualização 2D · ⬜

- **Vistas:** topo, frente e lateral, com posições salvas no projeto.
- **Dados:** o servidor calcula o estado visual de cada cabeça e envia em binário a 30 Hz. O mesmo adaptador serve ao 3D.
- **Da pesquisa da grandMA (2026-10-21):**
  - **E5**, o mapa do palco clicável: tocar nas fixtures e nos grupos do desenho 2D para escolhê-los.
  - **P8**, a planilha de fixtures: tabela de fixtures por atributos (dimmer, cores, pan, tilt…), com os valores ao vivo e edição direta. Usa o mesmo cálculo do estado de cada cabeça.
- **Ref:**
  - `qmlui/{mainview2d,previewcontext}.cpp`
  - `ui/src/monitor/monitorfixtureitem.cpp`

Validação:
- [ ] Um chaser aparece corretamente.
- [ ] 100 fixtures a 30 fps no celular.
- [ ] Tocar numa fixture do mapa a escolhe no editor de cena.
- [ ] A planilha mostra os valores ao vivo e edita um valor.

### F15 — Visualização 3D · ⬜

- **Render:** TresJS com loop imperativo.
- **Modelos:** glTF convertidos dos .dae do QLC+ (nós base/arm/head).
- **Feixes:** cones aditivos instanciados, e gobos.
- **Qualidade:** níveis configuráveis.
- **Câmera e palco:** presets de câmera e posicionamento das fixtures.
- **Ref:**
  - `qmlui/mainview3d.cpp`
  - `qmlui/qml/fixturesfunctions/3DView/`
  - `resources/meshes/fixtures/`

Validação:
- [ ] 48 fixtures a 60 fps no desktop e ≥30 fps no celular (qualidade baixa).
- [ ] Pan/tilt conferem com a fixture real.

## Etapa F — Entrega

### F16 — Release v1.0 · ⬜

- **Instalador:** bootstrapper do WebView2 e regra de firewall.
- **Diagnóstico:** log de crash e visualizador de logs.
- **Teste de resistência:** 2 h com 100 fixtures em 4 universos.
- **Documentação:** licenças e manual em pt-BR.

Validação:
- [ ] Instalação limpa (inclui o teste do instalador num PC limpo, adiado da F3).
- [ ] A [validação final](docs/validacao-final.md): os testes com os moving beams, adiados da F6, da F7 e da F8.
- [ ] Show de 2 h sem falhas.
- [ ] Desinstalação limpa.

## Backlog v2

- Android como host USB.
- Art-Net/sACN e OSC.
- Show/timeline com áudio.
- Paletas: saíram do Backlog e entram na F12b (2026-10-21).
- Só se o usuário pedir (explicados a ele na F5):
  - modificadores de canal (curvas por canal);
  - HTP/LTP forçado por canal;
  - fixtures que atravessam universos;
  - o "dump" do QLC+ (gravar a saída numa cena).
- Ficou de fora da F7 (só se o usuário pedir):
  - tempo em batidas nas cenas (o editor de cena só tem segundos);
  - BPM vindo de fora (MIDI clock ou áudio): pode entrar na F12.
  - Pausar um chaser e o hold infinito (∞) entraram na F10c (2026-10-21).
- Ficou de fora da F8 (só se o usuário pedir):
  - os modos Dimmer e RGB do EFX (o desenho no dimmer ou nas cores) e o "controle do dimmer" do QLC+ 5.3;
  - EFX em fixtures com várias cabeças de pan e tilt (hoje vale a primeira cabeça de cada uma);
  - a volta do EFX em batidas no próprio editor (hoje, só pelo elemento "Tempos" com "Seguir o BPM");
  - "Distribuir os inícios" do QLC+ 5 (0°, 90°, 180°… automático ou sorteado);
  - o "Início" geral do EFX, além do de cada fixture.
- RDM e o segundo universo da Enttec Pro Mk2.
- **Fora por ora (2026-10-21; catálogo em [docs/pesquisa-grandma.md](docs/pesquisa-grandma.md)):** a Art-Net e a sACN (S1: a saída é um `Device` em `rlc-dmx`, então entra depois sem mexer na estrutura), as páginas do console (E4), a linha de comando (E7), o multiusuário (S3), o tracking e as partes de cue (R4 e R5), o backup entre máquinas (S5), o timecode (S6), os plugins em Lua (S7) e o RDM (S8).

## Pedidos de recursos do Virtual Console (G4)

- **VC-1 (2026-10-16): elementos configuráveis pelo usuário.** O console é montado pelo usuário, que posiciona e ajusta cada elemento a seu gosto, como no QLC+. Na edição de cada botão ele escolhe:
  - destaque (botão aceso ou LED), cor e ícone (opcional, da biblioteca);
  - modo (liga/desliga ou flash);
  - ação: função, blackout geral, blackout de fixtures ou grupos específicos, ou stop all;
  - entrada externa: controle MIDI ou tecla do teclado.

  Protótipo na F2; implementação na F10/F11. O blackout por fixtures ou grupos não existe no QLC+.
- **VC-2 (2026-10-16): layouts por formato de tela.** O usuário cria e ajusta o console para formatos diferentes (celular, tablet, desktop), e cada aparelho abre o layout configurado para o seu formato. Os elementos são os mesmos em todos os formatos (só a posição muda), e o formato vem da largura da tela, com troca no ⚙ (E11). Protótipo na F2 (rodada 3); implementação na F10.
- **VC-3 (2026-10-16): fades ajustados ao vivo.** No console, um controle (fader ou knob) define na hora os tempos de fade das criações disparadas, para o operador ajustar conforme o momento do show pede. Os fades gravados nas cenas ficam como padrão. No QLC+, o mais próximo é o widget Speed Dial. No protótipo, é o elemento "Tempos", com a lista das criações que controla (E11). Protótipo na F2 (rodada 3). O fade in e o fade out das cenas foram ligados na F6 (2026-10-18, pedido do usuário); a duração entrou na F7, com os chasers. Na validação da F7 (2026-10-19), o elemento passou a seguir o BPM master ("Seguir o BPM"), com os tempos em batidas e o TAP do master (E11).

- **VC-4 (2026-10-20): atalhos do teclado na edição.** Com elementos escolhidos: Delete remove, Ctrl + D duplica, Ctrl + C e Ctrl + V copiam e colam, e as setas deslocam (2 células; com Shift, 4; com Ctrl, 1). Ver E12. Pedidos da mesma data, fora do console (E12): grupos de fixtures e de canais reordenáveis por arrasto e duplicáveis; pastas dentro de pastas no Studio.

---

## Riscos

| Risco | Mitigação |
|---|---|
| Jitter no Open DMX | Thread TIME_CRITICAL, prazo absoluto, medição de jitter na F3 |
| Semântica diferente do QLC+ | Portar `engine/test` como especificação, relógio determinístico |
| Sincronização entre vários clientes | Servidor como autoridade, revisões, trava do controle sob o dedo, snapshot ao reconectar |
| Segurança na rede local | Checagem de Origin, PIN → token, papéis, bind configurável |
| Celular em HTTP não é "contexto seguro" | Workaround estilo NoSleep. O MIDI fica no servidor |
| Desempenho no celular | Frames binários, decimação, níveis de qualidade, spike 3D na F1 |
| Curva de Rust | Crates pequenos, código direto (G7), testes legíveis, uma demo por fase |
| Crescimento de escopo do VC | Widgets por schema. Cada pedido vira mini-plano e decisão |

## Verificação

- **Automática:** `pnpm check` (rustfmt, clippy `-D warnings`, `cargo test`, typecheck e lint da UI).
- **Manual:** checklist da fase, rodando `pnpm dev` no desktop e no celular pelo QR. Hardware real a partir da F3. Os testes com os moving beams ficam na [validação final](docs/validacao-final.md) enquanto o usuário não puder fazê-los (G1).
- **Encerramento:** checklist completo e aprovação explícita do usuário.

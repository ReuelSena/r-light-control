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

## Pendências (2026-10-19)

O que falta validar ou decidir nas fases já entregues. O detalhe de cada item está no checklist da fase.

**A validar sem os moving beams** (com os ParLed e a Numbstar):
- **F6** ([fase-06.md](docs/fases/fase-06.md)):
  - o tick abaixo de 2 ms com 4 universos (no Monitoramento);
  - os fades de 3 s nos ParLed;
  - o elemento "Tempos" nas cenas.
- **F7** ([fase-07.md](docs/fases/fase-07.md)): o checklist inteiro, menos o deslizar dos Beams:
  - o crossfade sem o dimmer piscar;
  - as ordens, o Single Shot e "Para trás";
  - os modos dos tempos e o tap a 120 BPM;
  - a sequência, a coleção e a cue list;
  - o "Tempos" com "Seguir o BPM";
  - a edição com o chaser rodando, o Stop All e o pânico.
- **F8** ([fase-08.md](docs/fases/fase-08.md)): "Selecionar grupo", em "Adicionar fixture", inclui as fixtures na ordem da grade.

**Com os moving beams:** na [validação final](docs/validacao-final.md), antes do release: 3 itens da F6, 1 da F7 e 7 da F8 (o EFX).

**Na F16:** o instalador num PC limpo (adiado da F3).

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
- [ ] Tick abaixo de 2 ms com 4 universos. _Ainda não testado (o teste automático mede 0,2 ms no app compilado)._
- [ ] O elemento "Tempos" troca o fade in e o fade out das cenas da lista (ajuste pedido na validação). _Ainda não testado; dá para conferir na validação da F7._

### F7 — Chaser, Sequência, Coleção e BPM · ✅

Mini-plano: [docs/fases/fase-07.md](docs/fases/fase-07.md) (aprovado em 2026-10-18; entregue em 2026-10-18). Concluída em 2026-10-19, a pedido do usuário, sem a validação: os itens abaixo serão testados junto com a F8.

- **Chaser:** modos de velocidade Default/Common/PerStep, crossfade, next/prev/goto, run order Loop/SingleShot/PingPong/Random.
- **Sequência** e **Coleção.**
- **Tempo:** BPM global com tap tempo.
- **Fim do Single Shot (pedido do usuário, 2026-10-18):** opção do chaser: apagar depois do último passo ou manter o último passo aceso.
- **Cue list:** ligada ao chaser de verdade já nesta fase (play, stop, anterior, próximo e toque num passo).
- **Ajuste pedido na validação (2026-10-19):** o elemento "Tempos" segue o BPM master ("Seguir o BPM", ligada por padrão): tempos em batidas e um TAP só, o do master.
- **Ref:** `engine/src/{chaser,chaserrunner,chaserstep,sequence,collection,mastertimer}.cpp`

Validação (detalhada em [docs/fases/fase-07.md](docs/fases/fase-07.md)):
- [ ] Chaser de 8 passos com crossfade de 2 s, suave.
- [ ] Tap a 120 BPM mantém o tempo.
- [ ] PingPong e Random corretos.
- [ ] O TAP master muda os tempos de todos os elementos "Tempos" que seguem o BPM, e os chasers da lista acompanham (ajuste pedido na validação).

### F8 — Grupos de fixtures e EFX · 🟡

Mini-plano: [docs/fases/fase-08.md](docs/fases/fase-08.md) (aprovado em 2026-10-19; entregue em 2026-10-19, aguardando a validação do usuário, junto com os itens da F7). Commitada em 2026-10-19, a pedido do usuário, antes da validação: o que falta está nas [Pendências](#pendências-2026-10-19). Os testes com os moving beams foram para a [validação final](docs/validacao-final.md) (2026-10-19, pedido do usuário).

- **Grupos:** grupos 2D de cabeças.
- **EFX:** os 10 algoritmos, modo absoluto e relativo, spread/offset, preview do caminho.
- **Decisões do mini-plano (2026-10-19):** o EFX entra e sai como as cenas (fade a partir da posição de antes, volta ao que está embaixo); o relativo desenha em volta das criações rodando e da prévia; "Defasadas" a 1/n da volta; editor sem "Aleatória", com largura e altura até 127 e "Selecionar grupo" em "Adicionar fixture".
- **Ref:** `engine/src/{efx,efxfixture,fixturegroup,grouphead}.cpp`

Validação (detalhada em [docs/fases/fase-08.md](docs/fases/fase-08.md)):
- [ ] Um grupo em "Adicionar fixture" inclui as fixtures na ordem da grade.
- [ ] Os itens da F7, testados junto (pedido do usuário), menos o deslizar dos Beams.
- Na [validação final](docs/validacao-final.md), com os moving beams: o círculo em 4 moving heads com offset de 90°, o EFX relativo sobre uma posição do programador (a prévia) e de uma cena, e os outros testes do EFX.

### F9 — RGB Matrix · ⬜

- **Algoritmos nativos:** Plain, Text e Image em Rust.
- **Scripts do QLC+:** rodando em rquickjs, numa thread separada com limite de tempo e cache dos passos.
- **Ponto de partida (F8):** o RGB Matrix entra como mais um tipo da árvore `Program` do motor, como o EFX (`efx.rs`), traduzido em `running_programs` (`apply.rs`). Os grupos de fixtures já chegam conferidos pelo servidor (cada cabeça uma vez, dentro da grade).
- **Ref:**
  - `engine/src/{rgbmatrix,rgbalgorithm,rgbscript,rgbscriptscache}.cpp`
  - `resources/rgbscripts/`

Validação:
- [ ] Os scripts carregam.
- [ ] Preview 8×8 bate com o devtool do QLC+.
- [ ] Um script em loop infinito é abortado sem derrubar o motor.

## Etapa D — Operação ao vivo

### F10 — Virtual Console I · ⬜

- **Base:** registro de widgets com propriedades geradas por schema.
- **Edição:** grade com snap e edição por toque, modos Edição e Operação, páginas e frames.
- **Widgets:** Button (Toggle/Flash/Blackout/StopAll), Slider (Level/Playback/Submaster), Label.
- **Entradas externas:** modelo `InputRef` (controle MIDI ou tecla do teclado), definido na edição de cada elemento.
- **Ref:**
  - `qmlui/virtualconsole/{vcwidget,vcbutton,vcslider,vcframe,vcpage,vclabel}.cpp`
  - `engine/src/qlcinputsource.cpp`

Validação:
- [ ] Console de 2 páginas montado no desktop e operado no celular.
- [ ] Flash solta ao levantar o dedo e ao cair a conexão.
- [ ] Submaster escala uma cena.

### F11 — Virtual Console II, multi-cliente e segurança · ⬜

- **Widgets:** XYPad, SoloFrame, SpeedDial, CueList, Clock, Animation.
- **Acesso:** papéis admin/operador/visualizador, PIN → token, checagem de Origin.
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
- **Feedback:** LEDs e faders motorizados.
- **Opcional:** importar perfis .qxi.
- **Ref:**
  - `plugins/midi/src/common/{midiprotocol,midiinputdevice,midioutputdevice}.cpp`
  - `engine/src/{qlcinputprofile,inputpatch}.cpp`

Validação:
- [ ] Learn em menos de 10 s.
- [ ] LED acompanha um toggle acionado pelo celular.
- [ ] Mapeamentos sobrevivem a replugar.

## Etapa E — Ferramentas e visualização

### F13 — Editor de fixtures · ⬜

- Informações gerais, canais (grupo/preset e capabilities com validação de faixas), modos, cabeças e dados físicos.
- **Ref:** `qmlui/fixtureeditor/{channeledit,modeedit,physicaledit}.cpp`

Validação:
- [ ] Criar do zero um par RGBW de 7 canais, patchar e controlar.

### F14 — Visualização 2D · ⬜

- **Vistas:** topo, frente e lateral, com posições salvas no projeto.
- **Dados:** o servidor calcula o estado visual de cada cabeça e envia em binário a 30 Hz. O mesmo adaptador serve ao 3D.
- **Ref:**
  - `qmlui/{mainview2d,previewcontext}.cpp`
  - `ui/src/monitor/monitorfixtureitem.cpp`

Validação:
- [ ] Um chaser aparece corretamente.
- [ ] 100 fixtures a 30 fps no celular.

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
- Paletas.
- Só se o usuário pedir (explicados a ele na F5):
  - modificadores de canal (curvas por canal);
  - HTP/LTP forçado por canal;
  - fixtures que atravessam universos;
  - o "dump" do QLC+ (gravar a saída numa cena).
- Ficou de fora da F7 (só se o usuário pedir):
  - pausar um chaser;
  - tempo em batidas nas cenas (o editor de cena só tem segundos);
  - hold infinito (∞), para o chaser só andar no "próximo" da cue list: pode entrar na F11, com o resto da Cue List;
  - BPM vindo de fora (MIDI clock ou áudio): pode entrar na F12.
- Ficou de fora da F8 (só se o usuário pedir):
  - os modos Dimmer e RGB do EFX (o desenho no dimmer ou nas cores) e o "controle do dimmer" do QLC+ 5.3;
  - EFX em fixtures com várias cabeças de pan e tilt (hoje vale a primeira cabeça de cada uma);
  - a volta do EFX em batidas no próprio editor (hoje, só pelo elemento "Tempos" com "Seguir o BPM");
  - "Distribuir os inícios" do QLC+ 5 (0°, 90°, 180°… automático ou sorteado);
  - o "Início" geral do EFX, além do de cada fixture.
- RDM e o segundo universo da Enttec Pro Mk2.

## Pedidos de recursos do Virtual Console (G4)

- **VC-1 (2026-10-16): elementos configuráveis pelo usuário.** O console é montado pelo usuário, que posiciona e ajusta cada elemento a seu gosto, como no QLC+. Na edição de cada botão ele escolhe:
  - destaque (botão aceso ou LED), cor e ícone (opcional, da biblioteca);
  - modo (liga/desliga ou flash);
  - ação: função, blackout geral, blackout de fixtures ou grupos específicos, ou stop all;
  - entrada externa: controle MIDI ou tecla do teclado.

  Protótipo na F2; implementação na F10/F11. O blackout por fixtures ou grupos não existe no QLC+.
- **VC-2 (2026-10-16): layouts por formato de tela.** O usuário cria e ajusta o console para formatos diferentes (celular, tablet, desktop), e cada aparelho abre o layout configurado para o seu formato. Os elementos são os mesmos em todos os formatos (só a posição muda), e o formato vem da largura da tela, com troca no ⚙ (E11). Protótipo na F2 (rodada 3); implementação na F10.
- **VC-3 (2026-10-16): fades ajustados ao vivo.** No console, um controle (fader ou knob) define na hora os tempos de fade das criações disparadas, para o operador ajustar conforme o momento do show pede. Os fades gravados nas cenas ficam como padrão. No QLC+, o mais próximo é o widget Speed Dial. No protótipo, é o elemento "Tempos", com a lista das criações que controla (E11). Protótipo na F2 (rodada 3). O fade in e o fade out das cenas foram ligados na F6 (2026-10-18, pedido do usuário); a duração entrou na F7, com os chasers. Na validação da F7 (2026-10-19), o elemento passou a seguir o BPM master ("Seguir o BPM"), com os tempos em batidas e o TAP do master (E11).

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

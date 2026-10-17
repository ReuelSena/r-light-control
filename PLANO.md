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

### F5 — Patch, projeto e programador · ⬜

- **Projeto:** fixtures embutidas; patch com conflitos e endereçamento automático.
- **Grupos de canais:** gravados no projeto e usados pelos faders da mesa simples, das cenas (F6) e do console (F10).
- **Arquivos:** salvar/abrir com escrita atômica, autosave e recuperação após crash.
- **Programador:** seleção, intensidade, cor, pan/tilt, capabilities, highlight.
- **Edição:** API por operações tipadas com revisão.
- **Ref:**
  - `engine/src/{doc,fixture,channelmodifier}.cpp`
  - `qmlui/{fixturemanager,contextmanager}.cpp`
  - `qmlui/tardis/tardis.cpp`

Validação:
- [ ] 20 fixtures em 2 universos.
- [ ] Conflito sinalizado.
- [ ] Salvar → reabrir dá um projeto idêntico.
- [ ] Recuperação após matar o processo.
- [ ] Celular e desktop sincronizados.

## Etapa C — Funções

### F6 — Merge, fades e Cena · ⬜

- **Merge:** HTP na intensidade e LTP por ordem de início no resto.
- **Faders:** prioridades, fades e crossfades em f32, canais fine de 16 bits.
- **Grand Master:** Limit/Reduce × Intensity/All.
- **Cena:** gravada do programador e editável enquanto roda. Stop All.
- **Testes:** portados de `engine/test/{universe,genericfader,fadechannel,scene,grandmaster}`.
- **Ref:** `engine/src/{universe,genericfader,fadechannel,function,scene}.cpp`

Validação:
- [ ] Duas cenas sobrepostas: maior dimmer vence; para a cor, vence a última iniciada.
- [ ] GM a 50% reduz só a intensidade.
- [ ] Tick abaixo de 2 ms com 4 universos.

### F7 — Chaser, Sequência, Coleção e BPM · ⬜

- **Chaser:** modos de velocidade Default/Common/PerStep, crossfade, next/prev/goto, run order Loop/SingleShot/PingPong/Random.
- **Sequência** e **Coleção.**
- **Tempo:** BPM global com tap tempo.
- **Ref:** `engine/src/{chaser,chaserrunner,chaserstep,sequence,collection}.cpp`

Validação:
- [ ] Chaser de 8 passos com crossfade de 2 s, suave.
- [ ] Tap a 120 BPM mantém o tempo.
- [ ] PingPong e Random corretos.

### F8 — Grupos de fixtures e EFX · ⬜

- **Grupos:** grupos 2D de cabeças.
- **EFX:** os 10 algoritmos, modo absoluto e relativo, spread/offset, preview do caminho.
- **Ref:** `engine/src/{efx,efxfixture,fixturegroup,grouphead}.cpp`

Validação:
- [ ] Círculo em 4 moving heads com offset de 90°.
- [ ] EFX relativo sobre uma posição do programador.

### F9 — RGB Matrix · ⬜

- **Algoritmos nativos:** Plain, Text e Image em Rust.
- **Scripts do QLC+:** rodando em rquickjs, numa thread separada com limite de tempo e cache dos passos.
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
- [ ] Show de 2 h sem falhas.
- [ ] Desinstalação limpa.

## Backlog v2

- Android como host USB.
- Art-Net/sACN e OSC.
- Show/timeline com áudio.
- Paletas e undo/redo.
- RDM e o segundo universo da Enttec Pro Mk2.

## Pedidos de recursos do Virtual Console (G4)

- **VC-1 (2026-10-16): elementos configuráveis pelo usuário.** O console é montado pelo usuário, que posiciona e ajusta cada elemento a seu gosto, como no QLC+. Na edição de cada botão ele escolhe:
  - destaque (botão aceso ou LED), cor e ícone (opcional, da biblioteca);
  - modo (liga/desliga ou flash);
  - ação: função, blackout geral, blackout de fixtures ou grupos específicos, ou stop all;
  - entrada externa: controle MIDI ou tecla do teclado.

  Protótipo na F2; implementação na F10/F11. O blackout por fixtures ou grupos não existe no QLC+.
- **VC-2 (2026-10-16): layouts por formato de tela.** O usuário cria e ajusta o console para formatos diferentes (celular, tablet, desktop), e cada aparelho abre o layout configurado para o seu formato. Os elementos são os mesmos em todos os formatos (só a posição muda), e o formato vem da largura da tela, com troca no ⚙ (E11). Protótipo na F2 (rodada 3); implementação na F10.
- **VC-3 (2026-10-16): fades ajustados ao vivo.** No console, um controle (fader ou knob) define na hora os tempos de fade das criações disparadas, para o operador ajustar conforme o momento do show pede. Os fades gravados nas cenas ficam como padrão. No QLC+, o mais próximo é o widget Speed Dial. No protótipo, é o elemento "Tempos", com a lista das criações que controla (E11). Protótipo na F2 (rodada 3); implementação na F10/F11, sobre os fades da F6.

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
- **Manual:** checklist da fase, rodando `pnpm dev` no desktop e no celular pelo QR. Hardware real a partir da F3.
- **Encerramento:** checklist completo e aprovação explícita do usuário.

# Protocolo UI ↔ motor (v0)

Montado a partir do que as telas da F2 precisam e revisado pelo usuário em 2026-10-17. Implementado na F3: o servidor ([`crates/rlc-server`](../crates/rlc-server/src/)) é a autoridade, e a UI é um cliente WebSocket ([`ui/app/engine/ws.ts`](../ui/app/engine/ws.ts)).

- **Fonte da verdade:** os tipos ficam em [`crates/rlc-core/src/lib.rs`](../crates/rlc-core/src/lib.rs). A UI usa os mesmos tipos em TypeScript, gerados pelo ts-rs em `ui/app/types/generated/` ao rodar `cargo test -p rlc-core`. Nunca se editam os tipos gerados à mão.
- **Porta única das telas:** [`useEngine()`](../ui/app/composables/useEngine.ts) devolve `{ state, output, send, connected, clientId, error }`.
- **Regras das operações:** [`crates/rlc-server/src/apply.rs`](../crates/rlc-server/src/apply.rs). Cada uma valida, aplica e diz que pedaços do estado mudou. Uma operação recusada não muda nada.

## Modelo (A2, A3)

1. **Ao conectar**, o cliente recebe o `hello` e um `Snapshot` com o estado completo.
2. **Para alterar algo**, o cliente envia uma operação tipada (`Op`). O servidor valida, aplica, aumenta a revisão e avisa **todos** os clientes (janela principal, janelas externas, celulares) com o pedaço do estado que mudou.
3. **O servidor é a autoridade.** A tela não altera o estado sozinha. Só o controle sob o dedo é otimista: fader, knob e pad XY mostram o próprio movimento na hora e voltam ao valor do servidor 300 ms depois do último movimento (`useOptimistic`).
4. **Valores dos universos** vão em frames binários, fora do JSON.

## Envelope WebSocket (`/ws`)

```jsonc
// servidor → cliente, ao conectar
{ "t": "hello", "v": 1, "data": { "appVersion": "0.1.0", "clientId": "0199…" } }
{ "t": "snapshot", "rev": 42, "data": { /* Snapshot */ } }

// cliente → servidor
{ "t": "cmd", "id": 7, "data": { "op": "autoAddress", "ids": ["f1"], "universe": "u1", "start": 1 } }

// servidor → todos: o pedaço do estado que mudou, no caminho dele
{ "t": "evt", "rev": 43, "topic": "project.fixtures", "data": [ /* a lista nova */ ] }

// servidor → cliente que enviou, depois dos evt da operação
{ "t": "ack", "id": 7, "data": null }          // ou a resposta da operação (ex.: ImportReport)
{ "t": "err", "id": 7, "data": "motivo legível" }
```

- **Ordem:** os `evt` de uma operação chegam antes do `ack` dela. Quem espera a resposta já encontra o estado novo.
- **`rev`** é a revisão do estado: cada operação aplicada (e cada atualização do Monitoramento) aumenta uma. Os `evt` de uma mesma operação têm a mesma revisão.
- **Cliente lento:** se ele perder avisos, o servidor manda um `snapshot` novo e continua dali.
- **`err` sem `id`:** mensagem que nem chegou a ser um comando (JSON inválido).
- **`clientId`:** é o id do aparelho em `remote.clients`. É assim que ele se reconhece na lista do ⚙.

**Tópicos do `evt`** (o caminho no `Snapshot`, aplicado no lugar pelo cliente):
- **Projeto:** `project` (projeto inteiro, no "Novo projeto" e no "Abrir"), `project.name` (no "Salvar como"), `project.universes`, `project.fixtureDefs`, `project.fixtures`, `project.fixtureGroups`, `project.channelGroups`, `project.functions`, `project.folders`, `project.console`, `project.stage`.
- **Arquivo e biblioteca:** `projectPath`, `dirty`, `recovered`, `projects` (os projetos da pasta, relidos a cada segundo) e `library`. A biblioteca fica gravada no PC, fora do projeto (um arquivo por definição).
- **Aparelhos e acesso:** `interfaces` e `remote`.
- **Ao vivo:** `running`, `chaserSteps` (o passo de cada chaser rodando, quando muda), `paused`, `loadedSteps`, `desk`, `master`, `widgetValues`, `stats`, `testPattern`, `preview` (a cena em prévia) e `highlight` (o "Localizar": as `fixtures` acesas e a intensidade `others` das outras, de 0 a 255).

`stats` e `interfaces` chegam a cada segundo, com os números do Monitoramento.

## Frames binários

A 25 Hz, só dos universos que mudaram desde o último envio (todos, logo depois de conectar):

| Byte | Conteúdo |
|---|---|
| 0 | Tipo: `1` = valores de um universo |
| 1 | Tamanho do id do universo, em bytes (`n`) |
| 2 … 1+n | Id do universo (UTF-8) |
| 2+n … 513+n | Os 512 valores, do canal 1 ao 512 |

O id vai no frame, e não a posição na lista, para um universo criado ou removido nunca trocar os valores de lugar. No `useEngine()`, os valores ficam em `output[id]`.

## Regras (revisão de 2026-10-17)

- **Edição simultânea:** a última gravação vale, sem trava nem mescla.
- **IDs:** UUIDv7.
  - **Criados pela UI:** a UI gera o id (gerador próprio, porque `crypto.randomUUID` não existe no celular em HTTP), para a tela já escolher o que acabou de criar.
  - **Criados pelo servidor:** `addFixtures` e `addUniverse`, que não recebem id.
  - **Validação:** o servidor recusa um id vazio e ids repetidos na mesma operação (vários elementos do console, elementos do palco). As operações `save…` criam ou substituem pelo id.
- **Estado ao vivo** (criações rodando, Mesa simples, valores do console, seção master) não é salvo no projeto: ao reabrir, começa parado e zerado.
- **Preferências de cada aparelho** (zoom, formato do console, dicas de atalho, qualidade e fumaça do 3D) ficam no navegador, fora do contrato.
- **Acesso:** com "Permitir aparelhos da rede local" desligado, o servidor recusa quem não é este computador (HTTP 403), e os aparelhos de fora já conectados caem na hora. A escolha e o PIN ficam no `settings.json`. O PIN só passa a ser exigido na F11 (PIN → token, papéis).

## Operações v0 (`Op`)

Todas levam o campo `op` com o nome em camelCase e campos em camelCase.

| Operação | Campos | Efeito |
|---|---|---|
| `newProject` | — | Projeto vazio, com um universo na saída Dummy; zera a Mesa simples e o ao vivo |
| `openProject` | `name` | Abre um projeto da pasta pelo nome; solta o ao vivo como o `newProject` e começa um histórico novo. Arquivo inválido ou de versão mais nova é recusado sem mexer no projeto aberto |
| `saveProject` | `name?` | Sem `name`, salva no arquivo atual (a versão anterior vira `.rlc.bak`); com `name`, é o "Salvar como": um projeto novo na pasta (nome repetido é recusado) |
| `undo` / `redo` | — | Desfaz e refaz a última alteração do projeto ou da biblioteca, de qualquer aparelho. A operação ao vivo não entra. Sem passo, recusa ("Nada para desfazer") |
| `addFixtures` | `defId, mode, name, color, count, universe, address?` | Adiciona `count` fixtures; sem `address`, cada uma vai para o primeiro lugar livre |
| `updateFixture` | `fixture` | Substitui a fixture (nome, cor, modo, universo, endereço) |
| `removeFixtures` | `ids` | Remove fixtures e as referências a elas (grupos, cenas, sequências, EFX, console, palco) |
| `autoAddress` | `ids, universe, start` | Endereça em sequência, sem buracos |
| `saveFixtureGroup` / `removeFixtureGroup` | `group` / `id` | Cria ou substitui um grupo de fixtures (grade 2D, de 1 a 64 por lado; recusa célula fora da grade, duas cabeças na mesma célula, cabeça repetida ou que não existe). As cenas e os EFX rodando e os grupos de canais que acompanham o grupo mudam na hora (F10b). Remover congela o grupo onde ele era usado, sem mudar a luz: nas cenas, os canais que mudariam ganham o valor de agora na fixture; o item do EFX vira as fixtures dele (com o "Inverter" e o "Início" do item); o grupo de canais e o botão de blackout de fixtures ficam com as fixtures que estavam nele. Só o RGB Matrix fica sem grupo |
| `saveChannelGroup` / `removeChannelGroup` | `group` / `id` | Cria ou substitui um grupo de canais; remove. Com `fixtureGroup`, os canais são os `attributes` (sem os finos) das fixtures do grupo, na ordem da grade: o servidor refaz `channels` agora e sempre que o grupo de fixtures, as fixtures ou as definições mudam (e avisa `project.channelGroups`). Grupo de fixtures que não existe é recusado |
| `moveFixtureGroup` / `moveChannelGroup` | `id, to` | Leva o grupo para a posição `to` da lista (a partir de 0; passando do fim, vai para o último lugar). A ordem da lista vale em todas as telas (Mesa simples, console, EFX…). Id que não existe é recusado |
| `saveFixtureDef` | `def` | Cria ou substitui uma definição na biblioteca (e no projeto, se ele a usa). O id só tem minúsculas, números e hífens (vira o nome do arquivo) |
| `removeFixtureDef` | `id` | Tira a definição da biblioteca; o projeto continua com a cópia dele |
| `importQxf` | `fileName, content` | Converte o `.qxf` (crate `rlc-qxf`) e põe na biblioteca, como um `saveFixtureDef`: a mesma fixture (fabricante e modelo) substitui a anterior. Resposta: `ImportReport`, com os avisos da conversão. Um arquivo que não é fixture é recusado |
| `addUniverse` / `updateUniverse` / `removeUniverse` | — / `universe` / `id` | Universos e a interface de saída de cada um. Uma interface manda um universo só: escolhida num, sai do outro. Um universo com fixtures não é removido |
| `setRemoteAccess` | `enabled, pin?` | Acesso pela rede local e PIN |
| `saveFunction` / `removeFunction` | `function` / `id` | Cria ou substitui uma criação do Studio (recusa a que contém a si mesma, direto ou por outra, como no QLC+; no EFX, recusa largura ou altura acima de 127, frequência acima de 32, ângulo acima de 359°, a ordem aleatória, item repetido, fixture sem pan e tilt e grupo que não existe, e, no desenho geral e no de cada item, mais de 64 pontos no desenho à mão e a curva de velocidade fora da regra; na cena, grupo repetido ou que não existe); remove (e tira dos passos de chaser, das coleções e do console) |
| `saveFolder` / `removeFolder` | `folder` / `id` | Pasta do Studio (de um tipo só), que pode estar dentro de outra (`parent`, do mesmo tipo; recusa pai que não existe, de outro tipo ou a própria pasta dentro de si ou das pastas dela); remover leva o que estava dentro (criações e pastas) para a pasta de cima, ou solta no tipo. Mover uma criação de pasta é um `saveFunction` com outro `folder`; mover uma pasta, um `saveFolder` com outro `parent` |
| `setFunctionRunning` | `id, running` | Roda ou para uma criação. A cena acende com o fade in e apaga com o fade out; o chaser e a sequência andam pelos passos; a coleção dispara as criações dela; o EFX move o pan e o tilt das fixtures dele. O RGB Matrix acende a partir da F9 |
| `setFunctionPaused` | `id, paused` | Pausa uma criação rodando (F10c): os fades, os delays, o hold e os passos dela, e os EFX e chasers de dentro, param onde estão; continuar segue de onde parou. Parada, a criação não pausa (a operação não faz nada), e a que para deixa de estar pausada |
| `stepChaser` | `id, action` | Cue list: leva um chaser ou uma sequência a outro passo (`"next"`, `"previous"` ou `{ "go": passo }`, a partir de 0), na hora, com os tempos do passo. Parado, ele passa a rodar: "próximo" no primeiro passo e "anterior" no último. Pausado, ele continua. Com um passo carregado (`loadStep`), o "próximo" vai para ele, e ir para o passo carregado o desfaz. A ordem (Loop, Ping Pong, Random) fica com o motor |
| `loadStep` | `id, step?` | Cue list: carrega o passo do próximo Go, sem disparar (F10c); sem `step`, desfaz. Chaser parado: o Go começa nele. Passo fora do chaser é recusado; o passo carregado sai quando o chaser ou o passo deixa de existir |
| `stopAllFunctions` | — | Para todas as criações rodando, cada uma com o seu fade out ("Parar todas" do Studio e Stop All do console), e baixa os sliders de playback |
| `panic` | `all` | Pânico: o mesmo do `stopAllFunctions`, mas na hora, sem fade out. Com `all` (o segundo toque), também zera a Mesa simples inteira e desliga o padrão de teste, a prévia e o "Localizar" |
| `setDeskLevel` | `universe, channel, value` | Fader de um canal da Mesa simples (canal de 1 a 512, valor de 0 a 255) |
| `setDeskGroupLevel` | `group, value` | Fader de um grupo de canais na Mesa simples: grava o nível do grupo e o valor em cada canal dele (os faders de canal acompanham) |
| `resetDesk` | `universe?` | Zera a Mesa simples: os canais do universo ou, sem ele, os grupos de canais (soltando os canais deles) |
| `saveConsolePage` / `removeConsolePage` | `page` / `id` | Página do console; remover leva os elementos dela |
| `saveWidgets` | `widgets` | Cria ou substitui vários elementos do console de uma vez (mover, alinhar, colar, copiar layout) |
| `removeWidgets` | `ids` | Remove elementos; os filhos de uma moldura removida vão junto |
| `setWidgetValue` | `widget, value` | Valor ao vivo de um elemento (ver `widgetValues` abaixo); `value` vazio solta o valor. No slider de playback, sair do zero liga a criação dele e voltar ao zero desliga |
| `pressWidget` | `widget, control, down` | Uma parte de um elemento apertada (`down: true`) ou solta (F10). O servidor aplica o que o elemento faz: no botão, a ação (`function`, `exclusive` = "Só esta", `blackout`, `blackoutFixtures`, `stopAll`, `previous` = o passo anterior do chaser do botão, `pause` = pausa ou continua a criação do botão) no modo dele (`toggle` age ao apertar; `flash` e `temp`, enquanto apertado; `stopAll`, `previous` e `pause` agem ao apertar, em qualquer modo); na cue list (F10c), `main` é o Go (parado, começa, no passo carregado se houver; pausado, continua; rodando, vai para o passo seguinte ou para o carregado), `previous` volta, `pause` pausa ou continua e `stop` para (`next`, de antes, faz o mesmo que o Go rodando). O servidor anota quem está apertando: se o aparelho cair, ele solta. O tap do elemento "Tempos" fica no aparelho |
| `setGrandMaster` / `setBlackout` / `setBpm` / `setTempo` | `value` / `on` / `bpm` / `bpm` | Seção master do console e o BPM global (de 30 a 300). O `setBpm` é o tap: a batida do motor recomeça na hora dele, e o toque conta como a batida mais perto dele. O `setTempo` é o fader: muda o BPM sem recomeçar a batida (a parte da batida que falta continua a mesma) |
| `saveStage` | `stage` | Palco da 3D View: tamanho, posição das fixtures e elementos de palco |
| `setPreview` | `function?` | Cena em prévia na luz (só cenas); sem `function`, desliga. Uma por vez, acima das criações rodando e abaixo da Mesa simples |
| `setHighlight` | `fixtures`, `others` | Fixtures do "Localizar": dimmer e RGBW a 100%, shutter no padrão, acima da Mesa simples; sem fixtures, apaga. `others` (0 a 255, como o Grand Master) é a intensidade (dimmer e cores) de todas as outras fixtures enquanto algo está localizado: 255 não reduz nada e 0 deixa só as localizadas acesas. Pan, tilt e gobo não mudam |
| `setTestPattern` | `pattern?` | Padrão de teste da saída (`ramp` ou `chase`; sem ele, desliga). Página `/dev/dmx` |
| `setInterfaceKind` | `interface, kind?` | Tipo de uma interface USB (`enttecPro` ou `openDmx`) escolhido à mão e guardado no `settings.json`; sem `kind`, volta ao automático. A saída reabre no protocolo novo |

**Console (`Project.console`):** páginas e elementos (`VcWidget`). Cada elemento tem `id`, `page`, `frame` (moldura ou `null`), `caption`, `color`, `inputs` (controle MIDI e tecla por parte do elemento) e `layouts` com a posição em células em cada formato (`phone`, `tablet`, `desktop`; `null` = não aparece nele). O `type` (`button`, `slider`, `cueList`, `xyPad`, `speedDial`, `frame`, `clock`, `animation`, `label`) fica no mesmo nível dos campos do tipo.

**Palco (`Project.stage`):** largura, profundidade e altura em metros, a posição (x, y, z) e rotação de cada fixture e os elementos de palco (`elements`: tipo, posição e rotação), que só aparecem na prévia.

**Criações do Studio (`Function`):** `id`, `name`, `folder` (pasta ou `null`), `priority` (`low`, `normal` ou `high`) e `type` (`scene`, `chaser`, `sequence`, `collection`, `efx`, `rgbMatrix`) no mesmo nível dos campos do tipo. A sequência usa os campos do chaser, mais a cena (`scene`) e os valores em cada passo.

**Tempos e prioridade (F10c):**
- A cena tem `delayInMs`, `fadeInMs`, `delayOutMs`, `fadeOutMs`, a `curve` dos dois fades (`linear`, `easeIn` = lento no começo, `easeOut` = lento no fim, `easeInOut` = em S) e `attributeTimes`: tipos de atributo (`intensity` = dimmer; `color` = cores, rodas e macros de cor; `position` = pan e tilt; `effects` = shutter, gobo, prisma, zoom, foco e íris) com o delay e o fade de entrada e de saída próprios, no lugar dos da cena. O servidor manda a cena ao motor como uma coleção: uma parte com os tempos da cena e uma por tipo.
- Os tempos do chaser (`common` e o `timing` de cada passo) são `{ delayInMs, fadeInMs, holdMs, delayOutMs, fadeOutMs, curve }`. O delay de entrada e a curva seguem o `fadeInMode`; o de saída, o `fadeOutMode`. O passo dura o delay e o fade de entrada mais o hold. `holdMs` = 4294967295 (o `INFINITE` do contrato) é o hold ∞: o passo só sai com o Go, o anterior, outro passo ou o parar. A velocidade (Rate) não muda o ∞.
- A prioridade é a da criação disparada: o que um chaser ou uma coleção acende usa a deles; os sliders de nível ficam em Normal.

**Criações por grupo (F10b):**
- A cena tem `values` (fixture, posição do canal, valor) e `groups`: cada grupo de fixtures com os valores por tipo de canal (`{ attribute, index, value }`, o `index`-ésimo canal do atributo no modo, contando os finos: os dois prismas do Beam 7R são 0 e 1, e o Pan fino depois do Pan é 1). Ao rodar, o servidor aplica os valores do grupo em cada fixture que está nele naquele momento. Com uma fixture em dois grupos da cena, vence o de depois; o valor próprio dela (`values`) vence os dois (a exceção).
- O item da lista do EFX é `{ fixture, reverse, startOffset, spread, … }` ou `{ group, reverse, startOffset, spread, … }` (os campos da F10d estão abaixo). O grupo entra com as fixtures que estão nele, na ordem da grade, só as com pan ou tilt, e os ajustes do item ("Ao contrário", o antigo "Inverter", o "Início" e os da F10d) valem para todas. Cada fixture fica no primeiro lugar em que aparece na lista; a que também tem um item próprio usa os ajustes dele, mesmo dentro de um grupo (a exceção, como na cena). `spread` (só no grupo, de 0 a 359°; ausente, 0) é a defasagem: a k-ésima fixture do grupo começa em `startOffset + k × spread`. Remover o grupo põe esse início em cada fixture.
- A sequência continua por fixture: o passo guarda os valores da cena já com os grupos aplicados.

**EFX avançado (F10d):**
- O desenho do EFX (`EfxShape`) fica no mesmo nível dos outros campos, como antes: `algorithm`, `width`, `height`, `xOffset`, `yOffset`, `rotation`, as frequências e fases do Lissajous, mais `path` (o desenho à mão, `algorithm: "freehand"`: até 64 pontos `[x, y]` de -127 a 127, num caminho fechado, com o tilt para cima), `speed` (a curva da velocidade ao longo da volta, as mesmas dos fades; `linear` = constante) e `speedPoints` (a curva desenhada no lugar de `speed`: 16 valores de 0 a 255, sem descer, do 0 ao 255; ou `null`).
- O item da lista ganha `shape` (o desenho próprio, um `EfxShape` inteiro, ou `null` para o geral), `invertPan`, `invertTilt` e `swapAxes`. O espelho é do movimento em volta do centro do desenho; a troca dos eixos vem antes do espelho. Num grupo, valem para as fixtures dele; a exceção usa os do item dela.
- A velocidade vale pela posição no desenho (depois da propagação, do sentido e do início): todas as fixtures ficam lentas nos mesmos pontos do caminho, e a duração da volta não muda.
- O `saveFunction` confere o geral e o de cada item: os limites de antes, até 64 pontos de -127 a 127 no `path` e a curva desenhada com 16 pontos, de 0 a 255, sem descer.

## Estado de execução

Além do projeto, o `Snapshot` traz o que está acontecendo agora:

- `running`: criações disparadas pelo usuário, na ordem de início (na mistura LTP, vence a última). O que um chaser ou uma coleção dispara não entra. Um Single Shot que apagou no fim sai sozinho;
- `chaserSteps`: o passo atual (`{ chaser, step }`, a partir de 0) de cada chaser ou sequência da lista `running`, vindo do motor;
- `paused`: as criações da lista `running` pausadas;
- `loadedSteps`: o passo carregado (`{ chaser, step }`) de cada cue list, que o próximo Go usa;
- `desk`: níveis da Mesa simples (não são salvos no projeto);
- `master`: Grand Master, blackout e BPM;
- `widgetValues`: valores ao vivo dos elementos do console (não são salvos no projeto): slider e animation `[nível]`, XY pad `[x, y]`, speed dial `[fade in, fade out, duração]` em ms ou, com `beats` ("Seguir o BPM"), em milésimos de batida do BPM master (valem no lugar dos tempos gravados nas criações da lista dele: nas cenas, os fades; nos chasers, os tempos de todos os passos, menos a duração em segundos num chaser em batidas; numa coleção, nas criações dela; com dois elementos para a mesma criação, vale o último mexido; um `saveWidgets` que troca o `beats` solta o valor), botão de apagar fixtures `[1]` quando ligado, botão de Flash ou Temp `[1]` enquanto segura a criação que ele mesmo acendeu. Nos sliders (F10): o playback é a intensidade da criação; o submaster (sem valor, cheio) multiplica a intensidade das criações dos elementos da moldura dele (e das de dentro; solto, da página); o de nível acende os canais dos grupos dele, sem fade; a velocidade vai de ×0,25 (0) a ×4 (255), com ×1 em 128 (sem valor), nos tempos em segundos das criações da lista. Um `saveWidgets` que troca o modo ou a criação de um slider solta o valor (e o playback aceso desliga a criação);
- `stats`: CPU e memória do app, duração do último tick do motor e tempo ligado, para o Monitoramento;
- `interfaces`: as saídas DMX: a Dummy e as interfaces USB com chip FTDI (id `ftdi-` + número de série), com o tipo (`kind`, e `kindForced` quando escolhido à mão), se estão ligadas, quadros por segundo, maior atraso do último segundo e erros. Desplugada, a interface continua na lista;
- `testPattern`: o padrão de teste ligado, se houver;
- `remote.clients`: os aparelhos conectados de verdade (nome pelo sistema do navegador, endereço).

Os controladores MIDI (`midiDevices`) continuam simulados, vindos do show de exemplo, até a F12.

**Saída do motor:** camadas, de baixo para cima:
1. o valor padrão dos canais;
2. as criações rodando: cada cena acesa (sozinha, num passo de chaser ou numa coleção) com os tempos dela (o delay antes de cada fade, e o fade pela curva); dimmer e cores pelo maior valor (HTP), o resto pela última iniciada (LTP). Acima disso, a prioridade (F10c): num canal disputado, a criação de prioridade maior ganha, também no dimmer e nas cores; ao parar, ela desce até o que está embaixo. Na troca de passo, o que sai faz o fade out enquanto o que entra faz o fade in. O pan e o tilt de cada fixture de um EFX entram na mesma mistura (LTP). Esperando o delay de entrada, a cena fica fora da mistura, e o fade in parte do que está saindo quando o delay acaba;
3. a prévia;
4. os EFX relativos: o desenho somado ao que está embaixo (as criações e a prévia), com o zero em 127;
5. a Mesa simples;
6. o "Localizar";
7. o Grand Master (modo reduzir do QLC+) e o blackout, só nos canais de intensidade (dimmer e cores). Canais sem fixture contam como intensidade.

O padrão de teste substitui as camadas de 1 a 6. O RGB Matrix entra na saída na F9.

## Janelas do mesmo aparelho

As janelas externas (E13) são clientes como os outros: cada uma tem a sua conexão com o servidor. O `BroadcastChannel` do navegador só serve para a janela principal saber que a externa continua aberta.

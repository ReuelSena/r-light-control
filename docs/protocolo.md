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
- **Projeto:** `project` (projeto inteiro, no "Novo projeto"), `project.universes`, `project.fixtureDefs`, `project.fixtures`, `project.fixtureGroups`, `project.channelGroups`, `project.functions`, `project.folders`, `project.console`, `project.stage`.
- **Arquivo e biblioteca:** `projectPath`, `dirty` e `library`. A biblioteca fica gravada no PC, fora do projeto (um arquivo por definição).
- **Aparelhos e acesso:** `interfaces` e `remote`.
- **Ao vivo:** `running`, `desk`, `master`, `widgetValues`, `stats` e `testPattern`.

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
| `openProject` / `saveProject` | `path` / `path?` | Recusadas até a F5 (arquivos de verdade) |
| `addFixtures` | `defId, mode, name, color, count, universe, address?` | Adiciona `count` fixtures; sem `address`, cada uma vai para o primeiro lugar livre |
| `updateFixture` | `fixture` | Substitui a fixture (nome, cor, modo, universo, endereço) |
| `removeFixtures` | `ids` | Remove fixtures e as referências a elas (grupos, cenas, sequências, EFX, console, palco) |
| `autoAddress` | `ids, universe, start` | Endereça em sequência, sem buracos |
| `saveFixtureGroup` / `removeFixtureGroup` | `group` / `id` | Cria ou substitui um grupo de fixtures (grade 2D); remove |
| `saveChannelGroup` / `removeChannelGroup` | `group` / `id` | Cria ou substitui um grupo de canais; remove |
| `saveFixtureDef` | `def` | Cria ou substitui uma definição na biblioteca (e no projeto, se ele a usa). O id só tem minúsculas, números e hífens (vira o nome do arquivo) |
| `removeFixtureDef` | `id` | Tira a definição da biblioteca; o projeto continua com a cópia dele |
| `importQxf` | `fileName, content` | Converte o `.qxf` (crate `rlc-qxf`) e põe na biblioteca, como um `saveFixtureDef`: a mesma fixture (fabricante e modelo) substitui a anterior. Resposta: `ImportReport`, com os avisos da conversão. Um arquivo que não é fixture é recusado |
| `addUniverse` / `updateUniverse` / `removeUniverse` | — / `universe` / `id` | Universos e a interface de saída de cada um. Uma interface manda um universo só: escolhida num, sai do outro. Um universo com fixtures não é removido |
| `setRemoteAccess` | `enabled, pin?` | Acesso pela rede local e PIN |
| `saveFunction` / `removeFunction` | `function` / `id` | Cria ou substitui uma criação do Studio; remove (e tira dos passos de chaser, das coleções e do console) |
| `saveFolder` / `removeFolder` | `folder` / `id` | Pasta do Studio (de um tipo só); remover solta as criações dela. Mover uma criação de pasta é um `saveFunction` com outro `folder` |
| `setFunctionRunning` | `id, running` | Roda ou para uma criação (até a F6, não acende nada) |
| `stopAllFunctions` | — | Para todas as criações rodando ("Parar todas" do Studio e Stop All do console), baixa os sliders de playback e solta os passos escolhidos nas cue lists |
| `panic` | — | Pânico: o mesmo do `stopAllFunctions`, zera a Mesa simples inteira e desliga o padrão de teste |
| `setDeskLevel` | `universe, channel, value` | Fader de um canal da Mesa simples (canal de 1 a 512, valor de 0 a 255) |
| `setDeskGroupLevel` | `group, value` | Fader de um grupo de canais na Mesa simples: grava o nível do grupo e o valor em cada canal dele (os faders de canal acompanham) |
| `resetDesk` | `universe?` | Zera a Mesa simples: os canais do universo ou, sem ele, os grupos de canais (soltando os canais deles) |
| `saveConsolePage` / `removeConsolePage` | `page` / `id` | Página do console; remover leva os elementos dela |
| `saveWidgets` | `widgets` | Cria ou substitui vários elementos do console de uma vez (mover, alinhar, colar, copiar layout) |
| `removeWidgets` | `ids` | Remove elementos; os filhos de uma moldura removida vão junto |
| `setWidgetValue` | `widget, value` | Valor ao vivo de um elemento (ver `widgetValues` abaixo); `value` vazio solta o valor |
| `setGrandMaster` / `setBlackout` / `setBpm` | `value` / `on` / `bpm` | Seção master do console e o BPM global |
| `saveStage` | `stage` | Palco da 3D View: tamanho, posição das fixtures e elementos de palco |
| `setTestPattern` | `pattern?` | Padrão de teste da saída (`ramp` ou `chase`; sem ele, desliga). Página `/dev/dmx` |
| `setInterfaceKind` | `interface, kind?` | Tipo de uma interface USB (`enttecPro` ou `openDmx`) escolhido à mão e guardado no `settings.json`; sem `kind`, volta ao automático. A saída reabre no protocolo novo |

**Console (`Project.console`):** páginas e elementos (`VcWidget`). Cada elemento tem `id`, `page`, `frame` (moldura ou `null`), `caption`, `color`, `inputs` (controle MIDI e tecla por parte do elemento) e `layouts` com a posição em células em cada formato (`phone`, `tablet`, `desktop`; `null` = não aparece nele). O `type` (`button`, `slider`, `cueList`, `xyPad`, `speedDial`, `frame`, `clock`, `animation`, `label`) fica no mesmo nível dos campos do tipo.

**Palco (`Project.stage`):** largura, profundidade e altura em metros, a posição (x, y, z) e rotação de cada fixture e os elementos de palco (`elements`: tipo, posição e rotação), que só aparecem na prévia.

**Criações do Studio (`Function`):** `id`, `name`, `folder` (pasta ou `null`) e `type` (`scene`, `chaser`, `sequence`, `collection`, `efx`, `rgbMatrix`) no mesmo nível dos campos do tipo. A sequência usa os campos do chaser, mais a cena (`scene`) e os valores em cada passo.

## Estado de execução

Além do projeto, o `Snapshot` traz o que está acontecendo agora:

- `running`: criações rodando;
- `desk`: níveis da Mesa simples (não são salvos no projeto);
- `master`: Grand Master, blackout e BPM;
- `widgetValues`: valores ao vivo dos elementos do console (não são salvos no projeto): slider e animation `[nível]`, XY pad `[x, y]`, speed dial `[fade in, fade out, duração]` em ms, cue list `[passo]` escolhido à mão, botão de apagar fixtures `[1]` quando ligado;
- `stats`: CPU e memória do app, duração do último tick do motor e tempo ligado, para o Monitoramento;
- `interfaces`: as saídas DMX: a Dummy e as interfaces USB com chip FTDI (id `ftdi-` + número de série), com o tipo (`kind`, e `kindForced` quando escolhido à mão), se estão ligadas, quadros por segundo, maior atraso do último segundo e erros. Desplugada, a interface continua na lista;
- `testPattern`: o padrão de teste ligado, se houver;
- `remote.clients`: os aparelhos conectados de verdade (nome pelo sistema do navegador, endereço).

Os controladores MIDI (`midiDevices`) continuam simulados, vindos do show de exemplo, até a F12.

**Saída do motor na F3:** a Mesa simples (ou o padrão de teste), com o Grand Master (modo reduzir do QLC+) e o blackout por cima, só nos canais de intensidade (dimmer e cores). Canais sem fixture contam como intensidade. As criações do Studio e o console entram na saída a partir da F6.

## Janelas do mesmo aparelho

As janelas externas (E13) são clientes como os outros: cada uma tem a sua conexão com o servidor. O `BroadcastChannel` do navegador só serve para a janela principal saber que a externa continua aberta.

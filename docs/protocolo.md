# Protocolo UI ↔ motor (rascunho v0)

Rascunho montado a partir do que as telas da F2 precisam. É fechado na F3, quando o servidor Rust passa a falar com a UI de verdade.

- **Fonte da verdade:** os tipos ficam em [`crates/rlc-core/src/lib.rs`](../crates/rlc-core/src/lib.rs). A UI usa os mesmos tipos em TypeScript, gerados pelo ts-rs em `ui/app/types/generated/` ao rodar `cargo test -p rlc-core`. Nunca se editam os tipos gerados à mão.
- **Porta única das telas:** [`useEngine()`](../ui/app/composables/useEngine.ts) devolve `{ state, send }`. Na F2 é o `MockClient` ([`ui/app/engine/mock.ts`](../ui/app/engine/mock.ts)); na F3 passa a ser o cliente WebSocket, sem mudar as telas.

## Modelo (A2, A3)

1. **Ao conectar**, o cliente recebe um `Snapshot` com o estado completo: projeto, biblioteca, interfaces, MIDI e acesso remoto.
2. **Para alterar algo**, o cliente envia uma operação tipada (`Op`). O servidor valida, aplica e difunde o novo estado a **todos** os clientes (janela principal, janelas externas, celulares).
3. **O servidor é a autoridade.** A tela não altera o estado sozinha; só a UI do controle sob o dedo é otimista (fader, por exemplo).
4. **Dados de alta taxa** (valores DMX, preview) vão em frames binários, fora do JSON. Definição na F3.

## Envelope WebSocket (`/ws`)

```jsonc
// servidor → cliente, ao conectar
{ "t": "hello", "v": 1, "data": { "appVersion": "0.1.0", "lanIp": "192.168.0.6" } }
{ "t": "snapshot", "rev": 42, "data": { /* Snapshot */ } }

// cliente → servidor
{ "t": "cmd", "id": 7, "data": { "op": "autoAddress", "ids": ["f1"], "universe": "u1", "start": 1 } }

// servidor → cliente que enviou
{ "t": "ack", "id": 7, "data": null }          // ou a resposta da operação (ex.: ImportReport)
{ "t": "err", "id": 7, "data": "motivo legível" }

// servidor → todos
{ "t": "evt", "topic": "patch", "rev": 43, "data": { /* parte do estado que mudou */ } }
```

`rev` é a revisão do projeto. Um cliente que perde uma revisão pede um novo `snapshot`. O formato de `evt` (estado parcial ou operação aplicada) é decidido na F3.

## Operações v0 (`Op`)

Todas levam o campo `op` com o nome em camelCase e campos em camelCase.

| Operação | Campos | Efeito |
|---|---|---|
| `newProject` | — | Projeto vazio |
| `openProject` | `path` | Abre um arquivo (F5) |
| `saveProject` | `path` (opcional) | Salva no arquivo atual ou em outro |
| `addFixtures` | `defId, mode, name, color, count, universe, address?` | Adiciona `count` fixtures; sem `address`, usa o próximo endereço livre |
| `updateFixture` | `fixture` | Substitui a fixture (nome, cor, modo, universo, endereço) |
| `removeFixtures` | `ids` | Remove fixtures, e também as referências a elas nos grupos |
| `autoAddress` | `ids, universe, start` | Endereça em sequência, sem buracos |
| `saveFixtureGroup` / `removeFixtureGroup` | `group` / `id` | Cria ou substitui um grupo de fixtures (grade 2D); remove |
| `saveChannelGroup` / `removeChannelGroup` | `group` / `id` | Cria ou substitui um grupo de canais; remove |
| `saveFixtureDef` | `def` | Cria ou substitui uma definição na biblioteca |
| `importQxf` | `fileName, content` | Converte `.qxf` e põe na biblioteca. Resposta: `ImportReport` |
| `addUniverse` / `updateUniverse` / `removeUniverse` | — / `universe` / `id` | Universos e a interface de saída de cada um |
| `setRemoteAccess` | `enabled, pin?` | Acesso pela rede local e PIN |
| `saveFunction` / `removeFunction` | `function` / `id` | Cria ou substitui uma criação do Studio; remove (e tira dos passos de chaser e das coleções) |
| `saveFolder` / `removeFolder` | `folder` / `id` | Pasta do Studio (de um tipo só); remover solta as criações dela. Mover uma criação de pasta é um `saveFunction` com outro `folder` |
| `setFunctionRunning` | `id, running` | Roda ou para uma criação |
| `stopAllFunctions` | — | Para todas as criações rodando ("Parar todas" do Studio e Stop All do console), baixa os sliders de playback e solta os passos escolhidos nas cue lists |
| `panic` | — | Pânico: o mesmo do `stopAllFunctions` e zera a Mesa simples inteira (dois toques no botão da barra) |
| `setDeskLevel` | `universe, channel, value` | Fader de um canal da Mesa simples (canal de 1 a 512, valor de 0 a 255) |
| `setDeskGroupLevel` | `group, value` | Fader de um grupo de canais na Mesa simples: grava o nível do grupo e o valor em cada canal dele (os faders de canal acompanham) |
| `resetDesk` | `universe?` | Zera a Mesa simples: os canais do universo ou, sem ele, os grupos de canais (soltando os canais deles) |
| `saveConsolePage` / `removeConsolePage` | `page` / `id` | Página do console; remover leva os elementos dela |
| `saveWidgets` | `widgets` | Cria ou substitui vários elementos do console de uma vez (mover, alinhar, colar, copiar layout) |
| `removeWidgets` | `ids` | Remove elementos; os filhos de uma moldura removida vão junto |
| `setWidgetValue` | `widget, value` | Valor ao vivo de um elemento (ver `widgetValues` abaixo); `value` vazio solta o valor |
| `setGrandMaster` / `setBlackout` / `setBpm` | `value` / `on` / `bpm` | Seção master do console e o BPM global |
| `saveStage` | `stage` | Palco da 3D View: tamanho, posição das fixtures e elementos de palco |

**Console (`Project.console`):** páginas e elementos (`VcWidget`). Cada elemento tem `id`, `page`, `frame` (moldura ou `null`), `caption`, `color`, `inputs` (controle MIDI e tecla por parte do elemento) e `layouts` com a posição em células em cada formato (`phone`, `tablet`, `desktop`; `null` = não aparece nele). O `type` (`button`, `slider`, `cueList`, `xyPad`, `speedDial`, `frame`, `clock`, `animation`, `label`) fica no mesmo nível dos campos do tipo.

**Palco (`Project.stage`):** largura, profundidade e altura em metros, a posição (x, y, z) e rotação de cada fixture e os elementos de palco (`elements`: tipo, posição e rotação), que só aparecem na prévia.

**Criações do Studio (`Function`):** `id`, `name`, `folder` (pasta ou `null`) e `type` (`scene`, `chaser`, `sequence`, `collection`, `efx`, `rgbMatrix`) no mesmo nível dos campos do tipo. A sequência usa os campos do chaser, mais a cena (`scene`) e os valores em cada passo.

## Estado de execução

Além do projeto, o `Snapshot` traz o que está acontecendo agora:

- `running`: criações rodando;
- `desk`: níveis da Mesa simples (não são salvos no projeto);
- `master`: Grand Master, blackout e BPM;
- `widgetValues`: valores ao vivo dos elementos do console (não são salvos no projeto): slider e animation `[nível]`, XY pad `[x, y]`, speed dial `[fade in, fade out, duração]` em ms, cue list `[passo]` escolhido à mão, botão de apagar fixtures `[1]` quando ligado;
- `stats`: CPU, memória, duração do tick do motor e tempo ligado, para o Monitoramento.

Os **valores de saída** dos universos (512 canais cada) ficam fora do JSON: no `useEngine()` são o `output`, e do servidor (F3) virão em frames binários. Na F2, o `MockClient` os simula a 25 Hz a partir das criações rodando, do console e da mesa, com o Grand Master e os blackouts por cima.

## Janelas do mesmo aparelho

As janelas externas (E13) são clientes como os outros. No protótipo, o `MockClient` sincroniza as janelas pelo `BroadcastChannel` do navegador. Com o servidor, a sincronização vem do próprio WebSocket.

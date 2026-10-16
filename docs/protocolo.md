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

A lista cresce rodada a rodada (Studio, Mesa simples e Console nas rodadas 2 e 3).

## Janelas do mesmo aparelho

As janelas externas (E13) são clientes como os outros. No protótipo, o `MockClient` sincroniza as janelas pelo `BroadcastChannel` do navegador. Com o servidor, a sincronização vem do próprio WebSocket.

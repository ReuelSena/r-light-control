# Decisões do projeto

Regras e decisões que valem entre sessões. Toda decisão nova entra aqui com data. Uma decisão alterada é editada no lugar, com a data da mudança.

## Gerais (regras do usuário)

- **G1. Trabalho em fases com validação.** Executar → o usuário valida → ajustar → só então a próxima fase. Nenhuma fase começa sem a anterior aprovada.
- **G2. Interface e padrões primeiro.**
  - Os recursos são implementados sobre telas já aprovadas no protótipo (F2).
  - Mudança de tela passa primeiro pelo protótipo e pelo design system, para evitar retrabalho.
- **G3. O QLC+ é referência, não dependência.**
  - Algoritmos são reimplementados, não decompilados nem copiados.
  - Quando um algoritmo for portado de perto, ou houver reuso de assets (fixtures, meshes, rgbscripts), registrar a atribuição Apache-2.0 em `NOTICE`.
- **G4. Recursos do Virtual Console sob demanda.** O usuário vai pedir recursos extras durante o desenvolvimento. Cada pedido vira um item no `PLANO.md` e uma decisão aqui.
- **G5. Commits só quando o usuário pedir.**
- **G6. Idioma.**
  - Conversa, documentação e textos da UI em pt-BR.
  - Identificadores de código em inglês.
  - Comentários em pt-BR.
- **G7. Código enxuto e legível.**
  - A menor quantidade de código possível, sem minificar demais. Um desenvolvedor deve ler e entender com facilidade.
  - Não criar helpers, abstrações ou camadas para um único uso ou pouquíssimos usos, principalmente quando o código direto ocupa menos linhas que a definição do helper.
  - Abstrair só com repetição real ou exigência do projeto (ex.: a fronteira `EngineClient` mock/real).

## Específicas (técnicas)

- **E1. Stack.** Tauri 2 + Rust (motor, drivers USB, servidor HTTP/WebSocket embutido) + Nuxt 4 em modo SPA (`ssr: false`).
  - Motivos: timing do Open DMX, app leve e caminho para Android na v2.
- **E2. UI ↔ motor só por WebSocket e HTTP.**
  - A janela Tauri abre a mesma UI que o celular acessa pela rede local.
  - O IPC do Tauri fica restrito a recursos nativos (ex.: diálogo de arquivo).
- **E3. Formato próprio** (JSON versionado) para projeto e fixture.
  - Importação apenas de fixtures `.qxf` do QLC+, convertidas para o formato próprio.
  - Não há importação de `.qxw`.
- **E4. Interfaces da v1:** Enttec DMX USB Pro e compatíveis, e Open DMX (FTDI sem microcontrolador). Mais uma saída "Dummy" para desenvolver sem hardware.
- **E5. Recursos da v1:** Cenas, Chasers, Sequências, Coleções, EFX, RGB Matrix, Virtual Console e entrada MIDI.
  - Fora da v1: Show/timeline com áudio, Script, Art-Net/sACN, OSC.
- **E6. Design inspirado na Soundcraft Ui24R:** escuro, cara de mesa de som, faders grandes, touch-first, operável no navegador.
- **E7. Android como host USB fica para a v2** (Tauri Android + plugin Kotlin com usb-serial-for-android). Na v1 o celular acessa pelo navegador na rede local.
- **E8. Referência do QLC+** clonada em `reference/qlcplus`, fixada na tag `QLC+_5.3.0` e fora do git do projeto.
  - Essa árvore contém a UI v4 (`ui/`), a UI v5 (`qmlui/`) e o `engine/` compartilhado.
  - Só leitura: nunca compilar nem executar código de lá.

## Arquitetura

- **A1. Threads.**
  - Uma thread de motor a 50 Hz com deadline. Ela é dona do estado de runtime e não usa locks no caminho quente.
  - Os comandos chegam por canal e são drenados a cada tick.
  - Cada universo publica o frame mais recente. Cada interface DMX tem sua própria thread, que lê esse frame no próprio ritmo: Enttec Pro a 44 Hz; Open DMX a 30 Hz com break de 110 µs e MAB de 16 µs.
  - Se o motor falhar, as saídas repetem o último frame.
- **A2. Estado.**
  - O projeto tem um único escritor. Cada operação tipada é validada, incrementa a revisão e é difundida a todos os clientes.
  - O servidor é a autoridade. A UI só é otimista no controle que está sob o dedo.
- **A3. Protocolo WebSocket.**
  - Endpoint único `/ws`, com envelope JSON para comandos e eventos.
  - Frames binários para dados de alta taxa (universos, preview).
  - Snapshot completo ao (re)conectar.
  - Detalhes em `docs/protocolo.md` (a partir da F2).
- **A4. IDs.**
  - UUIDv7 nas entidades salvas.
  - Definições de fixture embutidas no projeto (slug + hash), para que um projeto nunca quebre por falta de definição.
- **A5. Arquivos.**
  - JSON com `{format, schemaVersion}`. Ao carregar, uma cadeia de migrações converte versões antigas.
  - Escrita atômica (arquivo temporário + rename) e `.bak`.

## Registro

| Data | Decisão |
|---|---|
| 2026-10-15 | Plano inicial aprovado (G1–G7, E1–E8, A1–A5). |
| 2026-10-15 | F0 validada. `docs/padroes-codigo.md` e o padrão de idioma (G6) aprovados. Porta padrão do servidor: 8090 (configurável em `settings.json`). |

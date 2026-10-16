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
  - Visual skeuomórfico: botões, faders, displays e painéis imitam componentes físicos (referência: `docs/design/ui24r-mix.png`).
  - **Só tema escuro.** Não há tema claro.
  - **Cores semânticas vivas, de contraste alto:** matiz pura com saturação e brilho máximos. Exemplos: vermelho `#FF0000`, verde `#00FF00`, amarelo `#FFFF00`, laranja `#FF8000`, ciano `#00FFFF`.
  - O filete de LED apagado fica bem fraco, para o aceso se destacar.
  - **Sem textos de descrição ou ajuda na tela.** Quando uma explicação for necessária, ela fica num ícone "i" ao lado do título da seção, página ou diálogo, que abre um balão (`RlInfo`). Avisos (ex.: conflito de endereço), resultados e telas vazias continuam como texto.
- **E9. Fader.**
  - Arrasto **relativo**: o fader não salta para onde o dedo tocou.
  - Um botão **FINO** logo abaixo de cada fader reduz o deslocamento, para ajustes de precisão.
  - No desktop, a roda do mouse ajusta fader e knob: 1% por clique, 0,1% com FINO.
- **E10. Exibição de valores.** Em % (0–100) por padrão, com opção de DMX (0–255) nas configurações.
- **E11. Virtual Console montado pelo usuário.** Como no QLC+, o usuário posiciona e configura cada elemento.
  - Aparência e comportamento dos botões são propriedades de cada elemento: destaque (aceso ou LED), cor, modo (liga/desliga ou flash) e ação (função, blackout geral, blackout de fixtures ou grupos, stop all).
  - Os botões fixos do sistema (FINO, navegação, abas) seguem os padrões do design system.
  - **Layouts por formato de tela:** o usuário cria e ajusta o console para cada formato (ex.: celular, tablet, desktop), e cada aparelho abre o layout do seu formato.
- **E12. Organização da interface.** Menus:
  - **Console:** Virtual Console.
  - **Fixtures:** criação, importação, configuração e agrupamento dos equipamentos.
    - Dois tipos de grupo: **de fixtures** (grade 2D de cabeças, para o RGB Matrix) e **de canais** (como os "Channel Groups" do QLC+: canais de quaisquer fixtures, do mesmo atributo ou misturados, usados em faders do console e da mesa simples e nas cenas).
    - Na lista, o grupo de canais mostra o ícone do atributo quando todos os canais são do mesmo tipo, e um ícone genérico quando são misturados.
  - **Studio:** todas as criações (cenas, chasers, sequências, coleções, EFX, RGB Matrix…). Cada tipo tem um ícone próprio, usado em todo o projeto para identificá-lo.
    - A lista agrupa as criações por tipo (sem filtro de tipo; só a busca). Dentro de cada tipo há pastas. Arrastar uma criação a move para uma pasta do mesmo tipo, ou para fora (soltando no título do tipo): com o mouse, clicar e arrastar; no toque, segurar e arrastar.
    - Ao criar uma coleção com criações rodando, o app pergunta se ela nasce vazia ou com as que estão rodando.
  - **Patch.**
  - **3D View:** visualização, configuração e posicionamento dos equipamentos para o preview.
  - **Mesa simples:** banco de faders, como no QLC+, para mexer em canais DMX específicos ou grupos de canais fora do console.
    - Em tela larga (a partir de 1024 px), como no QLC+: faders de canal em cima e a faixa dos grupos de canais embaixo. No celular, os grupos ficam numa aba.
    - O fader de um grupo grava nos canais do grupo, e os faders desses canais acompanham. O "Zerar" dos grupos solta os canais deles.
    - Os 512 canais do universo ficam numa faixa só, sem páginas. Ctrl + roda do mouse rola a faixa (sem Ctrl, a roda mexe no fader).
  - **Monitoramento:** os universos mandando sinais, mais dados do app e da máquina (uso de CPU etc.).
  - **In/Out:** uma tela só, sem abas, com duas seções:
    - **Saídas:** universos e interfaces DMX;
    - **Entradas:** só identifica os controladores MIDI conectados por USB.
  - **Entradas externas por elemento:** o controle MIDI (com "aprender") ou a tecla do teclado que aciona um elemento do console é definido na edição do próprio elemento, como no QLC+. Não há tela separada de mapeamento MIDI.
  - Configurações no ⚙.
  - **Onde se programa:** as ferramentas de atributo (intensidade, cor, posição, efeitos) ficam no editor de cena do Studio, como no QLC+ 5. Não há aba "Programador".
    - No editor de cena, o toque escolhe **uma fixture** para receber os valores. **Ctrl + clique** (ou **toque longo**, no celular) soma outras **do mesmo tipo** (mesmo modelo e modo), e o que se mexe vale para todas as escolhidas. Os grupos de canais da cena mexem em várias de uma vez: o fader do grupo grava nos canais das fixtures do grupo que estão na cena (não inclui outras).
    - A aba **Canais** vem primeiro. Intensidade, Cor, Posição e Efeitos (shutter/strobe, gobo, prisma, zoom, foco) só aparecem quando a fixture tem esse tipo de canal.
    - "Adicionar fixtures" é o primeiro botão da fileira de fixtures e abre uma lista com nome, universo e canais de cada uma, com os atalhos "Desmarcar todos", "Selecionar do mesmo tipo" (o toque vale para todas do mesmo modelo) e "Selecionar grupo" (inclui as fixtures de um grupo de fixtures).
    - Os grupos de canais começam desligados. Ligar um grupo traz para a cena todas as fixtures dele; para tirar uma, usa-se "Adicionar fixtures".
    - Fade in e fade out ficam no fim do editor de cena: no show, o console ajusta os fades ao vivo (VC-3).
  - **Celular:** Console · Studio · Mesa simples · Fixtures · Mais (com os demais menus e o ⚙).
  - A **seção master** (GM, Blackout, Stop All) fica só no Console. Exceções (2026-10-16):
    - **botão de pânico na barra superior**, em todas as telas: um toque para todas as criações; dois toques seguidos também zeram a Mesa simples inteira;
    - **"Parar todas" no Studio**, que para só as criações.
- **E13. Interface modular (opcional).**
  - Painéis encaixáveis e redimensionáveis, como no VS Code ou no encaixe de janelas do Windows.
  - Qualquer painel pode ir para uma **janela externa do Windows**, para usar outro monitor (ex.: 3D View num monitor dedicado). A janela externa é a mesma tela conectada ao motor como mais um cliente, então continua funcionando mesmo com a janela principal minimizada.
  - O app **começa no modo simples** (uma tela por vez). O modular é ativado opcionalmente. No celular, é sempre o simples.
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
| 2026-10-15 | Decisões da F1: visual skeuomórfico da Ui24R e só tema escuro (E6), fader relativo com botão FINO (E9), valores em % com opção de DMX (E10). |
| 2026-10-16 | Ajustes pedidos na validação da F1: roda do mouse em fader e knob (E9); botão com segundo modo de destaque, `highlight="led"` (só o filete acende e o botão fica afundado). |
| 2026-10-16 | Cores semânticas vivas no sistema todo (E6): matiz pura com saturação e brilho máximos. O filete de LED aceso usa a cor pura com brilho; o apagado fica a 30%. |
| 2026-10-16 | F1 validada: 60 fps no celular e 100 no desktop. Decisões: aparência e comportamento dos botões do VC configuráveis por elemento (E11, pedido VC-1); os testes de toque automatizados ficam fora do projeto por enquanto. |
| 2026-10-16 | F1 commitada. Decisões da F2: menus Console, Fixtures, Studio, Patch, 3D View e Mesa simples (E12); seção master só no Console (E12); interface modular opcional com janelas externas (E13); console com layouts por formato de tela (E11, pedido VC-2); protótipo entregue uma rodada por vez. |
| 2026-10-16 | Mini-plano da F2 aprovado: menus Monitoramento e In/Out (E12); ferramentas de atributo no editor de cena do Studio (E12); começa no modo simples (E13); navegação do celular como proposta. |
| 2026-10-16 | Ajustes pedidos na validação da rodada 1 da F2: grupos de canais com ícone por atributo (E12); abas agrupadas num trilho, como controle segmentado, no lugar de botões soltos (design system). |
| 2026-10-16 | In/Out sem abas, com as seções Saídas e Entradas; o mapeamento de MIDI e de teclado fica em cada elemento do console (E12). |
| 2026-10-16 | Rodada 1 da F2 aprovada e commitada (com arrastar fixtures na grade do Patch). |
| 2026-10-16 | Editor de cena: uma fixture por vez, várias pelos grupos de canais; aba Canais primeiro e as de atributo só quando fazem sentido; lista de fixtures com universo e canais (E12). |
| 2026-10-16 | Sem descrições na tela: explicações só no ícone "i" ao lado do título (E6, `RlInfo`). O fader de grupo da cena só mexe nas fixtures da cena. |
| 2026-10-16 | Editor de cena reorganizado: "Adicionar fixtures" com atalhos (desmarcar todos, mesmo tipo, grupo), divisor antes dos grupos de canais, e ligar um grupo de canais traz as fixtures dele para a cena (E12). |
| 2026-10-16 | Editor de cena: multisseleção de fixtures do mesmo tipo com Ctrl + clique ou toque longo; o que se mexe vale para todas as escolhidas (E12). `RlButton` ganha `@longpress`. |
| 2026-10-16 | Aba "Beam" do editor de cena renomeada para "Efeitos" (vale para PARs e moving heads) (E12). |
| 2026-10-16 | Studio: lista agrupada por tipo, sem filtro de tipo, com pastas por tipo e arrastar entre pastas (E12). |
| 2026-10-16 | Studio: coluna da lista mais larga (360 px), botão "Parar todas" (exceção ao Stop All só no Console) e coleção nova a partir das criações rodando (E12). |
| 2026-10-16 | Fades no fim do editor de cena. Pedido VC-3: fades ajustados ao vivo no console (G4, PLANO). |
| 2026-10-16 | Botão de pânico na barra superior: 1 toque para as criações; 2 toques também zeram a Mesa simples (exceção ao Stop All só no Console, E12). Abaixo de 1536 px a barra esconde o nome R·LIGHT e o texto USB. |
| 2026-10-16 | Mesa simples: em tela larga, canais em cima e grupos de canais embaixo; no celular, grupos numa aba (E12). |
| 2026-10-16 | Mesa simples: o fader de grupo grava nos canais do grupo e os faders de canal acompanham (E12). |
| 2026-10-16 | Mesa simples sem páginas: os 512 canais numa faixa, e Ctrl + roda do mouse rola a faixa (E12, design system). |
| 2026-10-16 | Rodada 2 da F2 aprovada e commitada. |

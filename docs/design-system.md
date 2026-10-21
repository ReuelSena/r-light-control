# Design system

Referência visual: Soundcraft Ui24R ([`docs/design/ui24r-mix.png`](design/ui24r-mix.png)). Decisões: E6 (skeuomórfico, só escuro), E9 (fader relativo com FINO) e E10 (% ou DMX).

Para ver e testar tudo: `pnpm dev` → `/dev/showcase` (catálogo), `/dev/faders` (64 faders, multitoque, fps) e `/dev/beams` (teste de feixes 3D).

## Princípios

1. **Parece físico.** Botões em relevo, displays LCD afundados, caps metálicos, painéis em camadas. A cor aparece só onde informa: estado, atributo ou seleção.
   - Quando aparece, é **viva**: matiz pura com saturação e brilho máximos, para contraste alto no palco.
2. **Toque primeiro.** Alvos de pelo menos 44 px, nada depende de hover, vários dedos ao mesmo tempo, nenhum controle fica preso.
3. **Seguro ao vivo.** Faders não saltam (arrasto relativo), o flash solta sempre e não há zoom, seleção de texto nem menu de contexto acidentais.

## Tokens

Definidos em `@theme` no [`ui/app/assets/css/main.css`](../ui/app/assets/css/main.css). Cada token vira utilitário do Tailwind (`bg-surface-1`, `text-fg-muted`, `font-lcd`) e também variável CSS (`var(--color-select)`).

| Token | Uso |
|---|---|
| `surface-0` … `surface-3` | Fundo → painel → controle → pressionado |
| `line` | Bordas e divisórias |
| `fg`, `fg-muted`, `fg-dim` | Texto principal, secundário, desabilitado/régua |
| `on` `#00FF00` | Ligado, confirmar |
| `flash` `#FFFF00` | Flash, FINO, atenção |
| `danger` `#FF0000` | Blackout, perigo, cap do master |
| `select` `#FF8000` | Seleção, aba e navegação ativas |
| `panel` `#0058E0` | Painel lateral (bancos e funções) |
| `lcd` `#00FFFF` | Texto de displays |
| `intensity` `#FFBF00` · `position` `#00BFFF` · `beam` `#8000FF` · `speed` `#FF00AA` | Cor por atributo de fixture (medidor, pad XY, knobs) |
| `font-sans` (Inter) / `font-lcd` (Share Tech Mono) | Interface / displays. As duas vêm embutidas e funcionam offline |

## Utilitários de relevo

| Classe | Aparência | Onde |
|---|---|---|
| `raised` | Botão em relevo | Botões, abas |
| `sunken` | Superfície afundada | Displays, trilhos, listas |
| `brushed` | Metal escovado | Barra superior, cabeçalho de diálogo |
| `field` | Campo afundado de 44 px | `<input>` e `<select>` nativos |

No celular, o `<select>` nativo abre o seletor do próprio sistema, que é melhor que qualquer imitação.

Para montar telas:

| Classe | Uso |
|---|---|
| `panel` | Bloco de conteúdo dentro de uma tela |
| `heading` | Título de seção (maiúsculas, discreto) |
| `divider` | Divisória entre seções de um mesmo painel (ex.: grupos de canais e tempos no editor de cena) |
| `field-label` | `<label>` com o texto em cima e o campo embaixo |
| `rl-table` | Tabelas (canais, faixas, mapeamentos) |
| `rl-led` | LED de status; a cor vem de `--led` (padrão verde) |
| `rl-key` | Etiqueta de tecla (atalho de teclado), em texto de display; legível sobre botão aceso |

## Componentes (`ui/app/components/rl/`)

| Componente | Quando usar | Comportamento |
|---|---|---|
| `RlButton` | Qualquer ação | `mode`: `action`, `toggle` ou `flash`. `highlight`: `face` (o botão inteiro acende, padrão) ou `led` (só o filete acende e o botão fica afundado). `color`: cor do destaque e do filete. `size`: `sm`, `md` ou `lg`. Emite `press` com o evento (para ler `ctrlKey`). Com `@longpress`, segurar 500 ms dispara `longpress` no lugar de `press`. |
| `RlLcd` | Mostrar valores e estados | Texto ciano por padrão; `color` muda a cor (ex.: blackout em vermelho). |
| `RlFader` | Nível contínuo (intensidade, submaster, GM) | Arrasto relativo, botão FINO (×0,1), toque duplo volta ao `default`, roda do mouse, setas e PageUp/PageDown. `meter` opcional; `cap="red"` para o master. `marks`: a escala ao lado do trilho (rótulo, número ou texto, e posição de 0 a 1; sem ela, 0 a 100%). `fineStep`: o passo da roda e das setas com o FINO (padrão 0,001, ou 0,1% do curso). |
| `RlKnob` | Parâmetros secundários (zoom, foco, velocidade) | Arrasto vertical relativo (150 px = curso todo), roda do mouse, toque duplo volta ao padrão. |
| `RlXYPad` | Pan/tilt | Arrasto relativo em X e Y, FINO, toque duplo centraliza. |
| `RlColorPicker` | Cor de fixtures RGB | Roda de matiz/saturação (toque absoluto, brilho máximo), amostras de gelatina, campos R, G e B. |
| `RlMeter` | Nível de saída | Segmentos de LED; `level` de 0 a 1. |
| `RlChannelStrip` | Canal ou fixture na mesa | Display (tocar emite `edit`), FLASH, SEL, fader com FINO, etiqueta colorida (laranja quando selecionada). Com `plain`, sem os botões (Mesa simples, faders de cena). O slot `label` troca o texto da etiqueta (ex.: ícone + nome). |
| `RlNumpad` | Digitar um valor exato | Respeita % ou DMX; emite o valor normalizado (0–1). |
| `RlDialog` | Confirmações, edição pontual | Com `sheet`, vira painel: sobe de baixo no celular e entra pela direita em telas maiores (ex.: as Configurações, abertas pelo ⚙ sobre qualquer tela). `info` põe o "i" ao lado do título. |
| `RlTabs` | Alternar visões dentro de um painel | Abas agrupadas num trilho afundado (controle segmentado); a ativa é a tecla acesa em laranja. Ocupam a largura toda em espaços estreitos (o texto quebra em duas linhas se precisar) e, a partir de `@3xl`, só o necessário, com abas de largura igual. Conteúdo em slots com o nome do `value`, ou no slot padrão quando todas as abas mostram a mesma tela com dados diferentes (ex.: universos no Patch). O slot `actions` põe botões ao lado do trilho (ex.: **+** novo universo). Use sempre que botões trocam a visão do painel, em vez de botões soltos. |
| `RlSegmented` | Escolher uma entre poucas opções (ordem, sentido, propagação) | Mesmo trilho das abas, sem conteúdo embaixo. Sempre há uma opção escolhida. Com `size="sm"`, fica da altura de um botão pequeno (36 px), para barras ao lado de botões `sm` (ex.: vistas e "Editar posição" da 3D View). |
| `RlStepper` | Valor numa escala fixa (zoom) | − e + andam pelos degraus (`steps`); toque duplo no display volta ao padrão. |
| `RlTimeField` | Tempos (fade, hold, duração) | Mostra segundos (ou batidas, com `beats`) e guarda milissegundos. |
| `RlList` | Listas selecionáveis (bibliotecas, funções) | Linhas de 48 px; `v-model` é o id selecionado. O slot `item` personaliza a linha. Com `reorderable`, arrastar uma linha a leva para outra posição (mouse: clicar e arrastar; toque: segurar e arrastar) e o evento `reorder` traz o id e a posição final. |
| `RlSwatches` | Escolher uma cor de uma paleta curta (etiqueta de fixture) | `v-model` com a cor; `colors` com as opções. |
| `RlSwitch` | Opções liga/desliga em configurações | Alavanca com texto ao lado. |
| `RlInfo` | Explicar uma seção, página ou diálogo | Ícone "i" ao lado do título; tocar abre um balão com o texto. É o único lugar para explicações (E6). |
| `RlAppShell` | Moldura de toda tela | Slots: `toolbar`, `nav`, `toolbar-end`, padrão (principal), `master`, `side`, `bottom`. |

Diálogos e abas usam os primitivos da [Reka UI](https://reka-ui.com) (foco e teclado prontos). Fader, knob, pad XY e seletor de cor são nossos, porque nenhuma biblioteca tem comportamento de mesa.

## Ícones dos tipos de criação

`functionIcons` ([`utils/functions.ts`](../ui/app/utils/functions.ts)) é o ícone de cada tipo do Studio (E12), usado em todo lugar onde uma criação aparece: lista, passos de chaser e coleções. No console, é o ícone inicial quando o usuário liga o ícone de um botão.

| Tipo | Ícone (lucide) | Ideia |
|---|---|---|
| Cena | `Camera` | Uma "foto" dos valores |
| Chaser | `Repeat` | Passos em ciclo |
| Sequência | `Film` | Quadros de uma mesma cena |
| Coleção | `Layers` | Várias juntas |
| EFX | `Orbit` | Movimento desenhado |
| RGB Matrix | `LayoutGrid` | Grade de pixels |

## Ícones de atributo

`attributeLook` ([`utils/fixtures.ts`](../ui/app/utils/fixtures.ts)) dá o ícone e a cor de cada atributo de canal. Use-o sempre que um canal ou atributo aparecer: editor de grupo, mesa simples, editor de cena.

- Atributos da mesma família dividem o ícone e mudam só a cor:
  - **paleta:** todas as cores, com a cor pura de cada componente;
  - **setas:** pan e tilt.
- Os de intensidade, posição, beam e velocidade usam o token do atributo (`intensity`, `position`, `beam`, `speed`).
- **Grupo de canais** (`channelGroupLook`): mostra o ícone dos canais quando todos dividem o mesmo; se as cores diferem (ex.: R, G e B), o ícone fica neutro. Canais misturados mostram o ícone genérico de faders.

## Console (Virtual Console)

`widgetKinds` ([`utils/console.ts`](../ui/app/utils/console.ts)) dá o ícone (da paleta de edição), o tamanho inicial e as entradas externas de cada tipo de elemento.

**Botões e sliders em operação (F10):** o aparelho só manda "apertou" e "soltou" (`pressWidget`); o servidor aplica o que o elemento faz. O botão em Flash ou Temp usa o `RlButton` no modo `flash` (aceso enquanto o dedo está nele); o liga/desliga, o "Só esta" e o "Parar tudo" usam o modo `action`, e o aceso vem do servidor (a criação rodando, o blackout, as fixtures apagadas). O slider de velocidade mostra "×1,0" e a escala ×4, ×2, ×1, ½, ¼ (`marks`), com ×1 no meio; o submaster e o Animation sem valor ficam cheios; o toque duplo volta a esses valores.

**Ícone dos botões:** por padrão o botão mostra só o texto. Com "Mostrar ícone", ele começa com o ícone do tipo da criação (ou o da ação: apagar → `LightbulbOff`, parar tudo → `OctagonX`), e "Trocar" abre a biblioteca inteira do lucide (`ConsoleIcons`), com busca pelo nome em inglês e uma seleção sugerida. A biblioteca (~1.700 ícones) fica num pedaço separado do app ([`useIconLibrary`](../ui/app/composables/useIconLibrary.ts)), baixado só quando algum botão usa ícone.

| Elemento | Ícone (lucide) | Tamanho inicial (células) |
|---|---|---|
| Botão | `RectangleHorizontal` | 4 × 4 |
| Slider | `SlidersVertical` | 4 × 12 |
| Cue list | `ListOrdered` | 16 × 12 |
| XY pad | `Move` | 8 × 12 |
| Tempos (VC-3) | `Timer` | 12 × 8 |
| Moldura | `SquareDashed` | 16 × 12 |
| Relógio | `Clock` | 8 × 4 |
| Animation | `Sparkles` | 12 × 12 |
| Texto | `Type` | 8 × 2 |

- **Grade (VC-2):** 64 colunas no celular, 128 no tablet e 192 no desktop, para ajuste fino. A célula é quadrada e acompanha a largura; o elemento ocupa o retângulo dele com 3 px de folga e tem no mínimo 8 × 8 células (o Texto pode ter 4 de altura). Na moldura, as quatro primeiras linhas são do cabeçalho.
- **Cores (`widgetColors`):** cada cor viva com a versão escura dela (25% do brilho, para fundo de texto na cor viva) e sete cinzas do branco ao preto. O campo de cor (`ConsoleColor`) mostra a cor escolhida e abre a paleta; com `none`, oferece "Sem cor" (fundo e borda do Texto).
- **Edição:** a grade aparece, e o painel de propriedades fica sempre no lugar em telas largas (vazio, com um aviso, quando nada está escolhido), para escolher um elemento não mexer no layout. A célula guarda o tamanho da operação (o painel de propriedades faz a grade rolar para o lado, sem apertar os elementos). Elementos com controle MIDI mostram um ícone de teclado musical no canto.
- **Etiqueta de tecla:** o elemento acionado por tecla mostra a tecla (ou a combinação) numa etiqueta no canto superior direito, na operação e na edição. O botão de dicas de atalho (teclado, ao lado do "Editar", ou Alt + I) mostra ou esconde as etiquetas do console, inclusive as da seção master. Setas e espaço viram símbolos (← → ↑ ↓ ␣).
- **Zoom do console:** `RlStepper` ao lado do "Editar", com os degraus do navegador (50% a 200%). Multiplica a célula da grade, guardado por aparelho.
- **Seção master:** coluna à direita, com o GM num fader de cap vermelho e o BPM num fader ao lado (de 30 a 300, escala em BPM, FINO de 1 BPM), cada um com o seu FINO e o seu display; abaixo, blackout, parar tudo e o TAP. Em espaço estreito, faixa em cima com o GM e o BPM em knobs (o BPM, ao lado do número e do TAP).

## Regras de interação

- **Arrasto relativo** ([`useDrag`](../ui/app/composables/useDrag.ts)):
  - o controle captura o dedo que o tocou, então cada dedo controla o seu e o controle não se perde quando o dedo sai de cima;
  - o valor anda pelo deslocamento, nunca salta para o ponto tocado;
  - toque sem arrasto não muda nada;
  - dois toques em menos de 300 ms voltam ao padrão.
- **FINO:** liga e desliga o modo de precisão. Pode ser acionado com outro dedo enquanto o primeiro segura o fader.
- **Roda do mouse** (desktop), com o ponteiro sobre fader ou knob:
  - cada clique da roda anda 1%; com FINO, 0,1%;
  - é proporcional ao deslocamento, então o trackpad não dispara;
  - sobre esses controles a roda não rola a página.
  - com **Ctrl**, a roda rola a faixa de strips na horizontal (Mesa simples) em vez de mexer no fader.
- **Destaque `face` ou `led`:** `face` para estados que precisam chamar atenção de longe (blackout, flash, aba ativa). `led` para estados discretos, que parecem uma chave travada (seleção, opções ligadas).
- **Botões disparam ao soltar o dedo (`pointerup`)**, não no `click`. O navegador não gera `click` para um toque feito enquanto outro dedo está na tela. Pelo teclado, Enter/Espaço continuam funcionando.
- **Flash** acende ao tocar e apaga ao soltar, ao cancelar o toque ou ao perder a captura. Ele usa `touch-action: none`, para um dedo que escorrega não virar rolagem.
- **Multisseleção:** o toque escolhe um item; Ctrl + clique (desktop) ou toque longo (celular) soma ou tira outros. Ex.: fixtures do mesmo tipo no editor de cena.
- **Arrastar para posicionar** (Patch, edição do console e 3D View com "Editar posição"):
  - com o mouse, qualquer item é arrastado direto (na 3D View, com o botão esquerdo; a câmera fica no botão do meio e no direito, e o zoom na roda);
  - no toque, o primeiro toque seleciona e o arrasto vale só para o item selecionado (ele recebe `touch-action: none`), para o gesto sobre o resto continuar rolando;
  - o ponto agarrado fica sob o dedo, e o destino aparece antes de soltar: verde se livre, vermelho se houver conflito.
  - em listas que rolam (ex.: a do Studio), no toque o arrasto só começa depois de segurar 400 ms; antes disso, o gesto rola a lista. Com o mouse, começa ao mover 6 px.
- **Sob o dedo, otimista (A2):** fader, knob e pad XY mostram o próprio movimento na hora e o próximo movimento parte dele (`useOptimistic`); 300 ms depois do último movimento, vale de novo o valor do servidor. Os outros controles mostram só o estado do servidor.
- **Conexão:** sem o servidor, as telas ficam cobertas por um aviso, sem toque, até ele voltar (com o estado completo). Na abertura, a tela "Conectando…" fica até o primeiro estado chegar.
- **Operação recusada:** o motivo aparece num aviso embaixo, por 4 s. Quem esperava a resposta (`await send(...)`) para ali: um diálogo continua aberto para corrigir.
- **Desfazer e refazer:** "Desfeito" ou "Refeito" aparece no mesmo lugar, com borda neutra, por 1,5 s, porque a alteração pode estar em outra tela. "Nada para desfazer" vem como recusa.
- **Estados que mexem na luz por fora das criações:** um LCD amarelo na barra de cima, em todas as telas: "TESTE" (padrão de teste da saída) e "PRÉVIA" (cena em prévia). O segundo toque do pânico desliga os dois (o primeiro corta as criações na hora).
- **Liga e desliga que agem na luz** (Prévia, Localizar): `RlButton` com `highlight="led"` em amarelo (`--color-flash`). Ficam ligados só enquanto a tela está aberta: ao sair, o aparelho desliga o que ele ligou.
- **Confirmação por segundo toque:** ações que descartam algo (remover, "Novo" e abrir outro projeto com alterações não salvas) trocam o texto do botão ou mostram uma linha amarela, e só agem no segundo toque.
- **Clique fantasma do toque:** depois de um toque, o navegador manda um clique de compatibilidade no mesmo ponto, que cairia no que o botão acabou de abrir ali (ex.: o painel de propriedades no celular). O `RlButton` cancela esse clique quando foi acionado por toque. Por isso, nada deve depender do `click` de um `RlButton` (ex.: `DialogClose as-child`); use `@press`. Mesmo assim, o que aparece por causa de um toque vai numa linha abaixo do botão, não ao lado, para o layout não andar sob o dedo.
- **Rolagem:** as barras (navegação, bancos, strips) rolam pelo gesto. Faders, knobs, pads e flashes não deixam o gesto virar rolagem.
- **Barras de rolagem:** um estilo só para o app todo (`main.css`), com trilho afundado e cursor em relevo, como um fader pequeno, e sem setas. Vale com mouse; no toque ficam as barras do aparelho, finas e que somem sozinhas. No Firefox, só as cores.
- **Atalhos de teclado:**
  - padrão do app: Ctrl + − e Ctrl + + (zoom; o do console quando ele está na frente) e, no teclado numérico, − blackout, + parar tudo e Enter tap do BPM (com etiquetas `rl-key` na seção master); no console, Alt + E (editar) e Alt + I (dicas de atalho), que valem também com o foco num campo;
  - desfazer (Ctrl + Z) e refazer (Ctrl + Shift + Z) em todas as telas (`app.vue`). Num campo de texto, vale o desfazer do próprio campo; com o console na frente em operação, as duas combinações são dos elementos (não são reservadas), e na edição do console desfazem;
  - os elementos do console usam as outras teclas; a captura recusa as reservadas (`isReservedKey`);
  - o teclado numérico tem nome próprio ("Num 1", "Num -"), separado da fileira de cima das letras (`keyCombo`).
- **Proibidos na página:** zoom por pinça ou toque duplo, rolagem elástica, seleção de texto (exceto em campos) e menu de contexto do toque longo.

## Layout

| Largura | Comportamento |
|---|---|
| Celular (< 640 px) | A navegação (`nav`) desce para uma barra inferior; o painel lateral vira sheet (botão ☰). |
| Tablet (640–1023 px) | Navegação na barra superior; painel lateral ainda em sheet. |
| Desktop / tablet grande (≥ 1024 px) | Painel lateral azul fixo à direita. |

A seção master fica só no Console (E12). As strips rolam na horizontal.

**Dentro das telas, use container queries** (`@3xl:grid-cols-2`, e não `md:`). A mesma tela pode ocupar a janela inteira, um painel estreito do modo modular ou uma janela externa. Por isso ela se adapta à largura do espaço que ocupa, e não à da janela. Equivalências usadas: `sm` → `@xl`, `md` → `@3xl`, `lg` → `@5xl`, `xl` → `@7xl`. O shell e os diálogos continuam com breakpoints de janela.

**Zoom:** o zoom geral da interface (⚙) é o zoom nativo da janela do app desktop e, no celular e no tablet, a largura do viewport (`app.vue`), que o navegador encaixa na tela na escala escolhida (no celular, o 100% é 75% da escala do navegador): as medidas em px continuam valendo, e a tela se reorganiza como no zoom do navegador (um celular com zoom pequeno pode cair no layout de tablet). Não use `zoom` do CSS na página: ele desalinha os balões (Reka/floating-ui) e os arrastos, que misturam coordenadas da tela e do elemento.

**Classes do Tailwind sobre componentes `Rl*`:** o estilo do próprio componente vence as utilidades que mexem nas mesmas propriedades (`hidden`, `justify-start`, `normal-case`…). Nesses casos use o `!` no fim (`@3xl:hidden!`).

## Valores e textos

- **Valores:** use [`useValueFormat`](../ui/app/composables/useValueFormat.ts). O `format(v)` mostra % ou DMX conforme a preferência (E10). Internamente, todo valor de controle vai de 0 a 1.
- **Sem descrições na tela (E6):** nada de frases explicando como usar algo. Quando precisar, use `RlInfo` ao lado do título da seção ou página (ou `info` no `RlDialog`). Avisos, resultados e telas vazias continuam como texto.
- **Textos:** todo texto visível passa pelo i18n ([`ui/i18n/locales/pt-BR.json`](../ui/i18n/locales/pt-BR.json)), com chaves `rl.*` para os componentes.
  - Símbolos universais (R, G, B, X, Y, %, DMX, BPM) não são traduzidos.
  - Nomes digitados pelo usuário (fixtures, cenas) são dados, não textos de interface.

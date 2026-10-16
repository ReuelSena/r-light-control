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

## Componentes (`ui/app/components/rl/`)

| Componente | Quando usar | Comportamento |
|---|---|---|
| `RlButton` | Qualquer ação | `mode`: `action`, `toggle` ou `flash`. `highlight`: `face` (o botão inteiro acende, padrão) ou `led` (só o filete acende e o botão fica afundado). `color`: cor do destaque e do filete. `size`: `sm`, `md` ou `lg`. Emite `press`. |
| `RlLcd` | Mostrar valores e estados | Texto ciano por padrão; `color` muda a cor (ex.: blackout em vermelho). |
| `RlFader` | Nível contínuo (intensidade, submaster, GM) | Arrasto relativo, botão FINO (×0,1), toque duplo volta ao `default`, roda do mouse, setas e PageUp/PageDown. `meter` opcional; `cap="red"` para o master. |
| `RlKnob` | Parâmetros secundários (zoom, foco, velocidade) | Arrasto vertical relativo (150 px = curso todo), roda do mouse, toque duplo volta ao padrão. |
| `RlXYPad` | Pan/tilt | Arrasto relativo em X e Y, FINO, toque duplo centraliza. |
| `RlColorPicker` | Cor de fixtures RGB | Roda de matiz/saturação (toque absoluto, brilho máximo), amostras de gelatina, campos R, G e B. |
| `RlMeter` | Nível de saída | Segmentos de LED; `level` de 0 a 1. |
| `RlChannelStrip` | Canal ou fixture na mesa | Display (tocar emite `edit`), FLASH, SEL, fader com FINO, etiqueta colorida (laranja quando selecionada). |
| `RlNumpad` | Digitar um valor exato | Respeita % ou DMX; emite o valor normalizado (0–1). |
| `RlDialog` | Confirmações, edição pontual | Com `sheet`, vira painel: sobe de baixo no celular e entra pela direita em telas maiores. |
| `RlTabs` | Alternar visões dentro de um painel | Aba ativa acende em laranja; conteúdo em slots com o nome do `value`. |
| `RlList` | Listas selecionáveis (bibliotecas, funções) | Linhas de 48 px; `v-model` é o id selecionado. |
| `RlSwitch` | Opções liga/desliga em configurações | Alavanca com texto ao lado. |
| `RlAppShell` | Moldura de toda tela | Slots: `toolbar`, `nav`, `toolbar-end`, padrão (principal), `master`, `side`, `bottom`. |

Diálogos e abas usam os primitivos da [Reka UI](https://reka-ui.com) (foco e teclado prontos). Fader, knob, pad XY e seletor de cor são nossos, porque nenhuma biblioteca tem comportamento de mesa.

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
- **Destaque `face` ou `led`:** `face` para estados que precisam chamar atenção de longe (blackout, flash, aba ativa). `led` para estados discretos, que parecem uma chave travada (seleção, opções ligadas).
- **Botões disparam ao soltar o dedo (`pointerup`)**, não no `click`. O navegador não gera `click` para um toque feito enquanto outro dedo está na tela. Pelo teclado, Enter/Espaço continuam funcionando.
- **Flash** acende ao tocar e apaga ao soltar, ao cancelar o toque ou ao perder a captura. Ele usa `touch-action: none`, para um dedo que escorrega não virar rolagem.
- **Rolagem:** as barras (navegação, bancos, strips) rolam pelo gesto. Faders, knobs, pads e flashes não deixam o gesto virar rolagem.
- **Proibidos na página:** zoom por pinça ou toque duplo, rolagem elástica, seleção de texto (exceto em campos) e menu de contexto do toque longo.

## Layout

| Largura | Comportamento |
|---|---|
| Celular (< 640 px) | A navegação (`nav`) desce para uma barra inferior; o painel lateral vira sheet (botão ☰). |
| Tablet (640–1023 px) | Navegação na barra superior; painel lateral ainda em sheet. |
| Desktop / tablet grande (≥ 1024 px) | Painel lateral azul fixo à direita. |

A seção master fica sempre visível à direita. As strips rolam na horizontal.

## Valores e textos

- **Valores:** use [`useValueFormat`](../ui/app/composables/useValueFormat.ts). O `format(v)` mostra % ou DMX conforme a preferência (E10). Internamente, todo valor de controle vai de 0 a 1.
- **Textos:** todo texto visível passa pelo i18n ([`ui/i18n/locales/pt-BR.json`](../ui/i18n/locales/pt-BR.json)), com chaves `rl.*` para os componentes.
  - Símbolos universais (R, G, B, X, Y, %, DMX, BPM) não são traduzidos.
  - Nomes digitados pelo usuário (fixtures, cenas) são dados, não textos de interface.

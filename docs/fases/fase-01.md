# F1 — Design system (inspirado na Ui24R)

**Status:** concluída e validada pelo usuário em 2026-10-16. Referência visual: [`docs/design/ui24r-mix.png`](../design/ui24r-mix.png). Regras de uso: [`docs/design-system.md`](../design-system.md).

**Mudanças em relação ao plano**
- `RlSheet` virou a opção `sheet` do `RlDialog`, porque o comportamento é o mesmo (G7).
- Campos de texto e select usam elementos nativos com a classe `field`, sem componente próprio. No celular, o select nativo abre o seletor do sistema.
- Os botões disparam no `pointerup`. O teste automático mostrou que o navegador não gera `click` quando outro dedo está na tela; era o caso de segurar o fader e tocar no FINO.

**Verificação automática (toque emulado)**
- Arrasto relativo.
- Toque sem arrasto não muda o valor.
- FINO acionado por outro dedo durante o arrasto (Δ 1 ponto, contra 13 sem FINO).
- Toque duplo volta ao padrão.
- Dois faders ao mesmo tempo.
- Flash solta no cancelamento do toque.
- Toggle por mouse (uma vez só) e por teclado.

Tudo passou. `pnpm check` e o build de produção também passaram.

**Ajustes pedidos na validação (2026-10-16)**
- Roda do mouse em fader e knob: 1% por clique, 0,1% com FINO. Testado: fader, FINO, knob, e a página não rola.
- `RlButton` com `highlight="led"`: ativo, só o filete acende e o botão fica afundado. O modo atual continua como `face` (padrão). Os dois aparecem lado a lado em "Botões e campos".
- Cores semânticas vivas no sistema todo: matiz pura com saturação e brilho máximos (vermelho `#FF0000` etc.). O filete aceso fica puro e com brilho; o apagado, a 30%.

## Objetivo

Definir a linguagem visual e os componentes base, e validá-los no desktop, no tablet e no celular antes de qualquer tela (G2). Ao fim da fase, as telas da F2 só combinam componentes já aprovados.

## O que trazemos da Ui24R

- **Cores:** fundo grafite quase preto, painéis em camadas de cinza. Cor aparece só onde há informação: estado ou atributo.
- **Channel strips** verticais lado a lado, com rolagem horizontal e etiqueta colorida no topo.
- **Seção master** fixa à direita. Aqui ela terá Grand Master, Blackout, Stop All e BPM.
- **Navegação:** abas no topo para trocar de visão e uma barra de bancos para pular entre grupos de strips.
- **Faders grandes**, com régua e valor numérico sempre visível.
- **Botões que acendem** no estado ativo. Na Ui24R: mute vermelho, solo amarelo. Aqui: blackout vermelho, flash amarelo, ativo verde.
- **Feita para toque:** alvos grandes, nada depende de hover.
- **Skeuomorfismo**, observado no print:
  - barra superior em metal escovado;
  - displays LCD com texto ciano sobre fundo escuro afundado;
  - caps de fader metálicos com ranhuras (o do master é vermelho);
  - botões em relevo que acendem por inteiro quando ativos e mostram um filete de LED colorido quando apagados;
  - seleção em laranja (aba e canal selecionados);
  - painel lateral azul para bancos e funções.

Na adaptação para luz, uma strip pode ser um canal, uma fixture ou um playback. As cores seguem o atributo (intensidade, cor, posição…).

```
┌───────────────────────────────────────────────────────────────┐
│ ● Projeto  [Console][Programador][Funções][Patch]…  BPM  ⏻  ⚙ │ barra superior
├───────────────────────────────────────────────────┬───────────┤
│ ┌────┐┌────┐┌────┐┌────┐┌────┐┌────┐              │  MASTER   │
│ │PAR1││PAR2││MH 1││MH 2││ …  ││    │              │   ┌───┐   │
│ │ ▮▮ ││ ▮▮ ││ ▮▮ ││ ▮▮ │  strips (rolagem      │   │GM │   │
│ │ ┃  ││ ┃  ││ ┃  ││ ┃  │  horizontal)          │   │ ┃ │   │
│ │ 75%││100%││ 40%││  0%│                       │   └───┘   │
│ │FLSH││FLSH││FLSH││FLSH│                       │ BLACKOUT  │
│ └────┘└────┘└────┘└────┘                        │ STOP ALL  │
├───────────────────────────────────────────────────┴───────────┤
│ [1–8] [9–16] [17–24] …                       bancos / páginas │
└───────────────────────────────────────────────────────────────┘
```

No celular, a seção master vira um botão na barra superior e as abas descem para uma barra de navegação inferior.

## Tokens

Ficam em variáveis CSS no `@theme` do Tailwind v4, então valem tanto nas classes quanto no CSS comum. Os valores abaixo são uma proposta e serão ajustados com os prints.

| Grupo | Proposta |
|---|---|
| Superfícies | fundo `#111214` · painel `#1a1c1f` · controle `#24272b` · pressionado `#2f3338` · borda `#3a3f45` |
| Texto | principal `#e6e8eb` · secundário `#9ba1a6` · desabilitado `#5f666d` |
| Estados (vivos, ajuste de 2026-10-16) | ativo verde `#00ff00` · flash/atenção amarelo `#ffff00` · blackout/perigo vermelho `#ff0000` · seleção laranja `#ff8000` · painel lateral azul `#0058e0` · LCD ciano `#00ffff` |
| Atributos (vivos) | intensidade âmbar `#ffbf00` · cor = a própria cor do canal · posição azul-celeste `#00bfff` · beam/gobo violeta `#8000ff` · velocidade/efeito magenta `#ff00aa` |
| Tipografia | Inter embutida no app (funciona sem internet), números tabulares, tamanhos 12/14/16/20/28 |
| Medidas | espaçamento base de 4 px · raio de 6/10 px · alvo de toque mínimo de 44 px (48 px no celular) |

## Componentes (`ui/app/components/rl/`)

| Componente | Comportamento |
|---|---|
| `RlFader` | Vertical, com **arrasto relativo**: o fader não salta para onde o dedo tocou, como na Ui24R. Isso evita saltos durante o show.<br>Multitoque: cada dedo controla o seu fader.<br>Botão **FINO** logo abaixo do fader: liga e desliga o modo de precisão (arrasto ×0,1). Toque duplo volta ao valor padrão.<br>Valor sempre visível, régua, medidor opcional e cor por atributo. Também responde a teclado (setas, PageUp/PageDown). |
| `RlButton` | Tipos: ação, toggle e flash (momentâneo). Estados apagado/aceso, com cor.<br>O flash solta ao levantar o dedo, ao cancelar o toque e ao perder a captura, então nunca fica preso. |
| `RlKnob` | Giratório, controlado com arrasto vertical. Mostra o valor num arco. Toque duplo reseta. |
| `RlXYPad` | Pan/tilt com mira e modo fino. Valores de 0 a 1 em X e Y. |
| `RlColorPicker` | Anel de matiz com saturação e brilho, amostras rápidas de cores de gelatina e entrada numérica RGB. |
| `RlMeter` | Barra vertical de nível. |
| `RlChannelStrip` | Composição no estilo Ui24R: etiqueta colorida, botão (selecionar/flash), fader e valor. |
| Estrutura | `RlAppShell`: barra superior, abas ou navegação inferior, seção master e barra de bancos.<br>Também: `RlTabs`, `RlSheet` (bottom sheet no celular, painel lateral no desktop), `RlDialog`, `RlNumpad` (digitar um valor exato), `RlList` e campos (texto, número, select, switch). |

**Bibliotecas**
- Tailwind v4: tokens e layout.
- Reka UI: primitivos acessíveis para dialog, tabs, select e popover, para não reescrever foco e teclado.
- lucide-vue-next: ícones embutidos, que funcionam offline.
- `@fontsource-variable/inter`: a fonte Inter, embutida.

Fader, knob e XY pad são feitos por nós, porque nenhuma biblioteca tem o comportamento de mesa que precisamos.

## Comportamento global de toque

- Nos controles: `touch-action: none` e `user-select: none`, e nenhum menu de contexto no toque longo.
- Na página: sem zoom por pinça ou toque duplo, e sem rolagem elástica (`overscroll-behavior: none`).
- Layout por largura: celular abaixo de 640 px, tablet de 640 a 1279 px, desktop a partir de 1280 px. Funciona em retrato e paisagem.

## Páginas de desenvolvimento

- **`/dev/showcase`:** todos os componentes em todos os estados, nos três tamanhos, mais um shell de exemplo com strips fictícias.
- **`/dev/faders`:** 64 faders animando a 50 Hz, teste de multitoque e medidor de fps.
- **`/dev/beams`:** 50 moving heads simplificadas, com feixes (TresJS, cones aditivos instanciados) em movimento, e medidor de fps. Serve para validar cedo o 3D no seu celular; o 3D de verdade é a F15.
- A página do contador da F0 sai. Até a F2, `/` abre o showcase.

## Documentação

`docs/design-system.md` com os tokens, quando usar cada componente, os estados, o grid e as regras de toque. Os prints de referência ficam em `docs/design/`.

## Fora do escopo

Telas reais (F2), dados e motor.

## Decisões do usuário

1. **Estilo:** skeuomórfico como na Ui24R; botões, faders e a interface toda parecem componentes físicos (E6).
2. **Fader:** arrasto relativo, com um botão FINO abaixo dele para ajustes de precisão (E9).
3. **Valores:** % por padrão, com opção de DMX (E10).
4. **Tema:** só escuro (E6).

## Validação

- [x] Visual aprovado no desktop, no tablet e no celular.
- [x] 4 ou mais faders movidos ao mesmo tempo com vários dedos.
- [x] 60 fps no celular em `/dev/faders` (100 no desktop). Em `/dev/beams`, pelo menos 30 fps.
- [x] Sem zoom, rolagem elástica ou seleção de texto acidentais.
- [x] O flash nunca fica preso (soltar o dedo fora do botão, trocar de app no meio do toque).
- [x] `docs/design-system.md` revisado.

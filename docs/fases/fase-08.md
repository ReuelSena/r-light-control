# F8 — Grupos de fixtures e EFX

**Status:** mini-plano aprovado em 2026-10-19, com as quatro decisões como recomendado (na 4, os três ajustes). Entregue em 2026-10-19, aguardando a sua validação. Commitada em 2026-10-19, a seu pedido, antes da validação. O que falta está nas pendências do [PLANO.md](../../PLANO.md#pendências-2026-10-19).

A F7 foi concluída a seu pedido sem a validação. Os itens dela, e os da F6 que ficaram, entram no teste junto com esta fase (ver "Validação").

**Entregue**
- **Motor (`rlc-engine`, `efx.rs`):**
  - os 10 desenhos com as contas do QLC+, a volta de cada fixture, as três propagações, "Inverter" e "Para trás", "Uma vez" e "Vai e volta";
  - um fader por fixture na mistura da F6, com o fade in a partir da posição de antes e o fade out até o que está embaixo;
  - o EFX relativo como camada própria, por cima das criações e da prévia;
  - a volta pela duração de agora: mudar a duração ou o BPM não dá salto.
- **Servidor:**
  - traduz o EFX rodando, com o pan e o tilt de cada fixture nos endereços DMX (com o fino) e os tempos do EFX, do passo do chaser ou do elemento "Tempos";
  - confere o EFX e o grupo de fixtures ao gravar;
  - remover um grupo também o tira do RGB Matrix;
  - o arquivo do projeto passa à versão 4, com a migração.
- **Telas:**
  - os três ajustes da decisão 4 no editor de EFX;
  - a prévia do editor com as regras do motor;
  - na grade do grupo, pintar uma cabeça que já está no grupo a muda de lugar.
- **Diferenças do plano** (detalhes que o plano não dizia):
  - "Selecionar grupo" só aparece quando algum grupo tem fixtures com pan ou tilt, e a lista mostra só esses grupos.
  - Um EFX "Uma vez" sem fixtures termina na hora, como no QLC+. Nas outras ordens, fica rodando sem acender.
  - **Defeito corrigido na grade do grupo:** com poucas colunas num painel largo, as células ficavam enormes (641 px) e se sobrepunham. Agora cada célula tem no máximo 80 px, e a grade não é mais espremida pelo painel. O visual das células não mudou.
- **Desempenho:** 8 cenas em fade, 4 chasers trocando de passo e um EFX em 48 moving heads, com os 2.048 canais de 4 universos, dão 0,68 ms por tick no app compilado.
- **Testes:**
  - automáticos:
    - os pontos dos testes `preview…` do QLC+ nos 10 desenhos, o sentido contrário (e a linha meia volta à frente), o Lissajous com a frequência 0, e os números do `rotateAndScale` e do `widthHeightOffset`;
    - no motor: as três propagações, "Inverter" com "Para trás", "Vai e volta", "Uma vez", os 16 bits e a ponta do curso, os fades, o relativo sobre uma cena e sobre a prévia (com a Mesa simples por cima), a duração e o BPM sem salto, a edição ao vivo, e o EFX num passo de chaser e numa coleção;
    - no servidor: a tradução (endereços com o fino, fixture sem pan e tilt fora, o elemento "Tempos" em batidas), as conferências do EFX e do grupo, a limpeza do RGB Matrix e a migração da versão 3;
  - ponta a ponta: o pan e o tilt do MH 1 mudam a cada quadro, e o EFX "Uma vez" sai das criações rodando;
  - no navegador, contra o seu `pnpm dev`, com um grupo e um EFX de teste só nos MHs do universo 1 (Dummy):
    - o servidor recusou a cabeça repetida e a largura 200;
    - a ordem sem "Aleatória", e só o grupo de teste em "Selecionar grupo";
    - o grupo entrou na ordem da grade (MH 3, MH 1, MH 4, MH 2);
    - rodando, o pan mudou a cada quadro, e as quatro cabeças ficaram em pontos diferentes;
    - pintar o MH 1 na célula do MH 3 o mudou de lugar;
    - o editor no celular.
  - Depois do teste, as edições foram desfeitas, e o `settings.json`, o projeto salvo e a biblioteca ficaram iguais aos de antes.
- **No seu projeto:**
  - O "Círculo nos MHs" estava em "Aleatória" e abriu em "Repetir" (a migração). O arquivo salvo só muda quando você salvar.
  - Ele inclui os 4 Beams do Rig RSena, junto com os MHs genéricos do universo 1.

## Objetivo

Fazer o EFX acender: as fixtures com pan e tilt andam pelo desenho escolhido no Studio, sozinhas ou junto com as outras criações. Depois da F8:
- "Rodar" um EFX move os moving heads no desenho (círculo, oito, linha…), com o tamanho, o centro e a rotação do editor;
- as fixtures andam juntas, defasadas ou uma depois da outra;
- o EFX relativo desenha em volta da posição que as fixtures já têm (de uma cena ou da prévia);
- o EFX acende também num passo de chaser e numa coleção, e o elemento "Tempos" muda a velocidade dele, inclusive nas batidas do BPM;
- os grupos de fixtures servem para incluir as fixtures no EFX na ordem da grade.

## O que já existe (F2 a F7)

- **Studio:** o editor de EFX com a prévia animada do desenho, os 10 algoritmos (as contas do QLC+, em `utils/functions.ts`), as fixtures com "Inverter" e "Início (°)", a propagação, a ordem, o sentido e os tempos.
- **Fixtures:** o editor dos grupos de fixtures (a grade de cabeças), gravado no servidor desde a F3.
- **Motor (F6 e F7):** cada cena acesa tem um fader, com a mistura HTP/LTP e o fade a partir do valor que está saindo; os chasers e as coleções disparam as criações como uma árvore (`Program`). Num passo de chaser ou numa coleção, o EFX já ocupa o lugar dele, mas não acende.

## O que entra

### 1. EFX no motor

- **Cada fixture anda pelo desenho:** a posição na volta vira um ponto do desenho. O ponto é girado, escalado pela largura e pela altura e levado ao centro. Depois vira pan e tilt (0 a 255), em 16 bits quando a fixture tem o canal fino. O que passa do curso fica na ponta (0 ou 255).
- **Os 10 desenhos** com as contas do QLC+, as mesmas da prévia da tela. No Lissajous, a frequência 0 vira um vai e vem em linha reta, como no QLC+.
- **Duração da volta:** o tempo "Duração da volta" do editor. Mudar a duração com o EFX rodando (no editor ou no elemento "Tempos") não dá salto: a cabeça continua do ponto onde está, como no QLC+.
- **Sentido:**
  - "Para trás" percorre o desenho ao contrário.
  - "Inverter" vale só para aquela fixture. Com os dois ligados, ela volta a andar para a frente, como no QLC+.
  - Na Linha, o sentido contrário anda meia volta à frente, porque ida e volta passam pelos mesmos pontos.
- **Início (°):** a fixture começa adiantada nesse ângulo.
- **Propagação (decisão 3):**
  - **Juntas:** todas no mesmo ponto.
  - **Defasadas:** todas começam juntas, cada uma 1/n da volta à frente da anterior (com 4 fixtures, 90°).
  - **Em sequência:** cada uma começa o desenho do início, 1/n da volta depois da anterior. Até chegar a vez dela, fica na posição que tinha.
- **Ordem (decisão 4):**
  - **Repetir.**
  - **Uma vez:** uma volta, e o EFX termina e sai das criações rodando, com o fade out.
  - **Vai e volta:** a cada volta, o sentido vira.
- **Fades (decisão 1):** no fade in, cada cabeça sai da posição em que está e entra no desenho em movimento. Ao parar, ela sai do ponto onde está e volta, no fade out, ao que está embaixo (outra criação ou o valor padrão), como as cenas (decisão 2 da F6).
- **Mistura:** o pan e o tilt do EFX entram na mistura da F6. Vence o que foi iniciado por último:
  - um EFX disparado depois de uma cena com posição move as cabeças;
  - uma cena com posição disparada depois do EFX segura as cabeças paradas;
  - parar essa cena devolve as cabeças ao EFX.
- **Relativo (decisão 2):**
  - O desenho é somado à posição que está embaixo. O centro em 127 não desloca; outro valor desloca o desenho em relação a essa posição.
  - Vale sobre todas as criações rodando, em qualquer ordem de início.
  - No fade in, o desenho cresce a partir dessa posição; no fade out, encolhe de volta para ela.
- **Só pan e tilt:** como no QLC+, o EFX não mexe na luz. O dimmer, a cor e o gobo vêm de uma cena. Para ver o movimento, rode junto uma cena com o dimmer aceso, ou ponha as duas numa coleção.
- **Edição com o EFX rodando:** vale na hora, sem recomeçar (desenho, tamanho, centro, rotação e fixtures). A fixture incluída entra com o fade in; a retirada sai com o fade out.

### 2. EFX nos chasers, nas coleções e no elemento "Tempos"

- **Num passo de chaser,** o EFX acende. O fade in e o fade out do passo valem para ele; no modo "Padrão", valem os do EFX. O fade in conta na duração do passo, como nas cenas.
- **Numa coleção,** ele roda junto com as outras criações, com os fades dele. A coleção só com criações que terminam sozinhas (um EFX "Uma vez", um chaser Single Shot) também termina.
- **Elemento "Tempos":** o fade in, o fade out e a duração valem para o EFX da lista. A duração é a da volta:
  - em segundos;
  - com "Seguir o BPM", em batidas (ex.: um círculo a cada 4 batidas). Um tap muda a velocidade na hora, sem salto.

### 3. Grupos de fixtures

- O servidor passa a conferir o grupo ao gravar: as células dentro da grade, uma cabeça por célula, cada cabeça uma vez só (como no QLC+) e só fixtures e cabeças que existem.
- Pintar uma cabeça que já está no grupo muda ela de lugar.
- Remover um grupo tira ele também do RGB Matrix que o usava. Nos botões de blackout, isso já acontece.
- **No EFX (decisão 4):** a lista de "Adicionar fixture" mostra os grupos. Tocar num grupo inclui as fixtures dele que têm pan ou tilt, na ordem da grade (linha por linha), que é a ordem da propagação.
- O RGB Matrix usa a grade na F9, e o blackout por grupos do console entra na F10.

### 4. Servidor, contrato e protocolo

- **Tradução para o motor:** o servidor manda cada EFX rodando com o pan e o tilt de cada fixture já nos endereços DMX (com o fino) e os tempos que valem: os do EFX, os do passo do chaser ou os do elemento "Tempos".
- **Conferência do EFX ao gravar:**
  - largura e altura até 127, frequências até 32 e ângulos até 359;
  - sem a ordem "Aleatória";
  - só fixtures com pan ou tilt, sem repetir.
- **Contrato:** sem campos novos.
- **Arquivo do projeto, versão 4 (decisão 4):** o EFX em "Aleatória" abre em "Repetir", e a largura ou a altura acima de 127 abre em 127.
- **Protocolo:** nenhuma operação nova. O `setFunctionRunning` e as camadas da saída em `docs/protocolo.md` passam a incluir o EFX. O EFX relativo vira uma camada própria, logo acima da prévia (decisão 2).
- **NOTICE (G3):** os desenhos do EFX passam a valer também em `rlc-engine`. O sentido, a propagação e a ordem vêm de `efxfixture.cpp`.

### 5. Telas (G2)

Mudanças da decisão 4, no editor de EFX:
- a ordem passa a ter só Repetir, Uma vez e Vai e volta;
- os knobs de largura e altura vão até 127;
- os grupos entram na lista de "Adicionar fixture".

A prévia do editor passa a seguir as regras do motor: o sentido da Linha e a propagação "Em sequência". As outras telas não mudam.

## Decisões para você

1. **Como o EFX entra e sai.**
   - **Do jeito das cenas** (recomendado):
     - no fade in, a cabeça sai da posição em que está e entra no desenho em movimento;
     - no fade out, ela volta ao que está embaixo, seguindo a decisão 2 da F6.
   - **Como no QLC+:**
     - no fade in, a cabeça pula para o centro, e o desenho cresce do centro até o tamanho todo;
     - o EFX não tem fade out: ao parar, a cabeça fica onde estava.
2. **Em volta de que o EFX relativo desenha.**
   - **Das criações rodando e da prévia** (recomendado): com a Prévia ligada no editor de cena, o desenho anda em volta da posição que você está programando. É o item "EFX relativo sobre uma posição do programador" do plano. A camada dele fica entre a prévia e a Mesa simples, então a prévia deixa de mostrar a cena exatamente como está nas fixtures desse EFX.
   - **Só das criações rodando:** a prévia continua por cima, com a cena exatamente como está (as cabeças paradas). Para mirar o desenho, você roda a cena e a edita rodando (F6).
3. **O espaçamento da propagação.**
   - **1/n da volta** (recomendado): com 4 fixtures, 90° entre elas, como a prévia do QLC+ mostra.
   - **Como o motor do QLC+:** 1/(n+1) da volta (com 4 fixtures, 72°). É diferente da própria prévia dele. Nele, a fixture "Em sequência" já começa adiantada quando chega a vez dela.
4. **Ajustes no editor de EFX** (mudam a tela, G2). Recomendo os três:
   - **a)** Ordem sem "Aleatória". O EFX do QLC+ não tem essa ordem, que não faz sentido num desenho.
   - **b)** Largura e altura de 0 a 127, como no QLC+: 127 é o curso inteiro do pan ou do tilt, com o centro em 127. Hoje o knob vai a 255, e o desenho passaria do curso.
   - **c)** Grupos na lista de "Adicionar fixture", na ordem da grade. É o uso dos grupos no EFX, como no QLC+.

## Fica de fora

- **Modos Dimmer e RGB do EFX** (o desenho no dimmer ou nas cores) e o "controle do dimmer" do QLC+ 5.3. Aqui o EFX só mexe no pan e no tilt; os outros podem entrar depois, se você quiser.
- **Fixtures com várias cabeças de pan e tilt:** o EFX usa a primeira cabeça de cada uma.
- **Volta em batidas no próprio editor:** por enquanto, só pelo elemento "Tempos" com "Seguir o BPM". Pode entrar depois.
- **"Distribuir os inícios"** do QLC+ 5 (0°, 90°, 180°… automático ou sorteado): por enquanto, o Início (°) é escrito à mão em cada fixture, ou se usa "Defasadas".
- **O "Início" geral do EFX** (no QLC+, além do de cada fixture).
- **O movimento no 2D e no 3D:** entra na F14 e na F15.
- **Intensidade por criação e submaster:** entram na F10.

## Testes automáticos

- **Desenhos:** os pontos dos testes `preview…` do QLC+, nos 10 desenhos, para a frente e para trás. Também a rotação, a escala e o centro (`rotateAndScale` e `widthHeightOffset`).
- **Motor:**
  - propagação: 4 fixtures defasadas a 90°; "Juntas" no mesmo ponto; "Em sequência" esperando a vez;
  - sentido: "Inverter" e "Para trás", também os dois juntos;
  - ordem: "Vai e volta" e "Uma vez", que termina;
  - saída: os 16 bits com o fino, e a ponta quando o desenho passa do curso;
  - fades: o fade in a partir da posição de antes; o fade out até o que está embaixo;
  - relativo: sobre uma cena, sobre a prévia e com o centro deslocado;
  - mudar a duração sem salto, e a volta em batidas acompanhando o tap;
  - o EFX num passo de chaser e numa coleção, e a edição ao vivo.
- **Servidor:** a tradução (endereços com o fino, fixture sem tilt), os tempos do elemento "Tempos", a conferência do EFX e do grupo (cabeça repetida, célula fora da grade) e a migração da versão 3.
- **Desempenho:** 4 universos cheios, com chasers e um EFX em 48 moving heads, abaixo de 2 ms por tick no app compilado.
- **Ponta a ponta:** rodar um EFX muda o pan e o tilt na saída a cada quadro.
- `pnpm check` passando. No navegador, o editor de EFX (ordem, knobs e grupos), com o EFX rodando e a saída conferida.

## Validação (com você)

**Os testes do EFX com os moving beams (2026-10-19, o seu pedido)** foram para a [validação final](../validacao-final.md): você ainda não pode testar com os Beams. São o círculo a 90°, o relativo, os desenhos, os fades, as ordens e a propagação, o EFX no chaser e na coleção, e o elemento "Tempos" com "Seguir o BPM".

Fica para agora:
- [ ] Um grupo em "Adicionar fixture" inclui as fixtures na ordem da grade.
- [ ] `pnpm check` passa.

Pendentes da F7, para testar junto (o seu pedido):
- os itens de [fase-07.md](fase-07.md#validação-com-você), menos o deslizar dos Beams;
- da F6, o tick com 4 universos, os fades de 3 s nos ParLed e o elemento "Tempos" nas cenas.

A posição dos Beams entre cenas e os fades nos Beams, da F6, também estão na validação final.

## Referências estudadas

- `engine/src/{efx,efxfixture,fixturegroup,grouphead}.cpp`
- `engine/src/{genericfader,universe}.cpp`: os canais relativos e a prioridade do EFX relativo
- `engine/test/{efx,efxfixture,fixturegroup}`
- `qmlui/qml/fixturesfunctions/EFXEditor.qml`: as ordens do EFX e a distribuição dos inícios

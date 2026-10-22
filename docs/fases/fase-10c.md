# F10c — Tempos, cue list e prioridade

**Status:** mini-plano aprovado em 2026-10-21, com as quatro recomendações. Entregue em 2026-10-21, aguardando a validação (ver [Entregue](#entregue)).

Itens da pesquisa da grandMA ([pesquisa-grandma.md](../pesquisa-grandma.md)): **R1** (cue list profissional), **R2** (delay e curvas), **R3** (tempos por tipo de atributo) e **R6** (prioridade por criação). Entram também o "Voltar" e o "Pausar" dos botões, que a F10 deixou para cá (E1), e dois itens do Backlog da F7: **pausar um chaser** e o **hold infinito**.

## Objetivo

Depois da F10c, um show roteirizado roda pelo console:
- a **cue list** anda no **Go**: cada passo pode esperar o Go (hold infinito) ou seguir sozinho; dá para **pausar** no meio de um fade, **voltar** e **escolher o próximo** sem disparar;
- cada cena e cada passo ganham **delay** (esperar antes de subir ou de descer) e **curva** do fade (não só a linear);
- na mesma cena, **cada tipo de atributo tem o seu tempo**: a posição em 6 s, a cor em 1 s, o dimmer em 2 s;
- cada criação tem uma **prioridade**: uma cena "base" de prioridade baixa nunca cobre um efeito de prioridade alta.

## O que já existe

- **Chaser (F7):** passos com fade in, hold e fade out (do passo, comuns ou os da criação), em tempo ou em batidas; Loop, Uma vez, Vai e volta e Aleatória; crossfade na troca de passo; "manter o último passo" no fim da "Uma vez".
- **Cue list do console (F7 e F10):** a lista dos passos com o atual aceso; play/stop, anterior e próximo; tocar num passo vai para ele na hora. Parado, "próximo" começa no primeiro passo.
- **Fades (F6):** lineares, em inteiros, do valor que está saindo; um fade in e um fade out por cena. Rodas de cor, gobo, prisma e strobo mudam de uma vez.
- **Mistura (F6):** dimmer e cores pelo maior valor (HTP); o resto, a última criação iniciada (LTP). Não há prioridade escolhida.
- **No motor:** cada cena acesa é um "fader" com os canais e os tempos dela; uma coleção acende várias de uma vez. O tempo de cada canal já é guardado à parte.

## O que entra

### 1. Cue list (R1)

- **Hold infinito (∞):** o hold de um passo (ou o comum) pode ser "∞": o passo entra com o fade dele e **espera o Go**. Com o hold comum em ∞, o chaser vira uma cue list manual; com alguns passos em ∞ e outros com tempo, partes do roteiro seguem sozinhas ("Follow" = hold 0, "depois de X segundos" = hold X, nas batidas = o chaser em batidas).
- **Pausar e continuar:** a criação pausada **congela**: o fade para onde está, o hold para de contar, o EFX e os chasers de dentro param. Continuar segue de onde parou. O **Go** com a criação pausada também continua.
- **Carregar a próxima:** escolher qual passo vem no próximo Go, sem disparar. O passo carregado fica marcado na lista.
- **No fim:** a "Uma vez" que apaga no fim já é o "Off"; o Go depois dela recomeça do primeiro passo (já é assim).
- **Elemento Cue List:** os botões passam a ser **Voltar**, **Go**, **Pausar** e **Parar**. Tocar num passo vai para ele (como hoje); o **toque longo** carrega o passo como o próximo. As teclas e o MIDI (F12) ganham o "Pausar".
- **Botões do console (E1):** duas ações novas, **Voltar** (o passo anterior do chaser do botão) e **Pausar** (pausa ou continua a criação do botão).

### 2. Delay e curvas (R2)

- **Delay:** cada cena e cada passo ganham o **delay de entrada** (espera antes de começar o fade in) e o **de saída** (espera antes do fade out). No passo, o delay de entrada conta na duração dele.
- **Curvas:** o fade pode ser **Linear** (como hoje), **Lento no começo**, **Lento no fim** ou **S** (lento nas duas pontas). A curva vale para o fade in e o fade out da cena ou do passo.
- Muda a decisão da F6 ("fades lineares"): o Linear continua sendo o padrão, com as mesmas contas de hoje.
- O elemento "Tempos" do console continua mexendo no fade in, no fade out e na duração; o delay e a curva ficam os gravados.

### 3. Tempos por tipo de atributo (R3)

- Na cena, além dos tempos dela, cada tipo de atributo pode ter os seus: **Intensidade** (dimmer), **Cor**, **Posição** (pan e tilt) e **Feixe** (zoom, foco, íris). Cada um com delay e fade de entrada e de saída; o que não for preenchido usa os tempos da cena.
- O que muda de uma vez (rodas, gobo, prisma, strobo) continua mudando de uma vez, no começo do fade in e no fim do fade out do tipo dele.
- No chaser, o passo em "Padrão" usa os tempos da cena, por tipo de atributo. O passo com tempo "Comum" ou "Por passo" impõe o mesmo tempo a todos os atributos, como hoje.
- **Como entra:** o servidor divide a cena em partes, uma por tipo de atributo com tempos próprios, e o motor as acende juntas, como uma coleção. O motor não ganha um conceito novo para isso.

### 4. Prioridade por criação (R6)

- Cada criação ganha a **prioridade**: Baixa, Normal (o padrão, que é a regra de hoje) ou Alta (decisão 2).
- **Mistura:** num canal disputado, a criação de prioridade maior ganha, também no dimmer e nas cores: o valor dela vale, mesmo que seja menor (decisão 3). Na mesma prioridade, vale a regra de hoje (HTP no dimmer e nas cores, LTP no resto).
- **Ao parar**, a de prioridade maior desce com o fade out dela até o que está embaixo, sem saltos.
- A prioridade é a da criação disparada: o que um chaser ou uma coleção acende usa a dele. Os sliders de nível ficam em Normal. A Prévia, a Mesa simples, o Localizar e o Grand Master continuam por cima de tudo.

### 5. Contrato, arquivo e protocolo

- A criação ganha a prioridade; a cena, os delays, a curva e os tempos por tipo de atributo; os tempos do chaser (comuns e de cada passo), os delays, a curva e o hold ∞.
- Operações novas: pausar e continuar uma criação, e carregar o próximo passo. O estado ganha as criações pausadas e o passo carregado de cada chaser.
- Nova versão do arquivo do projeto (9), com a migração: tudo abre como hoje (sem delay, Linear, Normal, sem tempos por atributo).
- `docs/protocolo.md` atualizado.

### 6. Telas (G2)

- **Editor de cena**, na seção "Tempos": delay e fade de entrada e de saída, a curva, e "Por tipo de atributo" (uma linha para cada tipo, vazia = os tempos da cena).
- **Editor de chaser:** o delay e a curva nos tempos comuns e nos do passo; o "∞" no hold.
- **Todos os editores do Studio:** a prioridade, ao lado dos tempos.
- **Elemento Cue List:** Voltar, Go, Pausar e Parar; o passo carregado marcado; o toque longo para carregar.
- **Propriedades do botão:** as ações Voltar e Pausar.
- O campo de tempo ganha o "∞" onde ele vale (só no hold dos chasers).

### 7. Exemplos no show de exemplo (G8)

Regra nova do usuário: cada recurso novo das criações vem com um exemplo pronto, no `exemplo.json` e no "Show de exemplo" salvo na pasta dele:
- uma cue list manual (hold ∞) com o Voltar, o Go, o Pausar e o Parar num elemento do console;
- uma cena com delay de entrada e de saída diferentes e a curva S;
- uma cena com a cor rápida e o dimmer lento (tempos por tipo de atributo);
- uma cena base de prioridade Baixa e outra Alta nos mesmos ParLeds;
- botões Voltar e Pausar no console.

## Decisões (aprovadas em 2026-10-21, as quatro recomendações)

1. **As curvas:** quatro prontas, Linear, Lento no começo, Lento no fim e S. A curva desenhada à mão fica para a F10d, junto da curva de velocidade do EFX.
2. **Os níveis de prioridade:** três, Baixa, Normal e Alta.
3. **A prioridade no dimmer e nas cores:** a maior assume o canal, como na grandMA. Uma cena Alta com o dimmer a 30% deixa o dimmer em 30%, mesmo que uma Baixa esteja a 100%; ao parar, desce até o que está embaixo.
4. **Carregar a próxima na cue list:** o toque longo no passo. O toque curto continua indo para o passo na hora.

## Fica de fora

- **Tracking e partes de cue (R4 e R5):** fora por ora (decisão de 2026-10-21). O R3 dá quase o mesmo resultado das partes.
- **Delay e curva no EFX e no RGB Matrix:** a F10d (EFX avançado) trata da curva de velocidade do EFX.
- **O elemento "Tempos" mexer no delay e na curva.**
- **Timecode:** fora por ora.
- **Os faders de crossfade manual da cue list do QLC+ (A/B):** só se você pedir.

## Testes automáticos

- **Motor:**
  - o delay segura o fade (de entrada e de saída) e conta na duração do passo;
  - cada curva nos pontos de 25%, 50% e 75%, e a Linear igual à de hoje;
  - pausar congela o fade, o hold, o EFX e o chaser de dentro, e continuar segue de onde parou, sem salto;
  - o hold ∞ espera o Go; o passo carregado é o próximo do Go;
  - a prioridade: a Alta ganha da Baixa no dimmer (mesmo com o valor menor) e no pan; ao parar, desce até o que está embaixo; na mesma prioridade, a regra de hoje.
- **Servidor:**
  - a cena com tempos por tipo de atributo vira as partes certas, e o passo "Comum" impõe um tempo só;
  - a prioridade da criação disparada vale para o que ela acende;
  - pausar, continuar e carregar pelo console (cue list e botões), com o estado avisado aos aparelhos;
  - a migração da versão 8.
- **No navegador, contra o seu `pnpm dev`:** os editores de cena e de chaser com os tempos novos, a prioridade e o elemento Cue List (Voltar, Go, Pausar, Parar e o toque longo). Como sempre, com backup antes e comparação depois.
- `pnpm check` passando.

## Entregue

- **Motor (`rlc-engine`):**
  - cada fader tem a prioridade da criação, os tempos de saída e a curva; a mistura vai da prioridade menor para a maior, e a maior assume o canal de intensidade; parada, desce até o que está embaixo (a intensidade da criação passa para o valor de partida, para não haver salto);
  - o delay de entrada deixa o fader fora da mistura até acabar, e o fade in parte do que está saindo nessa hora; o de saída segura o valor antes do fade out; parado ainda no delay de entrada, o fader sai sem aparecer;
  - as curvas no `fade`: a Linear faz a mesma conta inteira de antes (os números do QLC+ continuam); Lento no começo é x², Lento no fim é 1 − (1 − x)², e o S é x²(3 − 2x);
  - a pausa: a criação pausada não anda (o `tick` dela não roda), e a cada quadro os tempos dos faders e dos passos dela andam junto com o relógio (também a posição na batida, nos chasers em batidas); nada sai do lugar e, ao continuar, tudo segue de onde parou;
  - o passo impõe o delay, o fade e a curva (`Imposed`) a tudo o que dispara, também ao EFX; o hold infinito é o próprio `u32::MAX`, que nunca acaba.
- **Servidor:** a cena com tempos por tipo de atributo vira uma coleção de partes (`scene_times` e `scene_program`); a prioridade e a pausa vão em cada `Run`; as operações `setFunctionPaused` e `loadStep`; a tradução do console (`console_op`, antes `cue_list_op`) faz o Go, o Voltar, o Pausar e o Parar da cue list e os botões Voltar e Pausar; a velocidade (Rate) não mexe no ∞; projeto na versão 9.
- **Telas:** a prioridade fica na barra do editor do Studio (vale para todos os tipos), com o botão Pausar ao lado do Rodar; o editor de cena ganhou o delay, a curva e a tabela por tipo de atributo; o de chaser, a entrada (delay, fade e curva), o hold com ∞ e a saída (delay e fade), nos comuns e na tabela dos passos; a cue list do console tem Voltar, Go, Pausar e Parar e marca o passo carregado (toque longo); os botões ganharam as ações Voltar e Pausar. No Studio, Rodar/Parar e Pausar ficam só com o ícone e acesos (não o LED), na barra do editor e na lista; pausada, a criação fica acesa em amarelo (pedido na entrega).
- **Exemplos (G8):** no `exemplo.json` e no "Show de exemplo" do usuário, a pasta de cenas "Tempos e prioridade" (Base branca (baixa), Azul a 30% (alta), Delay e curva S, Cor no escuro), o chaser "Roteiro manual (Go)" e a página "Tempos" do console.
- **Testes:** 7 novos no motor (curvas, delays, pausa da cena e do chaser, hold ∞, delay no passo, prioridade), 3 no servidor (tempos, partes e prioridade; o Go, o Voltar, o Pausar e o passo carregado; a pausa e o passo carregado acompanhando o chaser) e a migração da versão 8. No navegador, contra o `pnpm dev` com o "Show de exemplo": 37 conferências (os exemplos, a prioridade, o delay e a curva S, a cor no escuro, a cue list e os botões, os editores), com as edições desfeitas no fim.

### Diferenças do plano

- **A prioridade na barra do editor**, logo abaixo do nome, e não "ao lado dos tempos": vale para todos os tipos de criação, e cada editor tem os tempos num lugar diferente.
- **A curva no chaser segue a entrada** ("Padrão", "Comum" ou "Por passo" do fade in), como o delay de entrada; o delay de saída segue a saída. No editor, as linhas viraram "Entrada", "Hold" e "Saída".
- **O EFX num passo de chaser** também respeita o delay e a curva impostos pelo passo (o EFX continua sem delay e curva próprios).
- **O Go da cue list é o principal**: o "próximo" das cue lists antigas (teclas e MIDI) virou o Go na migração; com o principal já usado, o "próximo" sai.
- **O "Feixe" virou "Efeitos"**, o nome da aba do editor de cena, para as duas telas falarem igual.

## Validação (com você)

Com os ParLeds, abrindo o **Show de exemplo** (a página **Tempos** do console e a pasta **Tempos e prioridade** do Studio); o que precisa dos Beams vai para a validação final.
- [ ] **Roteiro manual (Go)**, na página Tempos: o Go começa no passo 1, e cada passo espera o próximo Go (hold ∞); Voltar volta; o toque longo num passo o marca em amarelo, e o Go vai para ele.
- [ ] Pausar no meio do passo 2 ("Entrada lenta", 3 s em S depois de 1 s) congela a luz; o Go (ou o Pausar) continua de onde parou.
- [ ] **Delay e curva S**: espera 1 s e sobe em S (lento nas pontas) em 3 s; ao parar, fica 2 s acesa e desce em 3 s.
- [ ] **Cor no escuro**: a cor muda na hora e o dimmer sobe depois de 2 s; ao parar, o dimmer desce em 3 s e a cor sai depois.
- [ ] **Base branca (baixa)** a 100% e **Azul a 30% (alta)**: a Alta assume (30% azul), mesmo disparando a Baixa de novo depois; parar a Alta volta ao branco a 100% com o fade.
- [ ] Os botões Voltar e Pausar da página Tempos (o Pausar acende enquanto o roteiro está pausado).
- [ ] Os projetos antigos abrem como antes (o RSena Palco abriu da recuperação, com a "Cena 8").
- [ ] `pnpm check` passa.
- Na [validação final](../validacao-final.md): a posição lenta com a cor rápida nos Beams.

## Referências estudadas

- Do projeto: o motor (`crates/rlc-engine/src/lib.rs`: os faders, o `fade` linear e a mistura; `chaser.rs`: os passos, o `over` e o `next`), a tradução no servidor (`program` e `scene_program` em `apply.rs`), o elemento Cue List (`ConsoleWidget.vue`) e o editor de chaser (`StudioChaser.vue`).
- grandMA3: os itens R1, R2, R3 e R6 da [pesquisa](../pesquisa-grandma.md).
- QLC+ 5.3.0 (só leitura):
  - **Pausa** (`engine/src/function.cpp`, `chaser.cpp`, `chaserrunner.cpp`, `genericfader.cpp`, `efx.cpp`): a criação pausada para de contar o tempo; o fade congela no meio (o fader continua escrevendo o valor de agora), o hold para, o EFX, o RGB Matrix e a coleção também param. É o que a F10c faz. No QLC+, só a última cena em fade out do chaser é pausada; aqui, tudo o que a criação acendeu.
  - **Cue list** (`ui/src/virtualconsole/vccuelist.cpp`, `qmlui/virtualconsole/vccuelist.cpp`): dois arranjos de botões ("Play/Pause + Stop" e "Play/Stop + Pause"); pausada, "próximo" e "anterior" só mexem na seleção. Parada, o comportamento de "próximo" e "anterior" é uma opção (começar no primeiro, no seguinte, só selecionar ou nada); o nosso é o "começar no primeiro", como hoje. Os faders laterais de crossfade manual (A/B e por passos) ficam de fora.
  - **Hold infinito** (`function.cpp`, `chaserrunner.cpp`): um valor especial ("∞") no tempo; o passo nunca acaba sozinho e só sai com próximo, anterior, ir para o passo ou parar. O mesmo aqui. O QLC+ aceita "∞" também no fade, sem tratamento (o fade dura ~49 dias): aqui o "∞" só vale no hold.
  - **Não existem no QLC+:** delay por passo, curvas de fade (a rampa é linear, `fadechannel.cpp`) e tempos por atributo (a cena usa o mesmo fade em todos os canais). Vêm da grandMA.
  - **Prioridade:** o QLC+ só tem prioridade de fader, fixa pelo tipo (`Universe::FaderPriority`: cena normal, override, flash, Simple Desk), não escolhida por criação. No HTP, o valor menor só vence com "override". A prioridade escolhida por criação vem da grandMA.

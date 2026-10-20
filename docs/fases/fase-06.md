# F6 — Merge, fades e Cena

**Status:** mini-plano aprovado em 2026-10-17, com as decisões 3 a 7 como recomendado. As mudanças:
- **Decisão 1:** as cores misturam pelo maior valor (HTP) em todas as fixtures, como no QLC+. Azul e vermelho rodando juntos dão magenta. Para uma cor por vez, o usuário usa a moldura solo.
- **Decisão 2:** ao parar uma cena, os canais LTP voltam ao que está embaixo (outra cena rodando ou o valor padrão) durante o fade out. O QLC+ os mantém onde estão. A regra fica num lugar só do motor, para voltar ao jeito do QLC+ se o usuário preferir.

Entregue em 2026-10-18. Na validação, o usuário aprovou 8 dos 12 itens; os outros 4 ainda não foram testados e serão ajustados depois, se precisar. Concluída em 2026-10-18, a pedido do usuário; o ajuste do elemento "Tempos" também ficou sem testar e pode ser conferido na validação da F7.

**Ajuste pedido na validação (2026-10-18)**
- **O elemento "Tempos" do console (VC-3) controla os fades das cenas da lista.** Antes ele só guardava os valores, e as cenas entravam sempre com o fade gravado.
  - O fade in vale na entrada e o fade out na saída, inclusive numa troca de cenas (numa moldura solo, a que sai e a que entra).
  - Mexer no elemento com a cena rodando vale para a saída dela.
  - "Gravados" volta aos tempos de cada cena. Um tempo desligado no elemento (ex.: o fade out) fica com o gravado.
  - O tempo gravado na cena não muda. No QLC+, o Speed Dial grava o tempo na função.
  - Com dois elementos para a mesma cena, vale o último mexido.
  - A duração (o hold dos chasers) entra na F7.
- **Testes:** no servidor, os tempos de cada cena com e sem o elemento. Ponta a ponta, o fade in em 0 faz a cena entrar direto, e o fade out mudado com a cena rodando vale ao parar. O teste falha se o aviso do elemento não chegar ao motor.

**Entregue**
- **Motor (`rlc-engine`):**
  - um fader por cena rodando, na ordem de início, com o fade in e o fade out dela;
  - a mistura HTP/LTP;
  - as cenas paradas em fade out até o fim;
  - o fade em 16 bits nos canais com fino;
  - os canais sem fade;
  - o corte na hora (`Cut`).
- **Servidor:**
  - traduz as cenas rodando para endereços DMX (`running_scenes`), com o fino junto do grosso;
  - reenvia a cada mudança das criações rodando, das cenas, das fixtures ou do patch;
  - o patch leva quais canais fazem fade.
- **Pânico:** a operação ganhou o campo `all`. O botão da barra de cima manda `panic` já no primeiro toque, e o pânico, o "Novo" e o "Abrir" cortam as cenas na hora.
- **Telas:** nenhuma mudou. O `docs/protocolo.md` descreve as camadas da saída e o `panic { all }`.
- **Diferenças do plano:** nenhuma.
- **Desempenho:** 8 cenas em fade, cada uma com os 2.048 canais de 4 universos, dão 0,2 ms por tick no app compilado e 1,1 ms nos testes sem otimização.
- **Testes:**
  - automáticos:
    - fades com os números do QLC+;
    - cena com fade in, edição ao vivo e fade out;
    - mistura HTP/LTP;
    - rodar de novo no meio do fade out;
    - canais sem fade;
    - 16 bits;
    - corte na hora;
    - camadas;
    - Grand Master;
    - desempenho;
    - endereços e pares de 16 bits das cenas rodando;
    - quais canais fazem fade;
  - ponta a ponta: o fade in chega na saída, a edição vale na hora, e o pânico apaga sem fade (o teste falha se o corte for desligado);
  - no navegador, contra o `pnpm dev`, com o "Show de exemplo":
    - "Rodar" no Studio com o fade in de 1 s;
    - azul + âmbar misturando as cores;
    - pânico sem fade;
    - "Parar todas" com fade;
    - o pan dos MHs em 16 bits, voltando ao padrão ao parar;
    - o Studio no celular.
  - Depois do teste: a cópia de recuperação que você tinha (o "Novo projeto" com 14 fixtures) foi devolvida, e o `settings.json`, a biblioteca e a pasta de projetos ficaram como estavam.

## Objetivo

Fazer as cenas acenderem a luz. Depois da F6:
- "Rodar" uma cena (no Studio ou num botão do console) acende a luz com o fade in dela, e "Parar" apaga com o fade out;
- várias cenas rodando se misturam como no QLC+: na intensidade e nas cores vence o maior valor e, no resto, a última iniciada;
- a cena pode ser editada enquanto roda, e a luz muda na hora;
- Grand Master, blackout, Stop All e pânico agem sobre as cenas rodando.

## O que já existe (F2 a F5)

- **Rodar e parar:** no Studio e nos botões do console. A lista das criações rodando já guarda a ordem de início.
- **Fades da cena:** fade in e fade out gravados no editor de cena.
- **Motor:** as camadas da saída (padrão, prévia, Mesa simples, localizar, Grand Master e blackout), o Grand Master no modo "reduzir" e o blackout só na intensidade.

## O que entra

### 1. Mistura das cenas rodando (HTP e LTP)

- **Intensidade (HTP):** o dimmer e as cores (vermelho, verde, azul, branco, âmbar, UV, ciano, magenta, amarelo…). Vence o maior valor entre as cenas rodando, como no QLC+.
- **O resto (LTP):** pan, tilt, rodas, gobo, prisma, foco, zoom… Vence a cena iniciada por último.
  - Uma cena rodando reafirma os seus valores a cada quadro.
  - Parar e rodar de novo põe a cena por último, e ela volta a vencer.
- **Ao parar uma cena** (decisão 2), durante o fade out dela:
  - a intensidade desce até 0. Se outra cena acende o mesmo canal, a luz desce só até o valor dela;
  - os canais LTP deslizam até o que está embaixo: o valor de outra cena rodando ou, sem nenhuma, o valor padrão.
- **Canal sem nenhuma criação:** fica no valor padrão, como hoje.

### 2. Fades

- **Linear no tempo, com contas em inteiros.** Dão os mesmos números dos testes do QLC+. O plano falava em f32.
  - O primeiro quadro já anda um passo (são 50 passos por segundo).
  - Um fade de até 20 ms é imediato.
- **O fade começa do valor que está saindo agora:**
  - um dimmer já aceso por outra cena não pisca;
  - os moving heads vão da posição atual para a da cena;
  - rodar de novo uma cena no meio do fade out dela continua de onde estava.
- **O que muda de uma vez** (decisão 3): rodas de cor e de gobo, prisma, strobo/shutter, macros, efeitos, velocidades, manutenção e "outros".
  - Mudam no começo do fade in e no fim do fade out, com a luz já apagada.
  - Fazem fade: intensidade e cores, pan e tilt, foco, zoom, íris e temperatura de cor.
- **16 bits:** pan e tilt (e qualquer canal com fino na fixture) fazem o fade em 16 bits, para os moving heads andarem sem degraus.
  - O editor de cena grava só o canal grosso. Sem o fino na cena, ele vale 0, e o movimento fica liso do mesmo jeito.
  - O k-ésimo fino de um atributo no modo é do k-ésimo canal grosso dele ("Pan, Tilt, Pan fino, Tilt fino").
  - Uma cena só com o fino muda só o fino. O QLC+ leva o grosso a 0 nesse caso.

### 3. Cena

- **Rodar:** a cena começa a misturar com as outras, com o seu fade in.
- **Parar:** a cena sai da lista na hora (o LED apaga) e o fade out continua sozinho no motor.
- **Edição com a cena rodando** (decisão 7):
  - mudar um valor vale no quadro seguinte, sem fade e sem reiniciar a cena, para cima ou para baixo, como no QLC+ 5;
  - tirar um canal ou uma fixture da cena solta o canal na hora. No QLC+, o canal continua aceso até a cena reiniciar.
- **Desfazer** uma edição de uma cena rodando também muda a luz na hora.

### 4. Camadas da saída

De baixo para cima:
1. valor padrão;
2. **criações rodando** (a mistura da seção 1);
3. prévia, por cima das criações: os canais da cena em edição aparecem exatamente como estão (decisão 4);
4. Mesa simples (vence tudo o que está embaixo, também na intensidade, como no QLC+);
5. localizar;
6. Grand Master e blackout.

### 5. Grand Master, blackout, Stop All e pânico

- **Grand Master:** continua só no modo atual, reduzir a intensidade (dimmer e cores), e age depois da mistura (decisão 6).
- **Blackout:** instantâneo, como hoje. As cenas continuam rodando por baixo e, ao soltar, tudo volta.
- **Stop All** (console) e **"Parar todas"** (Studio): cada cena apaga com o seu fade out, como no QLC+ (decisão 5).
- **Pânico:** corta na hora, sem fade, já no primeiro toque. "Novo" e "Abrir" também cortam.
  - O botão mandava o `stopAllFunctions` no primeiro toque. Ele passa a mandar `panic { all: false }`, e o segundo toque manda `panic { all: true }`, que também zera a Mesa simples.

### 6. Servidor e motor

- O servidor traduz cada cena rodando para endereços DMX, com o canal fino junto do grosso. O motor continua sem conhecer fixtures (A1).
  - A lista vai na ordem de início e é reenviada a cada mudança das criações rodando, da cena, do patch ou das fixtures.
  - O patch de cada universo leva também quais canais fazem fade.
- O motor guarda, por cena rodando:
  - os valores de partida e de chegada;
  - o início do fade;
  - as cenas paradas ainda em fade out, até o fim dele.
- O valor de partida de um fade vem do último quadro calculado. No QLC+, ele é lido de outra thread no meio do cálculo.
- Chasers, sequências, coleções, EFX e RGB Matrix continuam sem acender até a F7, a F8 e a F9.

### 7. Telas (G2)

Nenhuma tela muda. No `docs/protocolo.md`:
- a descrição de `setFunctionRunning` perde o "até a F6, não acende nada";
- o `panic` ganha o campo `all`.

## Decisões para você

1. **Cores na mistura.** **Resposta:** como no QLC+, as cores sempre misturam pelo maior valor (HTP). Para uma cor por vez, a moldura solo.
2. **Canal LTP que só uma cena controlava, ao parar.** **Resposta:** volta ao valor padrão no fade out. Pode voltar a ficar onde está, como no QLC+, se o usuário não gostar.
3. **Rodas, gobo, prisma, strobo e macros mudam de uma vez**, sem fade (recomendado, aprovado).
4. **A prévia fica por cima das criações rodando** (recomendado, aprovado).
5. **Stop All e "Parar todas" usam o fade out de cada cena; o pânico corta na hora** (recomendado, aprovado).
6. **Grand Master só no modo atual**, reduzir a intensidade (recomendado, aprovado). Os modos "limitar" e "todos os canais" saem do plano.
7. **Edição da cena rodando vale na hora**, sem fade (recomendado, aprovado).

## Fica de fora

- **Intensidade por criação** (sliders de playback e submaster): na F10, sobre estes fades.
- **Fades ajustados ao vivo pelo elemento "Tempos"** (VC-3): na F10/F11. _Entraram na F6, a pedido do usuário na validação (veja o ajuste no começo)._
- **Flash do QLC+** (os valores entram na hora, sem fade in): na F10. Até lá, o botão do console em modo flash roda a cena enquanto está apertado, com os fades dela.
- **Modos de mistura** (máscara, soma e subtração) e **canais relativos** (do EFX relativo, na F8).
- **Pausar uma criação.**
- **Tempo em batidas** nas cenas: o BPM chega na F7.
- **HTP/LTP forçado e "pode fazer fade" por canal:** continuam no backlog.

## Testes automáticos

- **Fade** (portados de `fadechannel_test`):
  - 0→250 em 1 s dá 5, 10, 15…;
  - um fade de até 1 tick é imediato, e 2 ticks dão 125 e 250;
  - 13→147 e 245→101 batem com os números do QLC+.
- **Cena** (portados de `scene_test`):
  - fade in e fade out em HTP e LTP;
  - edição ao vivo subindo e descendo;
  - depois de parar, a intensidade em 0.
- **Mistura:**
  - duas cenas: vence o maior valor no dimmer e nas cores (azul + vermelho = magenta) e o pan da última iniciada;
  - parar a última faz o pan deslizar para o da cena de baixo e, sem nenhuma, para o padrão;
  - rodar de novo no meio do fade out continua do valor atual.
- **16 bits:** pan com grosso e fino; só com o grosso, o fino anda durante o fade.
- **Canais que mudam de uma vez:** no começo do fade in e no fim do fade out.
- **Camadas:** padrão, criações, prévia, Mesa simples, localizar, Grand Master e blackout.
- **Grand Master** (portado de `universe_test`): a 63, os valores 10, 20 e 50 viram 2, 5 e 12, e o pan não muda.
- **Stop All** com o fade out de cada cena, e o pânico, o "Novo" e o "Abrir" na hora.
- **Desempenho:** tick abaixo de 2 ms com 4 universos cheios e várias cenas em fade.
- **Servidor:** endereços, pares de 16 bits e ordem das cenas rodando; edição ao vivo; tirar uma fixture da cena rodando.
- `pnpm check` passando, e um teste no navegador.

## Validação (com você)

Antes de começar:
- **Reinicie o `pnpm dev`.** O app abre a sua cópia de recuperação ("Novo projeto", com 14 fixtures). Se quiser guardá-la, use "Salvar como" antes de abrir o "Show de exemplo": abrir outro projeto descarta a recuperação.
- **O "Show de exemplo" não tem cenas no Rig RSena** (as dele estão no universo Palco, na saída Dummy). Crie duas cenas de cor nos ParLed (ex.: azul e vermelho) e duas de posição nos Beams, com fade de 3 s.
- **Moldura solo:** para testar uma cor por vez, ponha os botões das cenas de cor numa moldura solo do console.

Com o rig e a Numbstar:
- [x] Duas cenas sobrepostas: vence o maior dimmer, e as cores se misturam (azul + vermelho = magenta); numa moldura solo, uma cor por vez.
- [ ] Na posição dos Beams, vence a última cena iniciada; ao parar a última, eles voltam para a posição da anterior. _Na [validação final](../validacao-final.md), com os moving beams._
- [x] Grand Master a 50% reduz só a intensidade; pan, tilt e gobo não mudam.
- [x] Fade in e fade out de 3 s suaves nos ParLed. _Os Beams sem degraus e a roda de cor e o gobo de uma vez: na [validação final](../validacao-final.md)._
- [x] Editar uma cena rodando muda a luz na hora.
- [ ] Parar a única cena de posição: os Beams voltam à posição padrão durante o fade out. _Na [validação final](../validacao-final.md), com os moving beams._
- [x] Stop All apaga com o fade out de cada cena; o pânico apaga na hora.
- [x] Blackout instantâneo com cenas rodando; ao soltar, tudo volta.
- [x] Um botão do console liga e desliga a cena, com os fades.
- [x] Celular e desktop sincronizados.
- [ ] Tick abaixo de 2 ms com 4 universos (no Monitoramento). _Na [validação final](../validacao-final.md): o usuário não pôde testar em 2026-10-20._
- [x] `pnpm check` passa.
- [x] O elemento "Tempos" troca o fade in e o fade out das cenas da lista, na entrada, na saída e na troca de cenas. "Gravados" volta aos tempos da cena. _Ajuste pedido na validação._

## Referências estudadas

- `engine/src/{universe,genericfader,fadechannel,function,scene,scenevalue,grandmaster,mastertimer}.cpp`
- `engine/test/{universe,genericfader,fadechannel,scene,grandmaster}`
- `qmlui/{sceneeditor,functionmanager}.cpp` (edição da cena rodando e as duas prévias do QLC+ 5)

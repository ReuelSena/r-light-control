# F7 — Chaser, Sequência, Coleção e BPM

**Status:** mini-plano aprovado em 2026-10-18, com as decisões 1 a 5 como recomendado. A mudança:
- **Decisão 6:** o fim do Single Shot vira uma opção em cada chaser (e sequência): apagar depois do último passo (o padrão, como no QLC+) ou manter o último passo aceso até alguém parar o chaser.

Entregue em 2026-10-18. Concluída em 2026-10-19, a seu pedido, sem a validação: os itens do checklist serão testados junto com a F8.

**Ajuste pedido na validação (2026-10-19): o elemento "Tempos" segue o BPM master.** O BPM do master e os segundos do elemento, cada um com o seu TAP, confundiam.
- **"Seguir o BPM":** chave nova nas propriedades do elemento, ligada por padrão (também no "Fades da frente" e nos outros que já existem).
  - **Ligada:** fade in, fade out e duração em batidas: 0, 1/8, 1/4, 1/2, 1, 2, 4, 8 e 16. Embaixo de cada knob, o tempo em segundos pelo BPM de agora.
  - O TAP do elemento passa a ser o mesmo do master ("TAP BPM"). Um tap (no master ou no elemento) muda o BPM, e todos os elementos que seguem o BPM e os chasers da lista acompanham na hora.
  - Os chasers da lista andam nas batidas, mesmo os que estão em segundos. Um passo de 1/2 ou 1/4 de batida também cai no tempo: o QLC+ só conta batidas inteiras, e aqui o motor acompanha a posição dentro da batida.
  - O tap conta como uma batida: os chasers em batidas se acertam com ele.
  - **Desligada:** segundos e TAP próprio, como antes. Trocar a chave volta o elemento aos tempos gravados (o valor de uma unidade não serve na outra).
- **Arquivo do projeto:** versão 3, com a chave. As versões anteriores abrem com ela ligada.
- **Testes:**
  - automáticos: passos de meia batida no tempo (12 e 13 quadros, sem deriva); o tap pouco antes de uma batida conta como ela; os tempos do elemento em batidas nas cenas e nos chasers (também nos em segundos); trocar a chave solta o valor; a migração das versões 1 e 2;
  - no navegador, contra o seu `pnpm dev`: o "Fades da frente" abriu seguindo o BPM; "1/2 bt" com os segundos embaixo; o TAP do master a cada 500 ms deu 113 BPM, e o "Tap BPM" do elemento a cada 1 s deu 60 BPM, com os segundos acompanhando; o "Troca de cores" andou a cada 502 ms em média (meia batida a 60 BPM); desligar a chave voltou o elemento aos segundos.
- Entregue em 2026-10-19 (a validação vai junto com a F8).
- **Em aberto (sugestão, sem decisão):** trocar o texto "Tempos em batidas do BPM" do editor de chaser por "Seguir o BPM", para as duas telas falarem igual.

**Entregue**
- **Motor (`rlc-engine`):**
  - as criações rodando chegam como uma árvore de cenas, chasers e coleções. Cada cena acesa ganha um fader, e a mistura da F6 continua igual;
  - o chaser (`chaser.rs`): passos no tempo e nas batidas, as quatro ordens, "para trás", crossfade, próximo, anterior e ir para um passo, "manter o último passo", edição ao vivo e chaser dentro de um passo;
  - as batidas do BPM, sem deriva, recomeçando no tap.
- **Servidor:**
  - traduz as criações rodando para o motor: os passos, os tempos de cada modo, o elemento "Tempos", a sequência e a coleção;
  - recusa a criação que contém a si mesma;
  - a operação `stepChaser` (a cue list) e o passo atual (`chaserSteps`), acompanhado a cada 40 ms. O Single Shot que apagou no fim sai das criações rodando;
  - o arquivo do projeto passa à versão 2, com a primeira migração.
- **Telas:**
  - a chave "Manter o último passo", só com a ordem "Uma vez";
  - o LED do passo no Studio e a cue list mostram o passo do motor;
  - as listas de "Adicionar passo" e da coleção escondem as criações que conteriam a própria.
- **Diferenças do plano** (detalhes que o plano não dizia):
  - Um chaser sem passos, ou uma coleção só com EFX e RGB Matrix, fica rodando sem acender nada. No QLC+, o chaser sem passos para sozinho.
  - "Anterior" no primeiro passo de um Single Shot não faz nada. No QLC+, ele para o chaser.
- **Desempenho:** 8 cenas em fade e 4 chasers trocando de passo a cada 3 quadros, todos com os 2.048 canais de 4 universos: menos de 1 ms por tick no app compilado.
- **Testes:**
  - automáticos:
    - a ordem dos passos (os `write…Five` do QLC+) e o Random;
    - o crossfade e o fade da cena contando na duração do passo;
    - a cue list (o `nextPrevious` do QLC+), também com o chaser parado;
    - o Single Shot com e sem "manter o último passo";
    - as batidas: 128 BPM por um minuto sem deriva, e o tap;
    - a edição ao vivo, o chaser dentro de um passo e a coleção;
    - no servidor, a tradução dos tempos e do elemento "Tempos", a criação que contém a si mesma, a migração da versão 1 e o passo atual;
  - ponta a ponta: a cue list leva o chaser a um passo, e o passo chega ao aparelho; o chaser anda sozinho; o Single Shot apaga e sai das criações rodando;
  - no navegador, contra o seu `pnpm dev` (o seu "Show de exemplo" recuperado):
    - o Studio rodando, com o LED do passo;
    - a cue list: play, toque num passo, próximo, anterior e "próximo" com o chaser parado;
    - a chave só no "Uma vez";
    - a lista de "Adicionar passo" sem a "Abertura";
    - o editor no celular.
  - O teste no navegador achou um bug, já corrigido: a conta da tela que esconde as criações que conteriam a própria esvaziava a lista da coleção na tela.
  - Depois do teste, as edições foram desfeitas, e o `settings.json`, o projeto salvo e a recuperação ficaram iguais aos de antes.
- **No seu projeto:** o elemento "Fades da frente" tem o "Troca de cores" na lista. Com um valor no elemento, os tempos dele valem no lugar dos do chaser; "Gravados" volta aos tempos do chaser.

## Objetivo

Fazer acender as criações do Studio que ainda não acendem, menos o EFX e o RGB Matrix. Depois da F7:
- "Rodar" um chaser ou uma sequência anda pelos passos sozinho, com crossfade entre eles, no tempo (segundos) ou nas batidas do BPM;
- a coleção dispara e para várias criações juntas;
- o tap da seção master acerta o BPM, e os chasers em batidas acompanham;
- a cue list do console mostra o passo de verdade e anda com anterior, próximo e toque num passo;
- o elemento "Tempos" também muda os tempos dos chasers.

## O que já existe (F2 a F6)

- **Studio:** os editores de chaser, sequência e coleção (passos, ordem, sentido, batidas e de onde vem cada tempo). O LED do passo atual é simulado na tela.
- **Console:** a cue list (simulada), o BPM com tap na seção master (só guarda o número) e o elemento "Tempos" (já muda os fades das cenas).
- **Motor (F6):** as cenas rodando, cada uma com o seu fader, misturadas em HTP e LTP na ordem de início, com o fade partindo do valor que está saindo.

## O que entra

### 1. Chaser

- **Passos:** cada passo dispara a criação dele (cena, outro chaser, sequência ou coleção).
  - O passo dura o fade in mais o hold. Quando acaba, o próximo entra com o fade in dele e o atual sai com o fade out dele, ao mesmo tempo: é o crossfade, como no QLC+.
  - O passo que entra parte do valor que está saindo (F6). Um dimmer aceso nos dois passos não pisca, e a posição desliza de um passo para o outro.
  - O passo que entra é o último iniciado: na posição (LTP), ele vence.
  - Um passo dura no mínimo um quadro (20 ms).
- **De onde vem cada tempo** (fade in, hold e fade out), como no QLC+:
  - **Padrão:** o da criação do passo (os fades gravados na cena ou, se ela está num elemento "Tempos", os dele). No hold, "Padrão" vale como "Comum" (decisão 5).
  - **Comum:** o mesmo para todos os passos.
  - **Por passo:** o de cada passo.
  - Os tempos do passo valem para tudo o que ele dispara (as cenas de uma coleção ou de outro chaser).
- **Ordem** (portada do QLC+):
  - **Loop:** depois do último, volta ao primeiro.
  - **Single Shot:** anda uma vez pelos passos. No fim, conforme a opção do chaser (decisão 6):
    - **apagar** (o padrão, como no QLC+): o último passo apaga com o fade out dele, e o chaser para (o LED apaga);
    - **manter o último passo:** o último passo fica aceso, e o chaser continua rodando até alguém pará-lo.
  - **Ping Pong:** vai e volta sem repetir as pontas (1, 2, 3, 2, 1, 2…).
  - **Random:** a cada volta, uma ordem sorteada com todos os passos, sem repetir o mesmo passo seguido na virada.
- **Para trás:** começa pelo último passo e anda ao contrário.
- **Parar:** o passo atual (e o que ainda estiver saindo) apaga com o seu fade out. Stop All e "Parar todas" fazem o mesmo; o pânico corta na hora.
- **Rodar de novo** um chaser que está apagando: recomeça do primeiro passo, partindo do valor que está saindo.
- **Edição com o chaser rodando:** vale na hora, sem recomeçar.
  - O passo atual continua, com os valores e os tempos novos. Com um hold menor que o tempo que já passou, o passo troca na hora.
  - A ordem, o sentido e os passos novos valem a partir da próxima troca.
  - Se o passo atual deixar de existir, o chaser vai para o próximo.
- **Uma criação não contém a si mesma** (decisão 4): o servidor recusa um chaser ou uma coleção que se inclua, direto ou por outra criação. As listas de "Adicionar passo" e da coleção não mostram essas criações.

### 2. Sequência

- É um chaser preso a uma cena: cada passo acende os canais da cena com os valores daquele passo.
- No modo "Padrão", os fades são os da cena.
- Editar os valores do passo atual com a sequência rodando muda a luz na hora, como na cena (F6).

### 3. Coleção

- Rodar dispara todas as criações dela ao mesmo tempo, cada uma com os seus fades. Parar apaga todas, cada uma com o seu fade out.
- Vale também a coleção dentro de outra coleção ou de um passo.
- Ela para sozinha quando tudo o que disparou terminou (ex.: só chasers Single Shot). Com uma cena dentro, só para quando alguém a para.
- Só a coleção aparece rodando, e não as criações dela (decisão 1).

### 4. BPM e batidas

- **BPM global:** o motor gera as batidas no ritmo do BPM da seção master, sem deriva (cada batida conta a partir da anterior, e não do quadro).
- **Tap:** já existe (a média dos últimos toques; uma pausa de 2 s recomeça). Ao mudar o BPM, a batida recomeça no último toque, para o chaser cair no tempo da música.
- **Chaser em batidas** (a chave "Tempos em batidas"):
  - o primeiro passo entra na hora; daí em diante, os passos trocam sempre numa batida;
  - o tempo do passo conta batidas inteiras, como no QLC+: um passo de 1,5 batida dura 2;
  - o fade em batidas vira tempo pelo BPM do momento em que o passo começa;
  - mudar o BPM com o chaser rodando vale a partir da próxima batida.
- O `setBpm` recusa valores fora de 30 a 300.
- O BPM continua fora do projeto (A2): abrir outro projeto não muda o BPM, e ele volta a 120 ao reiniciar o app.

### 5. Cue list do console (decisão 3)

- Mostra o passo atual de verdade, igual em todos os aparelhos e no Studio.
- O play roda o chaser do começo; o stop para.
- Próximo e anterior trocam na hora, com os fades do passo, como no QLC+. Com o chaser parado, "próximo" começa no primeiro passo e "anterior" no último.
- No último passo de um Single Shot, "próximo" faz o mesmo que o fim do chaser: apaga e para, ou não faz nada com "manter o último passo".
- Tocar num passo vai direto para ele (rodando o chaser, se estiver parado), e o chaser continua dali no tempo dele.
- As teclas da cue list fazem o mesmo.

### 6. Elemento "Tempos" nos chasers (decisão 2)

- O fade in, o fade out e a duração do elemento valem para todos os passos dos chasers e das sequências da lista, em qualquer modo, no lugar dos tempos gravados.
- Num chaser em batidas, a duração do elemento não vale (quem manda é o BPM); os fades valem, em segundos.
- Numa coleção da lista, o elemento vale para as criações dela.
- Mexer no elemento com o chaser rodando vale na hora, também no passo atual.
- "Gravados", o tempo desligado no elemento e "vale o último mexido" seguem como nas cenas (F6).

### 7. Servidor e motor

- **O motor roda os chasers** (A1): o tempo de cada passo, a ordem, as batidas e as trocas ficam na thread de 50 Hz, como o `ChaserRunner` do QLC+ no MasterTimer.
- O servidor manda ao motor as criações rodando, na ordem de início, já resolvidas:
  - as cenas nos endereços DMX, como na F6;
  - cada chaser com os passos, o que cada passo dispara e os tempos que valem;
  - cada coleção com a lista do que ela dispara.
- Cada passo usa os faders das cenas da F6: entrar é começar um fader por último na mistura; sair é o fade out dele.
- **Passo atual:** o motor publica o passo de cada chaser junto do quadro, e o servidor avisa os aparelhos quando ele muda. Um chaser Single Shot que apagou no fim, ou uma coleção que terminou, sai das criações rodando.
- **Contrato:** o chaser e a sequência ganham a opção "manter o último passo". O arquivo do projeto passa para a versão 2, com a primeira migração (A5): os projetos salvos abrem com "apagar".
- **Protocolo:**
  - uma operação nova para próximo, anterior e ir para um passo;
  - um pedaço novo do estado ao vivo, com o passo atual de cada chaser rodando;
  - a cue list deixa de guardar o passo escolhido em `widgetValues`;
  - o `setFunctionRunning` perde o "as outras criações acendem a partir da F7".
- **NOTICE (G3):** a ordem dos passos e as batidas, portadas de perto do `ChaserRunner` e do `MasterTimer`.

### 8. Telas (G2)

- **Uma mudança de tela (decisão 6):** no editor de chaser e de sequência, a chave "Manter o último passo", logo abaixo de "Tempos em batidas", no mesmo padrão dela (design system). Ela só aparece com a ordem Single Shot e começa desligada.
- O resto só deixa de ser simulado:
  - o LED do passo atual no editor de chaser e na cue list passa a mostrar o passo do motor;
  - as listas de "Adicionar passo" e da coleção escondem as criações que conteriam a própria.

## Decisões para você

1. **O botão e o LED de uma cena que um chaser ou uma coleção acendeu.** Exemplo: o chaser "Troca de cores" está no passo Azul, e o console tem um botão para a cena Azul. **Resposta:** só o chaser acende.
   - **Só o chaser acende** (recomendado, aprovado): o botão "Azul" fica apagado. Apertá-lo roda a Azul por conta própria, à parte do chaser: o chaser sair do passo Azul, ou parar, não apaga essa Azul.
   - **Como no QLC+:** o botão "Azul" ganha um destaque fraco enquanto o chaser está nela. O primeiro toque passa a cena para o botão (aceso). O segundo apaga a cena, inclusive dentro do chaser, até ele voltar a ela. Precisa de um terceiro visual no botão (muda a tela).
   - Vale o mesmo para as criações de uma coleção e para os LEDs do Studio.
2. **O elemento "Tempos" vale para todos os passos dos chasers, em qualquer modo, e para as criações das coleções.** **Resposta:** aprovado. No QLC+, o Speed Dial só muda os tempos comuns: num chaser "Por passo", ele não faz nada.
3. **A cue list do protótipo passa a usar o chaser de verdade já na F7:** play, stop, anterior, próximo e toque num passo. **Resposta:** aprovado. O resto da Cue List do QLC+ (os faders de crossfade manual) fica para a F11.
4. **Uma criação não pode conter a si mesma**, como no QLC+; as listas nem mostram as que fariam isso. **Resposta:** aprovado.
5. **No hold, "Padrão" vale como "Comum"**, como no QLC+, porque a cena não tem duração. **Resposta:** aprovado.
6. **Fim do Single Shot.** **Resposta:** uma opção em cada chaser (e sequência): apagar depois do último passo (o padrão, como no QLC+) ou manter o último passo aceso até alguém parar o chaser.

## Fica de fora

- **EFX e RGB Matrix** num passo ou numa coleção: o passo conta o tempo, mas eles só acendem na F8 e na F9.
- **Hold infinito (∞)**, para o chaser só andar no "próximo" da cue list: pode entrar na F11, com o resto da Cue List.
- **Intensidade por criação e submaster:** na F10.
- **Pausar** um chaser.
- **Tempo em batidas nas cenas:** o editor de cena só tem segundos (tela congelada). Se quiser, entra depois.
- **BPM vindo de fora** (MIDI clock ou áudio): fora do plano; pode entrar na F12, se você quiser.

## Testes automáticos

- **Ordem dos passos** (portados de `chaserrunner_test`): passos de 5 quadros em Loop, Single Shot e Ping Pong, para a frente e para trás, com a cena certa em cada quadro.
- **Random:** cada volta passa por todos os passos, sem repetir o mesmo passo seguido na virada.
- **Tempos** (portados de `currentFadeIn`, `currentFadeOut` e `currentDuration`): Padrão, Comum e Por passo, valendo para o que o passo dispara.
- **Crossfade:** os valores do passo que sai e do que entra, quadro a quadro; o dimmer aceso nos dois passos não pisca.
- **Próximo, anterior e ir para um passo** (portado de `nextPrevious`), também com o chaser parado.
- **Batidas:** a 120 BPM, o passo de 1 batida dura 25 quadros, por um minuto, sem deriva; o tap recomeça a batida; mudar o BPM com o chaser rodando.
- **Sequência:** cada passo com os seus valores; edição do passo atual rodando.
- **Coleção:** dispara e para tudo junto, com os fades de cada criação; coleção dentro de coleção; termina sozinha.
- **Edição com o chaser rodando:** o passo atual continua.
- **Single Shot:** com "apagar", termina e sai das criações rodando; com "manter o último passo", fica no último passo até ser parado. O "próximo" no último passo faz o mesmo.
- **Projeto:** um arquivo da versão 1 abre com "apagar" (a migração).
- **Servidor:** a tradução das criações rodando (passos, tempos e coleções), a recusa de uma criação que contém a si mesma e o elemento "Tempos" nos chasers.
- **Desempenho:** 4 universos cheios com vários chasers trocando de passo, abaixo de 2 ms por tick no app compilado.
- **Ponta a ponta:** o chaser troca a saída no tempo do passo, e o passo atual chega ao aparelho.
- `pnpm check` passando, e um teste no navegador (Studio e cue list).

## Validação (com você)

Antes de começar:
- **O `pnpm dev` já está com a F7:** ele recompilou sozinho e abriu a sua recuperação.
- **Monte um chaser de 8 passos no Rig RSena** com as cenas que você criou na F6 (ou crie mais).
- **"Fades da frente"** controla o "Troca de cores": com um valor no elemento, o chaser segue os tempos dele (em batidas do BPM); "Gravados" volta aos tempos do chaser.

Com o rig e a Numbstar:
- [ ] Chaser de 8 passos com crossfade de 2 s (fade in e fade out de 2 s), suave: o dimmer não pisca entre os passos. _Os Beams deslizando de uma posição para a outra: na [validação final](../validacao-final.md), com os moving beams._
- [ ] Loop, Ping Pong (sem repetir as pontas) e Random (sem repetir o mesmo passo seguido).
- [ ] Single Shot: sem a opção, apaga no fim e o LED do chaser apaga; com "Manter o último passo", o último fica aceso até você parar o chaser.
- [ ] "Para trás" começa pelo último passo.
- [ ] Tempos Padrão (os da cena), Comum e Por passo.
- [ ] Tap a 120 BPM: um chaser em batidas (1 batida por passo) acompanha o tap e mantém o tempo.
- [ ] Sequência: cada passo com os seus valores.
- [ ] Coleção: dispara e para tudo junto, cada criação com os seus fades.
- [ ] Cue list: play, stop, anterior, próximo e toque num passo; o passo atual igual no celular, no desktop e no Studio.
- [ ] Elemento "Tempos": o fade in, o fade out e a duração valem nos chasers da lista; "Gravados" volta aos tempos deles. Dá para conferir junto o item da F6 que ficou (os fades das cenas).
- [ ] "Seguir o BPM": o TAP do master (ou o do elemento) muda o BPM, os segundos embaixo dos knobs acompanham, e o chaser da lista anda nas batidas, também com 1/2 batida por passo. Com a chave desligada, o elemento volta aos segundos. _Ajuste pedido na validação._
- [ ] Editar um chaser rodando vale sem ele recomeçar.
- [ ] Stop All apaga com os fades; o pânico corta na hora.
- [ ] `pnpm check` passa.

## Referências estudadas

- `engine/src/{chaser,chaserrunner,chaserstep,sequence,collection,function,mastertimer}.cpp`
- `engine/test/chaserrunner`
- `ui/src/virtualconsole/vcspeeddial.cpp` (como o Speed Dial muda os tempos)

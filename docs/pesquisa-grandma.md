# Pesquisa: recursos da grandMA para profissionalizar o app

Pedido do usuário (2026-10-20): usar a grandMA, a base do mercado, como segunda referência (além do QLC+) e escolher os recursos que entram no projeto. Este arquivo é o **catálogo para a escolha**, explicado sem pressupor que se conhece a mesa. Depois que o usuário marcar os itens, o plano de cada um entra nas fases ainda não executadas (F9 a F16) e no `PLANO.md`.

Fonte: manual online da grandMA3 (MA Lighting), páginas da versão 2.4, lidas em 2026-10-20. O "no seu projeto" vem do código e da documentação do repositório nessa mesma data. Nada foi decidido ainda.

## Decisões do usuário (2026-10-21)

**Prioridade: a operação ao vivo.**

- **Entram no plano, nesta ordem de importância:** P4, R1, R2, R3, R7, E1, E2, E3 (só o fader de BPM, junto do tap), P6 (o Localizar com os outros mais fracos), P1, P2, E5, E6, S2 (só o GDTF) e o pedido novo **N1** (rig flexível, explicado abaixo).
- **Também entram, cada um na fase em que combina** (o usuário pediu todos, em 2026-10-21): P3, P5, P7 (só os filtros, sem os "Worlds"), P8, P9, R6, S2 (o MVR, além do GDTF), S4 e S6 (só o MIDI Show Control).
- **Fora por ora:** R4, R5, S5, S6 (timecode), S7, S8 (os "não recomendados"), **S1** (Art-Net e sACN: a saída é um `Device` do `rlc-dmx`, então entra depois sem mexer na estrutura), **E4** (já temos páginas), **E7** (linha de comando) e **S3** (multiusuário: a operação é de uma pessoa só). Do E3, os vários masters nomeados ficam fora; entra só o fader de BPM.

## N1 · Rig flexível: as criações não dependem de quantas fixtures há

**O problema (pedido do usuário):** as cenas, os EFX e o RGB Matrix foram montados para 8 ParLeds. Num show com 12, é preciso reconfigurar todas as criações antes de operar.

**Por que acontece:** hoje cada criação aponta para **fixtures uma a uma**: a cena guarda (fixture, canal, valor), o EFX guarda a lista de fixtures (o "Selecionar grupo" copia os membros na hora, não guarda o grupo) e o grupo de canais é uma lista fixa de canais. O RGB Matrix já aponta para um grupo de fixtures, e o desenho se ajusta ao tamanho da grade.

**Proposta (a mesma ideia dos "grupos" da grandMA):** o **grupo de fixtures** passa a ser a unidade das criações.
- **Cena por grupo:** além de fixtures soltas, a cena aceita um grupo como alvo ("ParLeds") e grava os valores uma vez. Vale para todas as fixtures que estão no grupo **agora**. Um valor de uma fixture específica continua possível e vence o do grupo (ex.: uma ParLed numa cor diferente).
- **EFX por grupo:** o EFX guarda o grupo, e não a cópia dos membros. A ordem vem da grade do grupo, e a defasagem se ajusta ao número de fixtures.
- **Grupo de canais dinâmico:** "o dimmer de todas as fixtures do grupo", em vez de uma lista fixa de canais.
- **RGB Matrix:** já funciona assim.
- **Resultado:** ir de 8 para 12 ParLeds é patchar as 4 e pô-las no grupo. Nada mais para editar. Para outro show, "Salvar como" e ajustar o patch e os grupos.

**Onde mexe:** o contrato (a cena e o EFX ganham o alvo "grupo", nova versão do projeto, com migração), o servidor (a tradução em `running_programs` e `scene_program` já resolve fixture em endereço DMX na hora de rodar, então expandir o grupo cabe ali) e os editores de cena e de EFX. O motor a 50 Hz não muda.

**Para decidir no mini-plano:** nas cenas existentes, as ParLeds costumam ter o mesmo valor ou valores diferentes por fixture? Se forem iguais, uma conversão automática das cenas atuais para "por grupo" é possível.

## Como ler

- **ID:** o código do item, para responder "quero P1, R2, S1…".
- **Custo** (estimativa para a arquitetura atual): **B** baixo, **M** médio, **A** alto.
- **Voto** (meu): ★ recomendo, ○ vale se o usuário quiser, ✕ não recomendo agora.
- Exemplos usam o rig do usuário: 12 ParLed 18x15w, 4 moving heads Beam 7R e o Strobo.

## Palavras que aparecem

- **Cue:** um "momento" da luz, como uma página de roteiro. É o equivalente do **passo** de um chaser daqui.
- **Cue list:** uma lista de cues que o operador avança, em geral com um botão **Go**.
- **Atributo:** um tipo de controle da fixture: intensidade (dimmer), cor, posição (pan e tilt), gobo, zoom, foco.
- **Programador:** o que o operador está mexendo agora e ainda não gravou. Aqui, é o editor de cena do Studio com a Prévia ligada (E12).
- **Fade:** subir ou descer a luz aos poucos. **Delay:** esperar antes de começar.
- **Executor:** o botão ou fader que dispara uma criação. Aqui, os elementos do console.
- **HTP e LTP:** quando duas criações mexem no mesmo canal, o maior valor ganha (HTP, usado na intensidade e nas cores) ou a última iniciada ganha (LTP, usado no resto).

## Como a grandMA pensa, e onde o projeto é diferente

- A grandMA é uma mesa de teclas e linha de comando, com um **programador global**: seleciona, mexe, grava. Aqui, o app é touch-first e programa no editor de cena. A ideia é adaptar o conceito, não copiar a mesa.
- Os valores gravados nas cues ficam **ligados a paletas** ("preset"): mudou a paleta, mudaram todas as cues.
- As sequências usam **tracking**: a cue só guarda o que mudou em relação à anterior. Aqui, cada passo aponta para uma cena completa, como no QLC+.

## O que o console faz de verdade hoje

Muita coisa do console existe como tela, mas o motor ainda não a usa. Pelo código (2026-10-20):

- **Funcionam:** botões (criação, blackout geral, parar tudo), moldura "solo", cue list (play, stop, anterior, próximo, toque no passo), elemento "Tempos" com o BPM e o tap, Grand Master, Blackout, Pânico.
- **Só a tela (F10 e F11):** sliders (nível, playback e submaster), XY pad, relógio, "Animation", o botão "blackout de fixtures", entradas MIDI. O servidor guarda o valor do elemento, mas nenhum código do motor o usa para acender.
- **RGB Matrix:** o editor existe, mas o motor ainda não acende (F9).

## Catálogo

### Programação: como se criam os looks

#### P1 · Paletas (presets) com referência · M · ★

- **O que faz:** uma paleta é uma "receita" com nome: uma cor ("Azul fundo"), uma posição dos Beams ("Centro da pista"), um gobo. Ao montar uma cena, em vez de gravar os números do canal, você aponta para a paleta. A cena guarda só a referência. A paleta "universal" vale para fixtures de modelos diferentes que tenham o atributo.
- **Vantagem:** o azul ficou escuro? Você corrige a paleta uma vez e as 40 cenas que a usam mudam juntas. O palco mudou de lugar? Corrige a posição "Centro" e todas as cenas acompanham.
- **No seu projeto:** não existe. Cada cena guarda os números de cada canal (fixture, canal, valor), então mudar uma cor exige editar cena por cena. "Paletas" está no Backlog v2.

#### P2 · Ordem de seleção e leque · B a M · ★

- **O que faz:** a ordem em que você tocou nas fixtures passa a contar. Com ela, o "leque" distribui um valor entre dois extremos ao longo da ordem: o primeiro Beam com pan em 20%, o último em 80%, e os do meio espalhados. Funciona também com intensidade (rampa) e cor (degradê).
- **Vantagem:** abrir os 4 Beams em leque ou fazer um degradê do vermelho ao azul nos 12 ParLeds num só gesto, sem calcular um por um. Inverter a ordem inverte o leque.
- **No seu projeto:** o editor de cena escolhe uma fixture ou várias do mesmo modelo (Ctrl + clique ou toque longo) e o valor mexido vale igual para todas. Não há ordem nem leque. A única ordem que existe é a da grade do grupo de fixtures, usada na propagação do EFX.

#### P3 · Divisão da seleção (o "MAtricks" da grandMA) · B a M · ○

- **O que faz:** ferramentas para dividir a seleção sem escolher fixture por fixture: "as duas metades", "asas" (de cada ponta para o centro), "uma por vez" (próxima e anterior), embaralhar a ordem de forma repetível e **espalhar tempo ou fase** (cada fixture começa um pouco depois da anterior).
- **Vantagem:** uma onda "dominó" nos 12 ParLeds (cada um acende 100 ms depois do vizinho) sem criar 12 cenas. Efeitos simétricos, de fora para dentro.
- **No seu projeto:** os grupos 2D de fixtures dão a grade, e o EFX já defasa os movimentos (juntas, em sequência ou defasadas). Para cenas, chasers e efeitos de cor, não há como dividir a seleção nem espalhar o tempo.

#### P4 · Efeitos por passos em qualquer atributo (os "phasers") · M (versão enxuta) a A (completa) · ★

- **O que faz:** um efeito em loop que mexe em **qualquer atributo**: onda senoidal no dimmer, troca de cor, círculo no pan e tilt. Você define a forma (seno, dente de serra, pulso), a velocidade (em BPM), a "fase" de cada fixture (o deslocamento dentro do ciclo) e toca com um botão. Cada passo tem largura, transição, aceleração e desaceleração.
- **Vantagem:** ondas e chases que seguem a música, em todas as fixtures, sem montar cena por cena. Um efeito só, reaproveitado em vários grupos.
- **No seu projeto:** o EFX mexe só no pan e no tilt (10 desenhos, absoluto ou relativo, defasagem entre fixtures). Efeito de dimmer ou cor hoje se faz com um chaser de várias cenas. O RGB Matrix (F9) fará efeitos de cor em grade 2D. Vale decidir antes da F9 se esses três (EFX, RGB Matrix e este) ficam separados ou compartilham uma base.

#### P5 · Gravar a partir da saída e "Update" · M · ○

- **O que faz:** "capturar" o que está saindo na luz agora (o resultado de tudo junto) e gravar numa cena. O "Update" grava um ajuste feito ao vivo de volta nas cues que estão rodando. Ao gravar sobre algo existente, escolhe-se: perguntar, sobrescrever, mesclar ou remover.
- **Vantagem:** monta-se o visual ao vivo, com faders e botões, e grava-se depois. Uma correção durante o ensaio entra na cue sem abrir editor.
- **No seu projeto:** a cena é editada no editor, e o que se muda nela rodando vale na hora. O "dump" do QLC+ (gravar a saída numa cena) está no Backlog. Atenção: a decisão E12 diz que o programador é o editor de cena, sem tela separada. Este item mexeria nisso.

#### P6 · Solo e Lowlight · B · ○

- **O que faz:** com uma fixture escolhida, o **Solo** apaga todas as outras e deixa só ela acesa, e o **Lowlight** deixa as outras num nível baixo. (A página que define os dois não foi lida; é o conceito conhecido da mesa.)
- **Vantagem:** achar qual Beam é o "3" e ver só o que você está ajustando, sem as outras atrapalhando.
- **No seu projeto:** o **Localizar** (F5) acende as fixtures escolhidas sem gravar na cena (dimmer e RGBW a 100%, pan e tilt parados). Ele não apaga as outras.

#### P7 · Filtros de atributo · B · ○

- **O que faz:** ao gravar ou aplicar, você diz quais tipos de atributo valem: "só a posição", "só a cor". O mesmo filtro pode limitar o que cada operador toca ("Worlds").
- **Vantagem:** atualizar só a posição dos Beams numa cena sem mexer nas cores; aplicar um look só na cor. Evita gravar um valor no lugar errado.
- **No seu projeto:** não existe. O editor grava os canais que você mexe, sem filtro. Os "Worlds" poderiam entrar junto com os papéis da F11.

#### P8 · Planilha de fixtures · M · ○

- **O que faz:** uma tabela com uma linha por fixture e uma coluna por atributo (dimmer, vermelho, verde, azul, pan, tilt…), com os valores ao vivo e edição direta.
- **Vantagem:** ver e comparar todas as fixtures de uma vez. Achar logo qual está diferente das outras.
- **No seu projeto:** a Mesa simples mostra os 512 canais por endereço DMX, sem agrupar por fixture. O Patch lista endereços. O editor de cena mostra uma fixture por vez.

#### P9 · ID numérico da fixture e seleção por faixa · B · ○

- **O que faz:** cada fixture ganha um número (1 a 16) e você as escolhe digitando "1 até 12". (As páginas lidas só citam "ID Types" e a palavra "Thru"; o resto é conhecimento geral da mesa.)
- **Vantagem:** seleção rápida pelo teclado. É a base de macros e de uma linha de comando.
- **No seu projeto:** a fixture tem nome e cor, e se escolhe por toque.

### Reprodução: cues, tempos e prioridades

#### R1 · Cue list profissional · B a M · ★

- **O que faz:** uma lista de cues que o operador avança com **Go**, como um roteiro: abertura, solo, refrão. Além do Go, há **voltar**, **ir para a cue N**, **pausar** e **carregar a próxima** sem disparar. Cada cue pode disparar de um jeito: manual (Go), sozinha X segundos depois da anterior, sozinha quando a anterior termina ("Follow") ou no ritmo do BPM. A lista pode ter uma cue final "Off" e voltar ao começo depois da última.
- **Vantagem:** show roteirizado, em que o operador só aperta Go, com cues que seguem sozinhas quando precisam. Pausar um fade no meio.
- **No seu projeto:** o chaser anda sozinho por tempo ou batidas (Loop, Uma vez, Vai e volta, Aleatória). A cue list do console tem play, stop, anterior, próximo e toque num passo. O que falta está no Backlog: **pausar**, **hold infinito** (o passo ficar esperando o Go) e BPM vindo de fora.

#### R2 · Tempos completos da cue · B · ★

- **O que faz:** além de subir, esperar e descer (que já existem), cada cue ganha **delay** (esperar antes de subir ou de descer) e **curvas**: o fade pode ser suave no começo e rápido no fim, em S, ou o contrário.
- **Vantagem:** o delay permite "os Beams só começam a subir 2 s depois dos ParLeds". A curva S parece mais natural ao olho do que o fade linear, que dá a impressão de acelerar no começo.
- **No seu projeto:** cada cena e cada passo tem fade in, hold e fade out, todos **lineares** (decisão da F6, igual ao QLC+). Não há delay nem curva. Adotar mudaria essa decisão.

#### R3 · Tempos por grupo de atributo · M · ★ (valida com os Beams)

- **O que faz:** na mesma cue, cada tipo de atributo tem o seu tempo: a posição leva 6 s para chegar, a cor 1 s e o dimmer 2 s.
- **Vantagem:** é o essencial para moving heads: mover devagar e acender depois, ou trocar a cor rápido enquanto o movimento é lento. Hoje isso exige duas cenas.
- **No seu projeto:** um fade in e um fade out por cena. O jeito de contornar hoje é separar a posição numa cena e a cor em outra, e rodar as duas juntas com fades diferentes. Rodas de cor, gobo e prisma mudam de uma vez, sem fade (regra da F6).

#### R4 · Tracking, "Cue Only", Shield e Release · A · ✕

- **O que faz:** com **tracking**, a cue 5 só guarda o que mudou em relação à 4. Se a cue 3 deixou o Beam 1 em azul e nenhuma outra mexeu nisso, ele continua azul nas cues seguintes. Editar o azul na cue 3 muda todas até alguém alterá-lo. O **Cue Only** grava só naquela cue. O **Shield** e o **Release** controlam até onde o valor se propaga.
- **Vantagem:** poucas gravações e a correção de um valor vale para toda a sequência. O preço é que o comportamento "de memória" confunde quem está começando.
- **No seu projeto:** é o oposto. Cada passo aponta para uma cena completa, então nenhum passo herda nada. É previsível, mas repete valores.
- **Por que não:** muda o modelo de dados inteiro. As **paletas (P1)** dão o principal benefício (corrigir uma vez) sem essa mudança.

#### R5 · Partes de cue · M · ✕

- **O que faz:** uma cue se divide em "partes" que disparam juntas, cada uma com tempo e transição próprios.
- **Vantagem:** numa cue só, o dimmer sobe rápido e a posição vai devagar.
- **No seu projeto:** não existe. O **R3** dá quase o mesmo resultado com muito menos.

#### R6 · Prioridade por criação · M · ○

- **O que faz:** cada criação tem um nível de prioridade, e a de nível maior ganha da menor quando disputam o mesmo canal, não importa quem começou por último. Do mais alto ao mais baixo: Super (ganha até do programador), Swap, HTP, Highest, High, LTP, Low e Lowest.
- **Vantagem:** uma cena "base", sempre ligada, nunca cobre um efeito. Uma cena de aviso ("Super") ganha de tudo.
- **No seu projeto:** na intensidade e nas cores, o maior valor ganha (HTP). Nos outros canais, a última criação iniciada ganha (LTP). Não há prioridade escolhida pelo usuário.

#### R7 · Ligar e desligar pelo fader · B · ★ (na F10)

- **O que faz:** o fader de uma cena a liga quando sai do zero e a desliga quando volta ao zero; no meio, define a intensidade. Também: desligar sozinha uma criação quando outra assume todos os canais dela ("Off when overridden").
- **Vantagem:** um fader por cena, como numa mesa analógica. Esquecer cena ligada deixa de acontecer.
- **No seu projeto:** o elemento Slider já tem o modo "Playback" (a criação roda com a intensidade do slider), mas só na tela: não acende nada ainda (F10). O "Parar tudo" já baixa os sliders de playback.

### Operação: executores, masters e atalhos

#### E1 · Funções dos botões · B (a maioria) a M (Swap) · ★

- **O que faz:** o botão do console pode ter vários comportamentos: **Go** (próxima cue), **Go−** (voltar), **Goto** (ir direto), **Pause**, **Flash** (acende enquanto segura), **Temp** (liga enquanto pressionado, com o fade da cue), **Toggle** (liga e desliga), **Kill** (corta tudo menos o que você escolheu) e **Swap** (liga este e apaga todos os outros).
- **Vantagem:** o operador "toca" a luz como um instrumento: flash no ritmo, volta de uma cue, troca rápida entre cenas.
- **No seu projeto:** o botão tem modo liga/desliga ou flash, e as ações: criação, blackout geral, blackout de fixtures e parar tudo. A moldura "solo" já faz o papel do Swap. Faltam Go−, Pause, Temp, Kill. O blackout de fixtures só existe na tela por enquanto.

#### E2 · Faders de executor: Master, Rate, Speed e Time · B a M · ★ (Master e Rate)

- **O que faz:** o **Master** é o nível da criação. O **Rate** é um fader que acelera ou desacelera todos os fades e delays em tempo real (por exemplo, de metade ao dobro da velocidade), **sem mudar o que está gravado**. O **Speed** ajusta a velocidade (BPM) de um efeito. O **Time** substitui o tempo gravado por um só.
- **Vantagem:** o operador ajusta a "pressa" do show conforme o público ou a música, sem editar nada.
- **No seu projeto:** o elemento **"Tempos"** já ajusta fade in, fade out e duração das criações da lista, em segundos ou em batidas do BPM (equivale ao Time e ao Speed). Falta o Rate (multiplicar em vez de substituir) e o master de cada criação, que é o Slider de Playback e Submaster da F10.

#### E3 · Masters de velocidade e de tempo compartilhados · M · ○

- **O que faz:** vários "botões de velocidade" (BPM) e de tempo, compartilhados. Ligue o chaser A e o efeito B ao Master de Velocidade 1 e os dois andam sincronizados e mudam juntos. O Master 2 pode ir mais devagar para outra parte do show. Também há masters de tempo, como "fade lento = 4 s", reutilizados por várias cues.
- **Vantagem:** partes do show em velocidades diferentes, cada parte sincronizada por dentro.
- **No seu projeto:** há **um** BPM master global, com tap. O elemento "Tempos" pode seguir esse BPM ou ter tempos próprios.

#### E4 · Páginas e configurações de executor · B a M · ○

- **O que faz:** o console tem várias páginas ("Abertura", "Banda", "Fim") e o que está ligado continua ligado ao trocar de página. Uma "configuração" guarda o jeito de um botão (tipo, cor, tamanho) para reaplicar em muitos.
- **Vantagem:** mais controles sem uma tela enorme; botões consistentes sem configurar um por um.
- **No seu projeto:** o console já tem páginas, e os elementos podem ser copiados e colados (Ctrl + C e Ctrl + V). Só falta a configuração reutilizável, que a cópia cobre em boa parte.

#### E5 · Mapa do palco clicável (o "Layout View") · M · ★ (na F14)

- **O que faz:** um desenho 2D do palco, visto de cima, com as fixtures e grupos. Você toca no desenho para escolher fixtures. (A página lida diz só que é um desenho 2D com objetos; usá-lo para selecionar é o uso conhecido da mesa.)
- **Vantagem:** "escolher os 3 Beams do lado esquerdo" tocando neles. Ideal no celular e no tablet.
- **No seu projeto:** a 3D View já guarda as posições de fixtures e elementos e tem vistas 3D, topo, frente e lateral, mas serve para posicionar. A seleção para programar é pela lista de fixtures do editor de cena. A visualização 2D está planejada na F14.

#### E6 · Macros · B a M · ○

- **O que faz:** uma "gravação" de ações que um botão executa em ordem, com espera entre elas: "apagar tudo, ligar a cena A, esperar 5 s, ligar a cena B".
- **Vantagem:** um toque para uma operação de vários passos, com menos risco de erro ao vivo.
- **No seu projeto:** o botão faz uma ação só (criação, blackout, parar tudo). A coleção dispara várias criações juntas, mas sem espera entre elas e sem ações como blackout.

#### E7 · Linha de comando · M · ○

- **O que faz:** digitar comandos, como "Fixture 1 Thru 12 At 50" (fixtures 1 a 12 em 50%). A forma geral é função mais objeto. (O exemplo é a sintaxe conhecida da mesa; as páginas lidas confirmaram só a forma geral e exemplos como `Delete Sequence 1`.)
- **Vantagem:** velocidade para quem opera pelo teclado. É a base natural de macros.
- **No seu projeto:** não existe. A interface é por toque e mouse, com atalhos só na edição do console e nas teclas master.

### Sistema, formatos e rede

#### S1 · Art-Net e sACN · B a M · ★ (promover da v2)

- **O que faz:** manda o DMX **pela rede** (cabo ou wifi) para "nodes" que o convertem em DMX físico perto das fixtures. É o padrão do mercado e também alimenta programas de visualização no computador.
- **Vantagem:** muito mais universos (hoje, cada interface USB é um universo), cabos longos sem perda, nodes baratos, e pré-programar com um visualizador (Capture, WYSIWYG etc.).
- **No seu projeto:** só USB: Enttec Pro, Open DMX (FTDI) e a saída Dummy. Art-Net e sACN estão fora da v1 (E5) e no Backlog v2. A arquitetura já ajuda: o motor publica um quadro por universo e cada saída tem a sua thread, então uma saída de rede seria "mais uma saída".

#### S2 · GDTF e MVR · M a A · ★ (GDTF) e ○ (MVR)

- **O que faz:** o **GDTF** é o formato aberto de descrição de fixture do mercado (o equivalente do `.qxf` do QLC+), publicado pelos fabricantes. O **MVR** é um arquivo de "planta" exportado por programas de desenho (Vectorworks e similares), com a posição e o patch das fixtures. (As páginas de GDTF e MVR não foram lidas; a descrição é o conhecimento geral dos formatos.)
- **Vantagem:** GDTF dá uma biblioteca maior e mantida pelos fabricantes. O MVR traz o rig inteiro, já posicionado e endereçado, sem digitar.
- **No seu projeto:** só importa `.qxf` do QLC+ (as 1.781 fixtures da 5.3.0 importam) e cria à mão. Não há GDTF nem MVR. Mudaria a decisão E3.

#### S3 · Multiusuário · M a A · ○

- **O que faz:** dois operadores ao mesmo tempo, cada um com a **sua** seleção e o **seu** programador, e uma saída só. Direitos por usuário, de acesso total até só mudar a vista.
- **Vantagem:** o operador de moving heads e o das cores trabalham juntos sem apagar a seleção um do outro.
- **No seu projeto:** vários aparelhos (PC, celular, tablet) ligados ao mesmo estado, e a última gravação vale (A2). A prévia é uma só para todos. O PIN e os papéis (admin, operador e visualizador) estão na F11.

#### S4 · Importação parcial de show · M · ○

- **O que faz:** traz só partes de outro show (as cenas, o patch ou o console de um show antigo) com regras para o que já existe.
- **Vantagem:** reaproveitar trabalho entre shows sem refazer.
- **No seu projeto:** "Abrir" troca o projeto inteiro. A biblioteca de fixtures é importada separadamente.

#### S5 · Backup e failover entre máquinas · A · ✕

- **O que faz:** duas máquinas na rede, e a reserva assume se a principal cair. (A página não foi lida.)
- **No seu projeto:** recuperação automática no PC e o `.bak` da versão anterior. Para um app de uma máquina, o custo não se paga agora.

#### S6 · Timecode e MIDI Show Control · A (timecode) a M (MSC) · ✕ timecode, ○ MSC

- **O que faz:** o **timecode** sincroniza as cues com o relógio de uma trilha de áudio ou vídeo, e o show se repete idêntico. O **MIDI Show Control** é um comando padrão de MIDI para outro equipamento disparar a luz.
- **No seu projeto:** show com timeline e áudio está fora da v1 (E5). O MIDI entra na F12 como entrada de controladores. O MSC caberia na F12.

#### S7 · Plugins em Lua · A · ✕

- **O que faz:** scripts em Lua que mexem no sistema por dentro. O manual avisa que os scripts podem quebrar entre versões e que a MA não dá suporte a eles.
- **No seu projeto:** Script está fora da v1 (E5).

#### S8 · Teste de DMX e RDM · B (teste) a A (RDM) · ✕

- **O que faz:** o teste de DMX acende canais sem programar. O **RDM** deixa o software descobrir e endereçar fixtures pelo próprio cabo DMX, e exige interface com suporte.
- **No seu projeto:** a página `/dev/dmx` (padrões de teste), a Mesa simples e o Localizar já cobrem o teste. RDM não faz sentido com as interfaces da v1.

## Já temos o equivalente

- **Blind / Preview:** a Prévia do editor de cena começa desligada e não afeta a luz.
- **Highlight:** o Localizar.
- **Web Remote da grandMA:** a UI servida na rede local, no celular e no tablet, sem o limite de 5 aparelhos.
- **3D Viewer, Grand Master, BPM com tap, desfazer e arquivo com migrações de versão.**

## Plano aprovado (2026-10-21)

Prioridade: a operação ao vivo. Os números das fases atuais não mudam (as novas levam uma letra) e só a ordem de execução muda.

| Ordem | Fase | Origem | O que entra |
|---|---|---|---|
| 0 | Ajustes rápidos | pedidos de 2026-10-21 | O Localizar com as outras fixtures mais fracas (P6). Fader de BPM na seção master, com o tap e o FINO (E3) |
| 1 | F10 Console I | original | Os sliders (nível, playback, submaster) e o blackout de fixtures passam a acender. Mais E1 (Go−, Pause, Temp, Kill), E2 (Master e Rate) e R7 (ligar e desligar pelo fader) |
| 2 | F10b Rig flexível | nova (N1) | Criações por grupo de fixtures |
| 3 | F10c Tempos, cue list e prioridade | nova | R1 (Go, voltar, ir para, pausar, gatilhos Time e Follow, hold infinito), R2 (delay e curvas), R3 (tempos por tipo de atributo) e R6 (prioridade por criação) |
| 4 | F9 RGB Matrix | original | Como no plano, já sobre grupos |
| 5 | F9b Efeitos por passos | nova | P4: efeito em qualquer atributo, com fase por fixture e BPM, reaproveitando a base do EFX. Não muda o desenho da F9 |
| 6 | F11 Console II | original | Mais E6 (macros) |
| 7 | F12 MIDI | original | Mais S6 (MIDI Show Control) |
| 8 | F12b Paletas e gravação | nova | P1 (paletas), P5 (gravar a partir da saída e Update) e P7 (filtros de atributo) |
| 9 | F12c Seleção | nova | P2 (ordem e leque), P3 (dividir a seleção e espalhar tempo ou fase) e P9 (ID numérico e seleção por faixa) |
| 10 | F13 Editor de fixtures | original | - |
| 11 | F13b Intercâmbio | nova | S2 (GDTF e MVR) e S4 (importação parcial de show) |
| 12 | F14 Visualização 2D | original | Mais E5 (mapa do palco clicável) e P8 (planilha de fixtures) |
| 13 | F15 3D e F16 release | original | - |

**Mudanças em relação ao plano original:**
- A F10 passa à frente da F9, porque o console é o que opera a luz ao vivo. A F9 continua antes da F11 (o elemento "Animation" da F11 controla um RGB Matrix).
- O que depende dos Beams (R3 e o leque do P2 e do P4 em pan e tilt) é testado nos ParLeds e na Numbstar o quanto der, e o resto vai para a [validação final](validacao-final.md).

## Limites da pesquisa

- Foram lidas só as páginas citadas abaixo. O que não estava nelas está marcado no texto.
- **Não lidas:** Agenda e timers, entradas remotas, MIDI fora do timecode, páginas de atributo, `network_session`, `rdm`, `dmx_priorities`, `patch_mvr`, `fixture_types`, `sc_psr`, `matricks_block/group/width/transform`, `presets_use`, `presets_create`, `masters_grand`, `executor_configurations`.
- A página de executores (`control_executor`) e a visão geral de teclas (`key_overview`) deram erro 404; usei `executor` e `executor_assign`.
- Duas páginas (`playback_masters`, `network`) e o Align mostram "Versão 2.3" dentro da pasta 2.4.
- As notas de lançamento desde a versão 1.0 não foram lidas por completo. Os destaques do 2.4 (receitas de phaser, formas, modo Preview, MSC, manuseio de presets, prioridades DMX, Highlight/Lowlight/Solo) vêm do índice de novidades do manual.
- A grandMA é proprietária e não tem código aberto. Por prudência (G3), o projeto reimplementa os conceitos com código e **nomes próprios em pt-BR**, sem copiar texto do manual nem imagens, e sem usar as marcas da MA ("MAtricks", "grandMA" etc.) na interface.

## Páginas lidas

Todas em `https://help.malighting.com/grandMA3/2.4/HTML/`: `help`, `presets`, `presets_pools`, `presets_edit`, `cue_tracking`, `cue_tracking_cue-only`, `cue_tracking_shield`, `cue_sequence`, `cue_sequence_sheet`, `cue_sequence_settings`, `cue_playback`, `cue_timing`, `cue_store_settings_preferences`, `cue_update`, `executor`, `executor_assign`, `executor_running_playbacks`, `masters`, `masters_speed`, `masters_playback`, `operate_programmer`, `operate_align`, `operate_selection`, `operate_fixture_sheet`, `fixture-sheet-modes`, `fixture-sheet-dmx-layer`, `group`, `matricks`, `matricks_wings`, `worldfilter`, `ws_ui_control_bar`, `phaser`, `phaser_editor`, `timecode`, `macros`, `layouts`, `command_syntax_keywords`, `plugins`, `patch`, `patch_3d_viewer`, `dmx`, `network`, `network_webremote`, `user`, `show_file_management`, `onpc`, `news`, `release-notes-2-4`, `key_releasenotes_2-3`.

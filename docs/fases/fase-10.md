# F10 — Virtual Console I

**Status:** mini-plano aprovado em 2026-10-21, com as quatro decisões como recomendado: as regras dos elementos no servidor; os dois modos (Flash e Temp), com os botões "flash" dos arquivos antigos virando Temp (nos seus projetos, nenhum botão estava em "flash"); o slider de playback como o próprio fade; o Rate como modo de slider, com a lista de criações. **Entregue e validada pelo usuário em 2026-10-21.**

**Entregue**
- **Motor (`rlc-engine`):**
  - intensidade por criação: multiplica só dimmer e cores de tudo o que ela acende (cena, passos de chaser, coleção) e muda ao vivo, sem recomeçar;
  - o fade de uma criação com a intensidade reduzida parte do valor cheio que dá o que está saindo: a luz não afunda quando ela entra no lugar de outra;
  - o blackout de fixtures, por cima do Grand Master, só na intensidade.
- **Servidor:**
  - a operação `pressWidget` (apertar e soltar), com as regras dos botões (liga/desliga, Flash, Temp, "Só esta", moldura solo, blackout, blackout de fixtures, parar tudo) e da cue list;
  - o servidor anota o que cada aparelho segura e solta quando ele sai;
  - um ping a cada segundo: o aparelho que fica 3 s sem responder (o Wi-Fi caiu) sai, e o que ele segurava é solto;
  - sliders: o playback (liga ao sair do zero, é a intensidade e o fade), o nível (os grupos de canais, sem fade), o submaster (moldura, molduras de dentro e página) e a velocidade (×0,25 a ×4, só nos tempos em segundos);
  - o pânico de dois toques zera os sliders de nível; o slider que troca de modo ou de criação solta o valor (e o playback aceso desliga a criação);
  - o arquivo do projeto na versão 7.
- **Telas:**
  - os três modos e a ação "Só esta" nas propriedades do botão;
  - o modo Velocidade, com a lista de criações, nas propriedades do slider;
  - a escala ×4, ×2, ×1, ½, ¼ e o display "×1,0" no slider de velocidade; o submaster sem valor em 100%;
  - os textos de ajuda dos modos.
- **Diferenças do plano** (detalhes que o plano não dizia):
  - **O ping de 3 s:** com o Wi-Fi caído, a conexão não fecha, só fica calada, e o TCP levaria dezenas de segundos para notar. O teste automático do plano fechava a conexão de um jeito limpo e não pegava esse caso.
  - O "Só esta" também aceita Flash e Temp: ao apertar, para as outras e liga esta; ao soltar, desliga esta.
  - Os botões da cue list e as teclas dos elementos também passam pelo servidor (`pressWidget`). O toque num passo da cue list e o TAP do "Tempos" continuam no aparelho.
- **Testes:**
  - automáticos:
    - no motor: a intensidade (dimmer e cor caem, o pan não; na coleção e no chaser; ao vivo), sem afundar na troca (o teste falha sem o ajuste), o blackout de fixtures;
    - no servidor: os botões (liga/desliga, solo, Temp, Flash a 100% sem fades e sem o submaster, Flash que não solta o que não acendeu, Temp com o submaster, "Só esta"), o blackout de fixtures, o parar tudo e o blackout em Temp; o slider de playback (fade, intensidade, zero, troca de modo); os submasters (moldura, página, moldura de dentro); o slider de nível e o pânico; a velocidade (×4, ×0,25, ×1, batidas sem mudar); a cue list; o que um aparelho segura, solto ao sair; a migração da versão 6;
    - de ponta a ponta: um aparelho que fecha a conexão segurando um Temp, e um que fica calado (sem fechar), soltos em menos de 6 s;
  - no navegador, contra o seu `pnpm dev` com o "Show de exemplo" (Numbstar desplugada):
    - o "Haze" antigo abriu como Temp e funcionou segurado; a "Âmbar" virou Flash e Temp pelas propriedades; um segundo aparelho fechado segurando o Temp soltou;
    - o slider de playback arrastado liga e, no zero, desliga; o blackout de fixtures apagou os PARs; o "Só esta" deixou só a "Troca de cores" e baixou o playback; o slider de velocidade mostrou ×1,0 e ×4,0 e o fade de 1 s durou 250 ms;
    - com a "Abertura" que você ligou durante o teste acendendo os PARs 1 a 6, o Flash solto, o fade do Temp com o submaster e o slider de nível foram conferidos no PAR 7, com elementos temporários (removidos no fim): Flash a 255 na hora e 0 ao soltar; Temp de 31 a 128 (submaster a 50%) e de 80 a 0; nível a 200 na hora;
  - `pnpm check` passa.
  - Depois do teste, o projeto aberto ficou igual ao de antes. O `settings.json` e os projetos da pasta não mudaram.
- **No seu "Show de exemplo":** ele ficou marcado com alterações não salvas (as minhas, já desfeitas e removidas; o conteúdo é o mesmo do arquivo), e por isso existe uma cópia de recuperação. O histórico do desfazer guarda os passos do teste: um Ctrl + Z traria de volta os elementos temporários. Abrir o projeto de novo pela lista (descartando) limpa os dois. Inclui os itens E1, E2 e R7 da pesquisa da grandMA ([pesquisa-grandma.md](../pesquisa-grandma.md)).

## Objetivo

Fazer o console operar a luz por inteiro, para o show ao vivo. Depois da F10:
- os **sliders** acendem de verdade:
  - o de **playback** liga a criação ao sair do zero, controla a intensidade e desliga ao voltar ao zero (R7);
  - o de **nível** mexe nos canais dos grupos de canais;
  - o **submaster** escala as criações da moldura;
- um slider novo de **velocidade** (o "Rate" da grandMA, E2) acelera ou desacelera os tempos das criações da lista sem mudar o gravado;
- os **botões** ganham o Flash de verdade (acende na hora), o Temp (com os fades) e o "Só esta" (E1);
- o botão de **blackout de fixtures** apaga as fixtures e grupos escolhidos;
- um **flash solta sozinho** se o aparelho perder a conexão com o dedo no botão.

## O que já existe

- **Console (F2 e ajustes de 2026-10-20):** grade com encaixe, edição e operação, páginas, molduras (com "solo"), propriedades por elemento, teclas por elemento, copiar e colar, e um layout para cada formato de tela.
- **Já acendem:** botões de criação (liga/desliga e o "flash" de hoje, que sobe e desce com os fades), blackout geral e "parar tudo"; a cue list (F7); o elemento "Tempos" (F6 e F7).
- **Só a tela, sem efeito na luz:** os sliders (nível, playback, submaster) e o botão de blackout de fixtures. O servidor guarda o valor, mas o motor não o usa.
- **Motor:** cada cena acesa tem um fader com o "dono" (a criação que o usuário disparou), o que já permite aplicar uma intensidade a tudo o que uma criação acende, inclusive nos passos de um chaser e numa coleção.
- **Onde ficam as regras hoje:** o que cada botão faz (liga, flash, solo da moldura) está na tela (`useConsole.ts`), que manda as operações ao servidor.

## O que entra

### 1. Intensidade por criação (motor)

- Cada criação rodando ganha uma **intensidade** (0 a 100%), que multiplica os canais de intensidade (dimmer e cores) de tudo o que ela acende: a cena, os passos do chaser, as criações da coleção. Pan, tilt, gobo e o resto não mudam, como no QLC+.
- A intensidade muda ao vivo, sem recomeçar a criação.
- Ela é o produto de tudo o que a controla: o slider de playback e os submasters das molduras.

### 2. Sliders

- **Playback (o "Master" da grandMA, R7):** sair do zero liga a criação; a intensidade segue o slider; voltar ao zero desliga. A criação aparece rodando no Studio, e o "Parar todas" e o pânico baixam o slider (isso já existe).
- **Nível:** os canais dos grupos de canais escolhidos, no valor do slider, sem fade. Na intensidade, mistura com as criações pelo maior valor (HTP). No resto (pan, gobo…), conta como uma criação ligada quando o slider saiu do zero. É como o QLC+ faz.
- **Submaster:** escala a intensidade das criações dos elementos da moldura (botões, sliders de playback e de nível, cue lists), também das molduras de dentro, multiplicando. Fora de uma moldura, vale para a página inteira. Sem valor, fica em 100%.
- **Velocidade (o "Rate" da grandMA, E2), modo novo:** tem uma lista de criações, como o elemento "Tempos". Vai de ×0,25 a ×4, com ×1 no meio, e o toque duplo volta a ×1. Multiplica a velocidade dos tempos em segundos (fade in, fade out, duração do passo, volta do EFX), também os do elemento "Tempos". Os tempos em batidas não mudam: neles manda o BPM.

### 3. Botões (E1)

- **Modos:**
  - **Liga/desliga** (como hoje);
  - **Flash** (novo): acende na hora, a 100% (sem o submaster), e apaga na hora ao soltar. Só solta o que ele mesmo acendeu: numa criação que já estava rodando, não faz nada;
  - **Temp:** liga enquanto pressionado, com os fades da criação e o submaster. É o "flash" de hoje.
- **Ação nova, "Só esta"** (o "Kill" da grandMA): liga esta criação e para todas as outras que estão rodando, cada uma com o seu fade out.
- **Blackout de fixtures:** passa a apagar a intensidade (dimmer e cores) das fixtures e dos grupos escolhidos, por cima de tudo, como o blackout geral.

### 4. As regras dos elementos no servidor

- O aparelho passa a mandar só "apertou" e "soltou" de cada elemento (e o valor dos sliders). O servidor aplica o que o botão faz: a ação, o modo, a moldura solo, o submaster.
- O servidor sabe qual aparelho está segurando cada flash ou Temp. Se a conexão cair, ele solta.
- É o mesmo caminho que o MIDI vai usar na F12, porque o MIDI chega no servidor, não na tela.
- O tap do BPM continua medido no aparelho, onde o dedo está.

### 5. Pânico

- O pânico de dois toques também zera os sliders de nível (como zera a Mesa simples). O de um toque só para as criações, como hoje.

### 6. Contrato, arquivo e protocolo

- Operação nova para apertar e soltar um elemento; o modo Temp nos botões, a ação "Só esta", o modo Velocidade nos sliders (com a lista de criações).
- Nova versão do arquivo do projeto, com a migração dos botões "flash" (ver a decisão 2).
- `docs/protocolo.md` e `docs/design-system.md` atualizados.

### 7. Telas (G2)

- Propriedades do botão: os três modos e a ação "Só esta".
- Propriedades do slider: o modo Velocidade, com a lista de criações.
- O slider de velocidade mostra "×1,0"; o submaster sem valor mostra 100%.
- Os textos de ajuda (ícone "i") dos modos.

## Decisões para você

1. **Onde ficam as regras dos elementos.**
   - **No servidor** (recomendado): é o que permite soltar o flash quando a conexão cai, e o MIDI da F12 usa o mesmo caminho.
   - **Na tela, como hoje:** o servidor só anota quem segurou cada flash. O MIDI vai precisar repetir as regras no servidor na F12.
2. **Os botões "flash" dos seus projetos.** Hoje o "flash" sobe e desce com os fades, o que passa a se chamar Temp.
   - **Viram Temp** (recomendado): nada muda no comportamento deles. Você troca para o Flash novo nos botões que quiser.
   - **Viram o Flash novo:** passam a acender e apagar na hora.
3. **Os fades no slider de playback.**
   - **O slider é o fade** (recomendado): ao sair do zero, a criação entra sem o fade gravado, e a intensidade segue a mão. Ao voltar ao zero, desliga sem fade (o slider já desceu).
   - **Como no QLC+:** ao sair do zero, a criação entra com o fade in gravado; ao voltar ao zero, sai com o fade out gravado.
4. **Onde fica o "Rate".**
   - **Um modo novo de slider, com a lista de criações** (recomendado): igual ao elemento "Tempos", cada show monta os seus.
   - **Um Rate geral na seção master:** um só, para todas as criações.

## Fica de fora

- **"Voltar" e "Pausar" nos botões (parte do E1):** dependem do motor pausar o chaser e o fade. Entram na F10c, com a cue list (R1).
- **Registro de elementos por schema** (estava no plano da F10): o painel de propriedades por tipo já existe e funciona; um registro genérico não paga a complexidade (G7).
- **Do QLC+:** o slider de Grand Master (a seção master já tem), o slider de outros atributos (largura de um EFX, cor de um RGB Matrix), o "monitor" do slider de nível, a intensidade inicial do botão e as molduras com páginas próprias.
- **MIDI:** F12. **XY pad, Animation e relógio:** F11.

## Testes automáticos

- **Motor:**
  - a intensidade escala só dimmer e cores, na cena, nos passos do chaser e na coleção, e muda ao vivo sem recomeçar;
  - o blackout de fixtures apaga só os canais de intensidade delas, por cima do Grand Master.
- **Servidor:**
  - botões: liga/desliga, Flash (sem fades, a 100%, não solta o que não acendeu), Temp (com os fades e o submaster), "Só esta", moldura solo, parar tudo, blackout e blackout de fixtures;
  - o flash e o Temp soltos quando o aparelho sai;
  - slider de playback: liga ao sair do zero, desliga no zero, a intensidade acompanha;
  - submaster: na moldura, nas molduras de dentro (multiplicando) e na página; sem valor, 100%;
  - slider de nível: HTP com as criações na intensidade;
  - velocidade: os tempos em segundos escalam, os em batidas não, também com o elemento "Tempos";
  - o pânico de dois toques zera os sliders de nível;
  - a migração dos botões "flash".
- **Ponta a ponta (WebSocket):** um flash segurado é solto quando a conexão fecha.
- **No navegador, contra o seu `pnpm dev`:** um console de duas páginas no desktop e no celular, com os sliders, os modos dos botões e as propriedades novas. Como sempre, com backup dos seus arquivos antes e comparação depois.
- `pnpm check` passando.

## Validação (com você)

Não precisa dos Beams (as ParLeds bastam).
- [x] Um console de 2 páginas montado no desktop e operado no celular.
- [x] Flash acende na hora e apaga ao soltar; também ao cair a conexão (desligar o Wi-Fi do celular com o dedo no botão).
- [x] Temp sobe e desce com os fades da cena.
- [x] "Só esta" para as outras criações e liga esta.
- [x] Submaster escala as cenas dos botões da moldura.
- [x] Slider de playback: sai do zero e liga, a intensidade acompanha, volta ao zero e desliga.
- [x] Slider de nível nos grupos de canais. _Na validação, não acendeu os ParLeds: o slider punha o dimmer deles em 54%, mas as cores estavam em 0, e a Mesa simples segurava o dimmer dos PARs (ela fica por cima do console). Funciona como planejado; o usuário concluiu a fase sem mudança._
- [x] Velocidade: acelera e desacelera os fades de uma cena e os passos de um chaser, sem mudar os gravados.
- [x] Blackout de fixtures apaga só as escolhidas.
- [x] `pnpm check` passa.

## Referências estudadas

- `qmlui/virtualconsole/vcslider.cpp`: os modos (Level, Adjust = o antigo Playback, Submaster, GrandMaster); o nível por canal sem fade, HTP na intensidade e LTP no resto; o playback que liga ao sair do zero (com o fade in gravado) e desliga no zero; o submaster que avisa a moldura.
- `qmlui/virtualconsole/vcbutton.cpp`: Toggle, Flash (só em cenas: acende sem fade e, ao soltar, sai com o fade out da cena), Blackout e StopAll; a intensidade aplicada na partida e ao vivo.
- `qmlui/virtualconsole/{vcframe,vcsoloframe,vcpage,vcwidget}.cpp`: o submaster que escala os outros elementos da moldura (e das molduras de dentro, multiplicando); a moldura solo, que só para botões e cue lists.
- `engine/src/{function,genericfader,fadechannel,collection,chaserrunner}.cpp`: a intensidade multiplica só os canais de intensidade; a coleção e o chaser passam a intensidade para o que disparam.
- `webaccess/`: o QLC+ **não** solta o flash quando o aparelho cai; aqui isso entra como melhoria.
- grandMA3 (manual 2.4, `executor_assign`): Flash, Temp, Kill e os faders de Master e Rate.

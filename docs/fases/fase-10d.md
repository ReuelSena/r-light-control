# F10d — EFX avançado

**Status:** mini-plano aprovado em 2026-10-22, com as quatro recomendações. Entregue em 2026-10-22, aguardando a validação (ver [Entregue](#entregue)), que leva junto a da F10c (pedido do usuário).

Pedidos do usuário na validação da F10b (G4): parâmetros do desenho por fixture ou grupo, desenho personalizado e curva de velocidade. Pedido da entrega da F10c: inverter o pan e o tilt nos EFX. Entra também a curva desenhada à mão, que a F10c deixou para cá (decisão 1 da F10c).

## Objetivo

Depois da F10d, um EFX só faz o que antes pedia vários:
- **cada item da lista** (fixture ou grupo) pode ter o **seu desenho**: os Beams da esquerda num círculo pequeno e os da direita num oito grande, no mesmo EFX;
- o desenho pode ser **feito à mão**, com o dedo ou o mouse, na área da prévia;
- a **velocidade varia ao longo da volta**: constante, acelerando, desacelerando, lenta nas pontas, ou numa curva desenhada;
- cada item pode **inverter o pan, o tilt** (ou os dois) e **trocar um eixo pelo outro**, para Beams de cabeça para baixo, de frente um para o outro ou montados de lado.

## O que já existe

- **EFX (F8):** os 10 desenhos do QLC+, com largura, altura, centro, rotação e os parâmetros do Lissajous; relativo; propagação (juntas, em sequência, defasadas); ordem (repetir, uma vez, vai e volta) e sentido; fade in, volta e fade out.
- **Itens da lista (F10b):** fixture ou grupo; cada item tem "Inverter" (o sentido do desenho), "Início" (graus) e, no grupo, a "Defasagem". A fixture do grupo que também está sozinha na lista é a exceção e usa os ajustes do item dela.
- **No motor (`efx.rs`):** cada fixture anda pelo ângulo da volta, e `point` faz o desenho, gira, escala e leva ao centro. O servidor (`efx_expand` em `apply.rs`) troca os grupos pelas fixtures deles, com os ajustes do item; a prévia da tela faz as mesmas contas (`utils/functions.ts`).
- **Curvas (F10c):** Linear, Lento no começo, Lento no fim e S, nos fades das cenas e dos passos.
- **O QLC+ não tem nada disso:** o `EFXFixture` só tem o sentido, o início e o modo (pan/tilt, dimmer ou RGB). O desenho é um só para todas as fixtures, não há desenho livre nem curva de velocidade, e a inversão do pan e do tilt não existe no EFX.

## O que entra

### 1. Desenho por item

- Os parâmetros do desenho (o desenho, a largura, a altura, o centro, a rotação, os do Lissajous, o desenho à mão e a curva de velocidade) valem para todos os itens.
- Cada item pode ligar o **"Desenho próprio"**: começa com uma cópia do desenho geral, e a partir daí o item tem o dele. Desligar volta ao geral (como o "Voltar ao grupo" da cena).
- **Ficam gerais:** o relativo, a propagação, a ordem, o sentido e os tempos (um EFX é uma criação só, com uma volta só).
- Num grupo, o desenho do item vale para todas as fixtures dele; a fixture que também está sozinha na lista continua sendo a exceção, com o desenho do item dela.

### 2. Desenho à mão

- Um desenho novo na lista, **"À mão"**. Com ele escolhido, o botão **"Desenhar"** da prévia deixa traçar o caminho na área dela, com o dedo ou o mouse; ao soltar, o fim se liga ao começo (o caminho é fechado, como os outros desenhos).
- O traço é simplificado para no máximo 64 pontos. As fixtures andam por ele com velocidade constante (pela distância percorrida), e a curva de velocidade vale por cima.
- A largura, a altura, o centro e a rotação valem também para o desenho à mão: ele pode ser reduzido, deslocado ou girado depois de feito.

### 3. Curva de velocidade

- Diz como a fixture anda ao longo da volta: **Constante** (como hoje), **Acelera**, **Desacelera**, **Lenta nas pontas** (S) ou **Desenhada**. As quatro primeiras são as curvas dos fades da F10c.
- A curva vale pela posição no desenho: todas as fixtures ficam lentas nos mesmos pontos do caminho, cada uma na vez dela (como carros numa pista, que freiam nas mesmas curvas).
- **Desenhada:** um quadrado em que se traça a curva (o tempo na horizontal, o quanto do desenho já foi andado na vertical). A curva só sobe, começa em 0 e termina na volta inteira; o traço é ajustado para isso e guardado em 16 pontos.
- A duração da volta não muda: a curva só redistribui a velocidade dentro dela.

### 4. Inverter e trocar os eixos

- Cada item ganha **"Inverter pan"**, **"Inverter tilt"** e **"Trocar pan e tilt"**. Num grupo, valem para todas as fixtures dele.
- O espelho é do movimento, em volta do centro do desenho (decisão 2): o centro continua onde foi posto, e o desenho sai espelhado naquele eixo. No relativo, o centro é a posição de baixo, então é o mesmo.
- O "Inverter" de hoje, que é o sentido do desenho, passa a se chamar **"Ao contrário"**, para não confundir com os novos (só o texto muda).

### 5. Contrato, arquivo e protocolo

- Os parâmetros do desenho viram um bloco só (`EfxShape`), gravado no arquivo do mesmo jeito de hoje no EFX; o item ganha o bloco opcional dele, a inversão e a troca dos eixos.
- O desenho ganha o "À mão" com os pontos, e a curva de velocidade com os pontos da desenhada.
- Nova versão do arquivo do projeto (10): a migração só acrescenta os campos novos (sem desenho próprio, sem inverter, velocidade constante). Tudo abre como hoje.
- O protocolo não ganha operações: o EFX continua sendo gravado inteiro (`saveFunction`). `docs/protocolo.md` atualizado com o contrato.

### 6. Telas (G2)

- **Editor de EFX:**
  - na lista, cada item ganha o "Desenho próprio" e os botões "Inverter pan", "Inverter tilt" e "Trocar pan e tilt", no padrão dos que já estão lá;
  - tocar num item com desenho próprio o escolhe (aceso), e os knobs e a prévia passam a mostrar e mexer no desenho dele; tocar de novo, ou no "Geral" acima dos knobs, volta ao geral;
  - a "Velocidade" (a curva) fica junto do desenho, com o editor da curva desenhada quando ela é a escolhida;
  - a prévia mostra os caminhos de todos os itens (o de cada item na cor das fixtures dele, quando é próprio) e as fixtures andando com a inversão e a curva; com "À mão", o botão "Desenhar".
- **3D View:** nada muda; ela mostra o que sai do motor.

### 7. Exemplos no show de exemplo (G8)

No `exemplo.json` e no "Show de exemplo" da sua pasta, numa pasta de EFX "EFX avançado", com os Beams:
- **Dois desenhos:** os dois Beams da esquerda num círculo pequeno e os dois da direita num oito grande;
- **Espelhados:** um círculo em que os Beams da direita têm o pan invertido;
- **À mão:** uma estrela desenhada;
- **Varredura em S:** uma varredura do pan com a velocidade "Lenta nas pontas", e outra com uma curva desenhada que para no meio.

## Decisões (aprovadas em 2026-10-22, as quatro recomendações)

1. **O desenho próprio do item é um bloco inteiro.** Ao ligar, o item copia o desenho geral e passa a ter todos os parâmetros dele; mudar o geral depois não mexe no item.
   - _Alternativa:_ cada parâmetro herdado à parte (o item troca só o tamanho, e o resto acompanha o geral). Dá mais controle, mas cada knob precisaria do "herdado/próprio", e a tela fica bem mais pesada.
   - **Recomendo o bloco inteiro** (G7): é o que o exemplo pede (desenhos diferentes por lado) com uma chave só.
2. **Inverter espelha o movimento em volta do centro do desenho.** Um círculo com o centro no pan 200 continua centrado no 200, girando para o outro lado.
   - _Alternativa:_ espelhar o curso inteiro do pan (255 − valor), como se a fixture estivesse montada ao contrário: o centro também iria para o outro lado (200 → 55).
   - **Recomendo o espelho do movimento:** o centro é ajustado olhando o palco (e agora pode ser por item), e o que se quer inverter é o sentido do desenho naquele eixo. O curso inteiro invertido é uma propriedade da fixture, para tudo (cenas, Mesa simples), e caberia no patch, se você pedir um dia.
3. **A curva desenhada à mão fica só na velocidade do EFX.** Os fades das cenas e dos passos continuam com as quatro curvas da F10c.
   - **Recomendo assim:** nos fades curtos, as quatro curvas cobrem quase tudo, e a desenhada nos fades levaria o editor da curva para a cena, para os tempos por atributo e para os passos. Entra depois, se você pedir.
4. **Trocar pan e tilt entra** (a fixture montada de lado): uma chave por item, sem custo no motor. **Recomendo incluir.**

## Fica de fora

- Os modos Dimmer e RGB do EFX e o "controle do dimmer" do QLC+ 5.3 (Backlog da F8).
- EFX em fixtures com várias cabeças de pan e tilt (continua a primeira cabeça).
- Relativo, propagação, ordem, sentido e tempos por item.
- Delay e curva dos fades no EFX (o EFX continua só com o fade in e o fade out; no passo do chaser, valem os impostos, como na F10c).
- Inverter o pan e o tilt na fixture (no patch), para tudo.

## Testes automáticos

- **Motor:**
  - dois itens com desenhos diferentes andam cada um no seu;
  - inverter o pan, o tilt e trocar os eixos dão o desenho espelhado em volta do centro, também no relativo e com o fino;
  - o desenho à mão: as fixtures passam pelos pontos traçados, com velocidade constante pela distância;
  - cada curva de velocidade nos pontos de 25%, 50% e 75% da volta; a Constante igual à de hoje; a curva não muda a duração da volta e, com a propagação, todas ficam lentas nos mesmos pontos;
  - os testes do EFX de hoje continuam passando (os números do QLC+).
- **Servidor:**
  - o grupo passa o desenho, a inversão e a troca do item para as fixtures dele, e a fixture sozinha continua sendo a exceção;
  - a conferência do EFX salvo: até 64 pontos no desenho à mão, a curva desenhada só subindo, de 0 à volta inteira;
  - a migração da versão 9.
- **No navegador, contra o seu `pnpm dev`:** o editor (desenho próprio, escolher o item, inverter, desenhar à mão, a curva desenhada) e a prévia batendo com a saída. Como sempre, com backup antes e comparação depois.
- `pnpm check` passando.

## Validação (com você)

Numa rodada só, a da F10d e a da F10c (pedido do usuário, 2026-10-22), abrindo o **Show de exemplo**.

**F10d**, na pasta **EFX avançado** do Studio, olhando a prévia do EFX e a 3D View (os Beams de verdade ficam para a validação final):
- [ ] **Dois desenhos:** os Beams da esquerda num círculo pequeno e os da direita num oito grande.
- [ ] **Espelhados:** os Beams da direita giram ao contrário dos da esquerda, no mesmo centro.
- [ ] **À mão:** desenhar outro caminho e ver as fixtures o percorrerem.
- [ ] **Varredura em S:** a varredura freia nas duas pontas; na **Curva desenhada (para no meio)**, os Beams param no meio do caminho e seguem.
- [ ] No editor de um EFX: ligar o "Desenho próprio" num item e mudar o desenho dele; inverter o pan de outro; desenhar à mão (botão "Desenhar") e traçar uma curva de velocidade.

**F10c**, com os ParLeds, na página **Tempos** do console e na pasta **Tempos e prioridade** do Studio (o checklist completo está em [fase-10c.md](fase-10c.md#validação-com-você)):
- [ ] **Roteiro manual (Go):** cada passo espera o Go (hold ∞); Voltar volta; o toque longo num passo o marca em amarelo, e o Go vai para ele.
- [ ] Pausar no meio do passo 2 congela a luz; o Go (ou o Pausar) continua de onde parou.
- [ ] **Delay e curva S:** espera 1 s e sobe em S em 3 s; ao parar, fica 2 s acesa e desce em 3 s.
- [ ] **Cor no escuro:** a cor muda na hora e o dimmer sobe depois de 2 s; ao parar, o dimmer desce em 3 s e a cor sai depois.
- [ ] **Base branca (baixa)** e **Azul a 30% (alta)**: a Alta assume, mesmo disparando a Baixa de novo; parar a Alta volta ao branco com o fade.
- [ ] Os botões Voltar e Pausar da página Tempos.

**As duas:**
- [ ] Os projetos antigos abrem como antes.
- [ ] `pnpm check` passa.
- Na [validação final](../validacao-final.md): os EFX desta fase com os Beams de verdade, e a posição lenta com a cor rápida da F10c.

## Entregue

- **Contrato:** o desenho virou o `EfxShape` (gravado no mesmo lugar de antes, no EFX), com o desenho à mão (`freehand` e `path`) e a velocidade (`speed` e `speedPoints`); o item ganhou `shape`, `invertPan`, `invertTilt` e `swapAxes`. Projeto na versão 10, com a migração.
- **Motor (`efx.rs`):** cada fixture usa o desenho do item ou o geral; a velocidade muda a posição no desenho depois da propagação, do sentido e do início (a conta das curvas é a mesma dos fades, `ease` em `lib.rs`); o desenho à mão anda pela distância; o espelho e a troca dos eixos são em volta do centro.
- **Servidor:** o grupo passa o desenho, o espelho e a troca às fixtures dele (já vinha do `..x.clone()` do `efx_expand`); o `check_efx` confere o geral e o de cada item.
- **Telas:** no editor, o "Desenho de" (Geral ou um item com desenho próprio), "À mão" com o botão "Desenhar", a "Velocidade" com o editor da curva desenhada (`StudioEfxCurve.vue`) e, em cada item, "Desenho próprio", "Ao contrário" (o antigo "Inverter"), "Inverter pan", "Inverter tilt" e "Trocar pan e tilt". A prévia mostra um caminho por aparência (o geral, cada desenho próprio e cada espelho) e recebe o traço do desenho à mão.
- **Exemplos (G8):** pasta "EFX avançado" no `exemplo.json` e na recuperação do seu "Show de exemplo" (também no arquivo salvo): "Dois desenhos", "Espelhados", "À mão (estrela)", "Varredura em S" e "Curva desenhada (para no meio)", com os Beams.
- **Testes:** 3 novos no motor (desenho por item e espelho, também no relativo; desenho à mão; curvas de velocidade e a propagação), 1 no servidor (o grupo passa os ajustes, e a exceção fica com os dela), a conferência do EFX salvo e a migração da versão 9. No navegador, contra o `pnpm dev` com o "Show de exemplo": 21 conferências (os exemplos na prévia, o desenho próprio, o espelho, a troca, o desenho à mão, a curva em S e a desenhada), num EFX temporário dos MHs do u1, removido no fim; o projeto voltou igual.

### Diferenças do plano

- **Ao desenhar à mão, o tamanho, o centro e a rotação voltam ao inteiro** (127, 127, 127 e 0), para o desenho ficar onde foi traçado; depois podem ser ajustados.
- **O exemplo "Lenta nas pontas" é uma varredura** (Linha 2 em "Vai e volta"), e não uma "Linha": a curva vale para a volta inteira, e a "Linha" vai e volta numa volta só, então frearia só numa das pontas.

## Referências estudadas

- Do projeto: o motor do EFX (`crates/rlc-engine/src/efx.rs`: `place`, `point`, `reverse` e `scale`), as curvas da F10c (`fade` em `crates/rlc-engine/src/lib.rs`), a tradução no servidor (`efx_expand` e `check_efx` em `apply.rs`), o editor e a prévia (`StudioEfx.vue`, `StudioEfxPreview.vue`) e as contas da tela (`efxPoint`, `efxFixtures` e `efxPositions` em `utils/functions.ts`).
- QLC+ 5.3.0 (só leitura): `engine/src/efx.cpp` e `efxfixture.cpp`. O item do EFX só tem o sentido (`m_direction`), o início (`m_startOffset`) e o modo; o desenho é do EFX inteiro; não há desenho livre, curva de velocidade nem inversão de eixo. Tudo aqui é próprio do projeto, sem código do QLC+ para portar.
- grandMA3: os itens da [pesquisa](../pesquisa-grandma.md) não cobrem o EFX de perto; a ideia do desenho por item vem da "exceção" da cena (F10b).

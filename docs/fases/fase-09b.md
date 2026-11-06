# F9b — Efeitos por passos (Phaser)

**Status:** mini-plano aprovado em 2026-10-27, com as sete recomendações. Entregue no mesmo dia (ver [Entregue](#entregue)), aguardando a validação, que vai junto com a da F9, a da F10c e a da F10d (o usuário pediu para seguir antes de validar: exceção ao G1, como na F9).

## Objetivo

Um efeito em loop que mexe em **qualquer atributo** de um grupo de fixtures: uma onda no dimmer, uma troca de cor, um balanço no pan e no tilt. Escolhe-se a **forma** da onda (seno, triângulo, dente de serra, pulso), a **velocidade** (em segundos ou em batidas do BPM master) e a **defasagem** de uma fixture para a próxima. Um efeito só serve a todas as fixtures do grupo, sem uma cena por fixture (P4 da [pesquisa da grandMA](../pesquisa-grandma.md)).

É o que no grandMA se chama _phaser_. Aqui entra na versão enxuta do catálogo ("M"): ondas calculadas, e não uma lista de passos com valores gravados (decisão 1).

## O que já existe

- **EFX (`efx.rs`):** o molde. Um _runner_ por criação, a volta contada em `turns`, a defasagem entre fixtures, os faders que entram a partir do que está embaixo e saem no fade out, o relativo (somado ao que está embaixo, zero em 127), a pausa e o elemento "Tempos" mandando na volta, também em batidas. Só mexe no pan e no tilt.
- **RGB Matrix (`matrix.rs`):** o molde das cabeças. As células do grupo (`matrix_cells`) já ligam cada cabeça aos canais de um atributo, com o canal fino em 16 bits, e um fader só leva todos os canais.
- **Grupos de fixtures (N1, F10b):** a criação aponta para o grupo, e o servidor o expande na hora de rodar, na ordem da grade (linha por linha).
- **Mistura (F6, F10c):** HTP no dimmer e nas cores, LTP no resto, prioridade, pausa, slider e submaster por criação. Uma criação nova na árvore `Program` herda tudo isso sem mexer na mistura.
- **Tempos (F7, F10c):** `Span` em ms ou em batidas, o BPM master e o `Times` com fade in e fade out.

## O que entra

### 1. Contrato e arquivo

- **Tipo novo de criação, `phaser`** (`FunctionType::Phaser`, `FunctionKind::Phaser(Phaser)`), com ícone próprio no Studio, como os outros.
- **`Phaser`:**
  - `group`: o grupo de fixtures (como o RGB Matrix; sem grupo, não acende);
  - `layers`: de 1 a 8 **camadas**, cada uma com **um atributo**;
  - `spread`: a defasagem em graus de uma fixture para a próxima, na ordem da grade, de −360 a 360 (0 = todas juntas; com 12 fixtures e 30°, a onda dá uma volta no grupo; negativo = a onda anda para o outro lado);
  - `beats`: o ciclo em batidas do BPM (como o chaser); `timing`: `hold_ms` é a duração do ciclo (ms, ou milésimos de batida), mais o fade in e o fade out.
- **`PhaserLayer`:**
  - `attribute`: dimmer, as cores (vermelho, verde, azul, branco, âmbar, UV, lima, índigo, ciano, magenta, amarelo), pan, tilt, zoom, foco, íris e rotação do gobo (os atributos contínuos; rodas, gobo, prisma, macros e strobo não);
  - `shape`: `sine`, `triangle`, `sawUp`, `sawDown` ou `pulse`; `width` (a parte do ciclo em que o pulso fica no alto, de 1 a 99%; só no pulso);
  - `low` e `high`: o piso e o teto da onda, de 0 a 255 (a escala DMX, como o centro do EFX);
  - `phase`: graus de atraso da camada em relação ao ciclo (0 a 359), somados à defasagem das fixtures;
  - `relative`: a onda é somada ao que está embaixo, como no EFX relativo (o zero em 127; `low` e `high` passam a valer como deslocamentos). Serve para balançar em volta de uma posição ou de um nível já gravado.
- **Projeto na versão 12:** o tipo é novo, então nada antigo muda; a migração só sobe o número (como as outras, A5).
- **Protocolo:** `saveFunction` já grava a criação inteira, então não há operação nova. Tipos gerados, `docs/protocolo.md` e os textos do `FunctionType` atualizados.

### 2. Motor (`phaser.rs`, programa novo na árvore `Program`)

- **O servidor traduz** (`phaser_program` em `apply.rs`): para cada camada, as fixtures do grupo que têm o atributo (na ordem da grade, cada fixture uma vez), com o endereço DMX do canal e o fino, se houver. Em fixtures de várias cabeças, vale a cabeça da célula do grupo, como no RGB Matrix; o pan e o tilt, a primeira cabeça de cada fixture, como no EFX.
- **O motor anda o ciclo** como o EFX: `turns` sobe `TICK / ciclo` a cada quadro. Mudar o ciclo (no editor, no "Tempos" ou no BPM, com o ciclo em batidas) continua do ponto onde está, sem salto. A fase de cada canal é `turns + (phase + i × spread) / 360`, onde `i` conta só as fixtures que têm o atributo da camada.
- **A forma** vira um valor de `low` a `high`: seno (de `low` a `high` e volta, começando em `low`), triângulo, dente de serra para cima e para baixo e o pulso com a largura. Em 16 bits quando há o canal fino.
- **Entrar e sair, como as cenas e o EFX:** as camadas absolutas ficam num fader só e as relativas em outro (no máximo dois). No fade in, os canais saem do que está embaixo e entram na onda; ao parar, voltam no fade out ao que está embaixo (decisão 2 da F6).
- **Mistura:** vale a de sempre. Dimmer e cores em HTP, o resto em LTP, com a prioridade, o slider de playback, o submaster e a pausa da criação. Pausada, o ciclo e os fades congelam.
- **Dentro de um chaser ou de uma coleção**, como o EFX e o RGB Matrix (os tempos que o passo impõe valem para os fades).
- **Mudar a criação rodando** (forma, piso, teto, fase, defasagem, ciclo) vale na hora, sem recomeçar o ciclo. Trocar o grupo, o atributo de uma camada ou o relativo recomeça os faders (a luz que estava sai no fade out).
- **Elemento "Tempos":** vale para o phaser da lista: fade in, fade out e a duração do ciclo (em batidas, seguindo o master), como no EFX.

### 3. Telas (G2)

- **Studio:** o tipo "Phaser" (ícone e texto no `RlInfo`) entra na lista, na criação, nas pastas e na busca, como os outros. Não muda nada no console (os botões, os sliders e o "Tempos" aceitam o tipo novo na lista de criações).
- **Editor do Phaser (`StudioPhaser.vue`):**
  - o grupo (e um aviso quando não há);
  - a lista de camadas, cada uma com o atributo, a forma, o piso e o teto (em %, ou em ± % no relativo), a largura do pulso, a fase e o "Relativo"; "Adicionar camada" e "Remover";
  - a defasagem entre as fixtures e o ciclo (segundos ou batidas), com o fade in e o fade out;
  - a **prévia animada**: uma linha por camada com a onda de cada fixture do grupo (as barras de intensidade que sobem e descem), calculada na tela com as mesmas contas do motor, no mesmo molde do EFX (`utils/functions.ts`, decisão 3).

### 4. Exemplos no show de exemplo (G8)

No `exemplo.json` e no "Show de exemplo" da sua pasta, numa pasta **"Phaser"**:
- **Onda de dimmer nos ParLeds:** seno no dimmer, com a defasagem entre as 12 ParLeds (a onda anda pelo grupo), o ciclo em 4 batidas (segue o TAP);
- **Pulso nos ParLeds:** pulso curto no dimmer, defasagem de 30°, como uma corrida de luz;
- **Arco-íris nos ParLeds:** três camadas, vermelho, verde e azul em seno, defasadas 120° uma da outra (a cor gira), com o dimmer de uma cena de base;
- **Balanço nos Beams (pan e tilt):** o pan e o tilt em seno, defasados 90° (o círculo), em relativo, em volta da posição de uma cena. Fica para a [validação final](../validacao-final.md) com os Beams.

## Decisões a aprovar

1. **Ondas calculadas (versão enxuta), e não uma lista de passos com valores gravados.**
   - _Alternativa:_ a versão completa do grandMA, com cada passo guardando os valores de cada fixture, largura, transição, aceleração e desaceleração. É o "A" do catálogo: um editor de passos por fixture, parecido com o de cena, muito maior.
   - **Recomendo as ondas** (G7). Cobrem os exemplos do catálogo (a onda de dimmer, a troca de cor, o círculo); a aceleração fica por conta da curva de velocidade, que já existe no EFX e pode entrar aqui depois se você pedir. O "passo" de verdade já existe no chaser.
2. **O círculo não é uma forma: são duas camadas** (pan e tilt em seno, defasados 90°), como o EFX faz. O mesmo vale para o arco-íris (três camadas defasadas 120°).
   - _Alternativa:_ uma forma "Círculo" e uma "Cor" que misturam dois atributos numa camada só. Mais atalhos na tela, mais casos no motor.
3. **A prévia é calculada na tela** (as mesmas contas do motor, como o EFX), e não pedida ao servidor como no RGB Matrix.
   - O motivo do RGB Matrix era o texto, a imagem e o JavaScript, impossíveis de repetir. A onda são quinze linhas, e a prévia fica fluida (um pedido por quadro seria demais).
4. **Um grupo de fixtures por Phaser** (como o RGB Matrix), e não uma lista de fixtures e grupos como o EFX. Para dois grupos com ondas diferentes, uma coleção de dois Phasers.
   - _Alternativa:_ a lista do EFX, com o "Inverter" e o "Início" por item e a exceção da fixture sozinha: muito mais telas para um uso raro.
5. **Os atributos são contínuos e soltos** (dimmer, cada cor, pan, tilt, zoom, foco, íris e rotação do gobo). Sem rodas, gobo, prisma, macros e strobo.
   - _Alternativa:_ uma camada "Cor" com duas cores (degradê entre elas, saindo em RGB, CMY ou cinza como o RGB Matrix). É o que a "troca de cor" do catálogo pede de forma mais simples, mas repete a conversão de cor do RGB Matrix. Entra depois se você preferir.
6. **O ciclo em batidas é do próprio editor** (a chave "Em batidas", como o chaser), seguindo o BPM master, o TAP e o fader. No EFX isso ficou no Backlog; aqui é parte do pedido ("velocidade em BPM, seguindo o master").
7. **Nome do tipo: "Phaser"** (o termo da grandMA, que você já usa na pesquisa).
   - _Alternativas:_ "Efeito" (confunde com a aba "Efeitos" dos atributos e com o EFX) ou "Onda". Mudar o nome depois é só texto.

## Fica de fora

- Os valores gravados por passo, a largura, a transição e a aceleração por passo da grandMA (decisão 1).
- A forma "Círculo" e a camada "Cor" (decisões 2 e 5).
- A divisão do grupo ("asas", "uma por vez", embaralhar) e a defasagem por seleção: entram na F12c (P3), e a defasagem do Phaser já fica pronta para elas.
- Rodas de cor, gobo, prisma, macros e strobo como atributo de uma camada.
- Fixtures de várias cabeças com um atributo por cabeça no pan e no tilt (vale a primeira cabeça de cada fixture, como no EFX).
- A curva de velocidade (acelera, desacelera) e a ordem Vai e volta: o seno e o triângulo já cobrem o balanço.
- A **Animation** do console (F11): o elemento que controla o Phaser com sliders ao vivo.

## Testes automáticos

- **Motor (`rlc-engine`):**
  - as formas: seno, triângulo, dente de serra para cima e para baixo e o pulso com a largura, do piso ao teto, em 8 e em 16 bits;
  - a defasagem: com `spread` 0 as fixtures andam juntas; com 360/n, uma onda completa pelo grupo; negativo vai ao contrário; a fase da camada soma;
  - o ciclo: mudar a duração e o BPM não dá salto; o ciclo em batidas segue o TAP;
  - entrar e sair: o fade in sai do que está embaixo e o fade out volta a ele; a prioridade, o HTP, o slider e a pausa; a camada relativa soma ao que está embaixo (zero em 127, clamp nas pontas);
  - editar rodando vale na hora; trocar o atributo recomeça;
  - dentro de um passo de chaser e de uma coleção.
- **Servidor:** a tradução do grupo em canais por camada (a ordem da grade; fixtures sem o atributo ficam de fora e não contam na fase; o fino do pan e do tilt); a conferência do Phaser salvo (grupo que existe, de 1 a 8 camadas, atributo contínuo, piso e teto, largura, fase e defasagem nos limites); remover um grupo tira o grupo do Phaser, como no RGB Matrix; o exemplo abre; a migração da versão 11.
- **Na tela, contra o seu `pnpm dev`:** o editor (as camadas, o relativo, as chaves) e a prévia batendo com o motor. Com backup antes e comparação depois.
- `pnpm check` passando.

## Validação (com você)

Abrindo o **Show de exemplo**, na pasta **Phaser** do Studio. As ParLeds estão no universo da Numbstar (sem ela conectada, veja na 3D View). A onda e o pulso mexem só no dimmer: ligue junto a cena **Cor dos ParLeds (sem dimmer)**. O arco-íris mexe nas cores: ligue junto a **Base dos ParLeds** (o dimmer).
- [ ] **Onda de dimmer nos ParLeds:** a luz anda de uma ParLed para a seguinte; mudar a defasagem para 0 as põe juntas, e para negativo inverte o sentido; a prévia do editor mostra o mesmo.
- [ ] **O TAP do BPM** e o fader de BPM mudam a velocidade da onda (ciclo em batidas); trocar para segundos no editor volta ao tempo fixo; o elemento **Tempos** muda o ciclo.
- [ ] **Pulso nos ParLeds:** a largura do pulso muda quantas ParLeds ficam acesas ao mesmo tempo.
- [ ] **Arco-íris nos ParLeds:** a cor gira pelo grupo; mudar a fase de uma camada muda a cor.
- [ ] **Entrar e sair:** o fade in e o fade out suavizam a entrada e a saída da onda, e a cena de base volta intacta; a pausa congela a onda.
- [ ] **Prioridade e mistura:** o Phaser sobre uma cena de prioridade maior não a cobre; com a mesma, o dimmer mistura pelo maior valor.
- [ ] **Dentro de um chaser:** um passo com o Phaser anda no tempo do passo.
- [ ] Os projetos antigos abrem (migração); `pnpm check` passa.
- [ ] **Balanço nos MHs (pan e tilt)**, com a cena **MHs no centro** ligada, nos 4 MHs genéricos (3D View): o pan e o tilt balançam ±8% em volta do centro, defasados 90° (um círculo pequeno).
- Na [validação final](../validacao-final.md), com os Beams: o mesmo balanço nos Beams de verdade (o grupo "Moving heads" tem os 4 Beams do rig: com a Numbstar ligada, eles se mexem).

## Entregue

- **Contrato (`rlc-core`):** `Phaser` (grupo, camadas, defasagem, ciclo em batidas e tempos), `PhaserLayer` (atributo, forma, largura, piso, teto, fase e relativa) e `WaveShape`; `FunctionType::Phaser` e `FunctionKind::Phaser`. Projeto na versão 12 (só sobe o número).
- **Motor (`phaser.rs`):** `PhaserRunner` na árvore `Program`, como o EFX: `turns` conta o ciclo (em ms ou em batidas, sem salto ao mudar), cada canal vai ao ponto da onda, as camadas absolutas e as relativas em um fader cada. Entra e sai pela regra das cenas, com a prioridade, o slider, o submaster e a pausa de sempre; vale dentro de chasers e coleções (a entrada do passo conta o fade in dele) e no elemento "Tempos".
- **Servidor (`apply.rs`):** `phaser_targets` (o canal do atributo em cada cabeça do grupo, na ordem da grade, cada canal uma vez) e `check_phaser`; `head_channels` e `attribute_level` saíram do `matrix_cells`, que os divide com o Phaser; remover o grupo deixa o Phaser sem grupo.
- **Telas:** o tipo "Phaser" no Studio (ícone, lista, criação, pastas), o editor `StudioPhaser.vue` (grupo, defasagem, camadas com atributo, forma, piso, teto, largura do pulso, fase e relativa, ciclo em segundos ou batidas e fades) e a prévia `StudioPhaserPreview.vue` (uma barra por fixture em cada camada, com as contas do motor em `utils/functions.ts`; o relógio fica nela, à parte do editor, como no EFX e no RGB Matrix).
- **Exemplos (G8):** pasta "Phaser" no `exemplo.json` e no "Show de exemplo" do usuário (a recuperação, pelo app aberto, e o arquivo salvo): "Onda de dimmer nos ParLeds", "Pulso nos ParLeds", "Arco-íris nos ParLeds" e "Balanço nos MHs (pan e tilt)", mais a cena "Cor dos ParLeds (sem dimmer)".
- **Testes:** 10 no motor (as formas, a defasagem, a fase, os atributos que as fixtures não têm, o ciclo em batidas e a mudança sem salto, entrar e sair, o relativo, a pausa, a edição rodando e o chaser e a coleção) e 5 no servidor (a tradução em canais na ordem da grade, o atributo que as fixtures não têm, a conferência ao salvar, remover o grupo e a migração da versão 11). Contra o servidor ao vivo (sem aparelho de verdade): a onda andando pelas 12 ParLeds, o pulso correndo, o arco-íris girando a cor e o balanço oscilando de 108 a 147 no pan e no tilt, em quadratura. No navegador: o editor, a prévia, criar pelo diálogo "Nova", a camada relativa e o aviso de grupo sem o atributo, sem erros na página.

### Diferenças do plano

- **Piso abaixo do teto:** o servidor recusa o piso acima do teto (o plano não dizia). A tela mantém os dois em ordem ao digitar.
- **Uma camada por atributo e tipo:** o servidor recusa o mesmo atributo repetido na mesma camada de tipo (duas ondas no mesmo canal brigariam). Uma absoluta e uma relativa podem dividir o atributo.
- **O pulso e o dente de serra para baixo partem do teto**, e o seno, o triângulo e o dente de serra para cima, do piso (o plano dizia que todos partem do piso).
- **A defasagem positiva atrasa a fixture seguinte** (a onda anda da primeira à última).
- **O relativo, na tela, em ±%** (de -50% a +50% do curso), e ligá-lo começa em ±8%.
- **O exemplo do balanço usa o grupo "Moving heads"** (os 4 MHs genéricos e os 4 Beams do rig), e se chama "Balanço nos MHs", não "nos Beams".

## Referências a estudar na implementação

- Do projeto: `efx.rs` (o ciclo, a defasagem, os faders e o relativo), `matrix.rs` (as células e o fader único), `running_programs` e `program` em `apply.rs`, `matrix_cells` e `pan_tilt` (os canais por atributo), o contrato (`Efx`, `RgbMatrix`, `Attribute`) e os editores `StudioEfx.vue` e `StudioRgbMatrix.vue`.
- grandMA3 (só leitura, a pesquisa): as páginas dos _phasers_ não foram lidas na pesquisa (está nos "Limites" do [catálogo](../pesquisa-grandma.md)); o conceito vem do P4. Se você quiser que a forma e os parâmetros sigam o manual de perto, leio as páginas antes de implementar.
- O QLC+ não tem nada parecido (o EFX só faz pan e tilt, e o "dimmer" e o RGB do EFX 5.3 ficaram fora na F8): nada a portar, e a atribuição do `NOTICE` não muda.

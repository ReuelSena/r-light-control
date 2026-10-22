# F10b — Rig flexível

**Status:** mini-plano aprovado em 2026-10-21. Decisão 1: os projetos salvos são de teste e podem ser convertidos à vontade; por isso a migração só acrescenta os campos novos (sem código de conversão, G7), e as criações do show de exemplo e do "RSena Palco" são passadas para os grupos uma vez, na entrega. Decisão 2: congelar. **Entregue e validada pelo usuário em 2026-10-21.**

**Entregue**
- **Contrato e arquivo:** a cena com `groups` (o grupo de fixtures e os valores por tipo de canal), o item do EFX como fixture ou grupo, o grupo de canais com `fixtureGroup` e `attributes`; projeto na versão 8 (a migração só acrescenta os campos; o EFX antigo já é uma fixture).
- **Servidor:**
  - a cena acende os valores do grupo em quem está nele na hora, com a exceção da fixture por cima, e o grupo de depois vence; o mesmo na Prévia;
  - o EFX com grupos: a ordem da grade, só as fixtures com pan ou tilt, cada fixture uma vez, e o "Inverter" e o "Início" do item para todas;
  - o grupo de canais que acompanha um grupo de fixtures refaz os canais quando o grupo, as fixtures ou as definições mudam (a Mesa simples, os sliders e as cenas seguem);
  - mudar um grupo muda na hora as criações rodando e a Prévia;
  - remover um grupo de fixtures congela onde ele era usado (cena, EFX, grupo de canais e botão de blackout de fixtures), sem mudar a luz.
- **Telas:**
  - editor de cena: as ações (Adicionar fixtures, Localizar, Voltar ao grupo) numa linha e, abaixo, os grupos da cena e as fixtures juntos num painel afundado, acesos quando escolhidos (ajustes pedidos na validação); tocar no grupo mexe nele, com as abas de sempre (a primeira fixture do grupo é o modelo dos canais); as fixtures do grupo aparecem na fileira, e mexer numa delas cria a exceção só nos canais tocados (marcada com *, na fixture e no canal); "Voltar ao grupo" tira as exceções; o "Selecionar grupo" põe ou tira o grupo;
  - grupo de canais (Fixtures): "Canais escolhidos" ou um grupo de fixtures, com os atributos;
  - editor de EFX: o grupo como item da lista, com o número de fixtures, e a prévia com as fixtures dele;
  - **defasagem no grupo do EFX (pedido na validação):** o item de grupo ganha "Defasagem (°)": cada fixture começa esse tanto da volta depois da anterior, na ordem da grade (0° = juntas). Os outros pedidos do EFX (parâmetros por item, desenho personalizado, curva de velocidade) foram para a F10d;
  - o passo novo da sequência copia a cena com os grupos aplicados.
- **Diferenças do plano** (detalhes que o plano não dizia, decididos na implementação):
  - **"Selecionar grupo" numa cena que já tem as fixtures do grupo:** o grupo entra com o valor mais comum de cada tipo de canal entre elas, e cada fixture fica só com o que difere (a exceção): a luz não muda. Foi assim que os seus projetos foram convertidos, pela própria tela.
  - **A exceção nasce de mexer na fixture**, e não de tocá-la em "Adicionar fixtures": só os canais tocados viram exceção, e o resto continua seguindo o grupo. Na lista de "Adicionar fixtures", a fixture do grupo mostra o ícone de grade, e tocar nela tira as exceções.
  - **Grupo de canais que acompanha um grupo:** a lista "Canais por fixture" some (os canais são os atributos escolhidos); o número de canais continua no topo.
  - **Grupo de canais de lista fixa numa cena por grupo:** o fader dele grava nas fixtures (vira exceção), como antes.
- **Exceção no EFX (pedido na validação):** uma fixture do grupo incluída também sozinha na lista fica no lugar dela no grupo, com o "Inverter" e o "Início" do item dela (antes, valia o primeiro lugar em que ela aparecia, e o item dela era ignorado).
- **Correção da validação:** o "Início" e a "Defasagem" do EFX voltavam a 0 ao digitar (o problema vinha da F8): a prévia animada redesenhava o editor a cada quadro. Ela foi para um componente próprio, com o relógio dentro. Conferido no navegador digitando devagar e com as setas, num EFX temporário.
- **Seus projetos** (de teste, liberados por você), convertidos pela tela, conferindo que cada cena acende igual antes e depois, e salvos:
  - "RSena Palco": as 7 cenas de cor no grupo "ParLeds 18x15w" (a ParLed 1 ficou como exceção na "Blue", 252, e na "White", 250); os 9 grupos de canais acompanham o grupo; o "EFX 1" com o "Grupo 11";
  - "Show de exemplo": o arquivo salvo ainda era o de 17/10, e as suas mudanças estavam só na cópia de recuperação (a "Frente vermelha" com as ParLeds, um botão novo, o chaser e a sequência). Ela foi a base da conversão, então nada se perdeu. Ganhou os grupos "Moving heads", "ParLeds" e "Beams"; as cenas da frente estão no "PARs frente" (a "Frente vermelha" também no "ParLeds"), a "MHs no centro" e o EFX no "Moving heads", e 5 grupos de canais acompanham os grupos. A versão anterior ficou no `.rlc.bak`.
- **Testes:**
  - automáticos: a cena do grupo (quem entra no grupo acende, a exceção vence, o grupo de depois vence, grupo repetido ou inexistente recusado), os dois canais do mesmo tipo (os prismas e o pan fino do Beam), o grupo de canais que acompanha (com a Mesa simples), o EFX com grupos (ordem da grade, sem repetir, o "Inverter" do grupo), remover um grupo congela (a luz não muda), a migração da versão 7; de ponta a ponta, a cena do grupo editada rodando;
  - no navegador, contra o `pnpm dev` com o "Show de exemplo" (Numbstar desplugada), 24 conferências: "Selecionar grupo" sem mudar a luz, a macro de cor no grupo, a exceção no PAR 2 e o "Voltar ao grupo", o PAR 7 entrando e saindo do grupo com a cena rodando, "RGB PARs" acompanhando o grupo e o fader dele gravando no grupo da cena, o EFX com o grupo (a prévia com 4 e, sem o MH 2, com 3) e remover o grupo pela tela sem mudar a luz;
  - `pnpm check` passa.

Pedido do usuário (N1, 2026-10-21): as cenas, os EFX e o RGB Matrix foram montados para 8 ParLeds; num show com 12, é preciso reconfigurar todas as criações antes de operar. Detalhes em [pesquisa-grandma.md](../pesquisa-grandma.md) (N1).

## Objetivo

O **grupo de fixtures** passa a ser a unidade das criações, como os grupos da grandMA. Depois da F10b:
- uma cena grava os valores **para o grupo** ("ParLeds"), e eles valem para quem está no grupo **agora**;
- um valor de uma fixture específica continua possível e vence o do grupo (uma ParLed numa cor diferente);
- o EFX pode ter um grupo na lista, em vez da cópia dos membros;
- um grupo de canais pode ser "o dimmer (ou o vermelho…) do grupo ParLeds", e acompanha o grupo na Mesa simples, nos sliders do console e nas cenas.

**Resultado:** ir de 8 para 12 ParLeds é patchar as 4 novas e pô-las no grupo. Nenhuma criação precisa ser editada.

## O que já existe

- **Cena:** guarda (fixture, canal, valor) para cada fixture. O "Selecionar grupo" do editor copia os membros do grupo na hora; quem entra no grupo depois não entra na cena.
- **Grupo de canais:** uma lista fixa de canais. O atalho "Por atributo" inclui o atributo de todas as fixtures, mas também na hora.
- **EFX:** a lista de fixtures. O "Selecionar grupo" também copia os membros.
- **RGB Matrix:** já aponta para um grupo de fixtures (o motor dele vem na F9).
- **Blackout de fixtures (F10):** já aponta para os grupos e acompanha quem está neles.
- **No seu "RSena Palco"** (lido hoje, sem mexer):
  - 12 ParLed 18x15w no grupo "ParLeds 18x15w" e 2 Beams no "Grupo 11";
  - 9 grupos de canais, cada um com um atributo de todas as ParLeds (Dimmer, Vermelho, Verde, Azul, Branco, Âmbar, UV, Strobo e Macro de cor);
  - 7 cenas de cor com as 12 ParLeds (a cor na cena, o dimmer em 0, vindo do slider "Dimmer ParLeds"):
    - em 5 delas, as 12 têm os mesmos valores;
    - na "Blue" e na "White", a ParLed 1 está com 252 e 250 e as outras com 255. Parece ter sido sem querer;
  - o "EFX 1" nos 2 Beams, na ordem do "Grupo 11".

Ou seja: o seu projeto já é montado por grupo. Ele só não acompanha o grupo quando ele muda.

## O que entra

### 1. Cena por grupo

- Na cena, o **grupo entra como alvo**, ao lado das fixtures soltas. Os valores do grupo são gravados **por tipo de canal** (dimmer, vermelho, pan, gobo…), não pela posição do canal: assim valem para qualquer fixture do grupo.
  - Quando a fixture tem dois canais do mesmo tipo (os dois "Prisma" do Beam 7R), vale a ordem: o primeiro prisma do grupo vai para o primeiro prisma de cada fixture.
- Ao rodar, o servidor aplica os valores do grupo em cada fixture que está nele naquele momento.
- **Exceção:** uma fixture do grupo com valores próprios na cena. O valor próprio vence o do grupo, canal a canal.
- Dois grupos na mesma cena com a mesma fixture (ex.: "ParLeds" e "ParLeds esquerda"): vence o que vem depois na lista da cena; a exceção vence os dois.
- Mudar o grupo (incluir, tirar, trocar a ordem) muda na hora as cenas rodando e a Prévia.

### 2. Grupo de canais que acompanha o grupo de fixtures

- Além da lista fixa de hoje, o grupo de canais pode ser **"os atributos X do grupo Y"**. Exemplo: "Dimmer ParLeds" = o dimmer do grupo "ParLeds 18x15w".
- O servidor refaz a lista de canais quando o grupo de fixtures, o patch ou as fixtures mudam. Assim, a Mesa simples, os sliders de nível do console e as cenas acompanham sem nenhuma mudança neles.
- Como no atalho "Por atributo" de hoje, entram só os canais grossos (sem os finos).
- Numa cena, ligar um grupo de canais desses grava no grupo de fixtures (o item 1). Assim, o seu jeito de montar as cenas de cor continua igual, e passa a acompanhar o grupo.

### 3. EFX por grupo

- Um item da lista do EFX pode ser um **grupo**. As fixtures dele entram na ordem da grade (linha por linha), só as que têm pan ou tilt, como no "Selecionar grupo" de hoje.
- O "Inverter" e o "Início" do item valem para todas as fixtures do grupo. Para espelhar (metade para cada lado), monta-se dois grupos ("Beams esquerda" e "Beams direita") e inverte-se um.
- A defasagem ("Defasadas", "Em sequência") conta as fixtures que estão nos grupos agora: com 4 Beams, 90°; com 6, 60°.

### 4. Remover um grupo de fixtures usado em criações

- Ver a decisão 2.

### 5. Contrato, arquivo e protocolo

- A cena ganha a lista de grupos com os valores de cada um; o item do EFX pode ser um grupo; o grupo de canais pode apontar para um grupo de fixtures e uma lista de atributos.
- Nova versão do arquivo do projeto (8), com a migração (decisão 1). Sem operações novas: as de hoje (`saveFunction`, `saveChannelGroup`) levam os campos novos.
- `docs/protocolo.md` atualizado.
- O motor a 50 Hz não muda: o servidor já traduz as criações em endereços DMX na hora de rodar, e a expansão do grupo cabe ali.

### 6. Telas (G2)

Com os componentes que já existem, sem tela nova.
- **Editor de cena:**
  - "Adicionar fixtures" → "Selecionar grupo" passa a pôr **o grupo** na cena, e não a cópia dos membros.
  - Na fileira de fixtures, os grupos aparecem antes, com o ícone de grade. Tocar no grupo mostra as mesmas abas de hoje (Canais, Intensidade, Cor, Posição, Efeitos), com os canais da primeira fixture do grupo como modelo, e o que se mexe vale para o grupo.
  - As fixtures com exceção aparecem depois, como hoje. Na aba Canais, o canal com valor próprio fica marcado. Um botão "Voltar ao grupo" tira as exceções da fixture escolhida.
  - Na lista de "Adicionar fixtures", a fixture coberta por um grupo da cena mostra o grupo. Tocar nela cria a exceção (começa com os valores do grupo, então a luz não muda) ou a tira.
- **Grupo de canais (Fixtures):** no topo, a escolha entre "Canais escolhidos" (como hoje) e um grupo de fixtures. Com um grupo, os botões "Por atributo" escolhem os atributos, e a lista por fixture só mostra os canais que entraram.
- **Editor de EFX:** o "Selecionar grupo" põe o grupo na lista, como um item com o ícone de grade e o número de fixtures. A prévia desenha cada fixture do grupo.
- **Sequência:** ao criar um passo, ele copia os valores da cena, já com os grupos aplicados (ver "Fica de fora").

## Decisões para você

1. **Os projetos que já existem** (o "RSena Palco" e o show de exemplo).
   - **Converter ao abrir** (recomendado). A luz fica exatamente igual, e as criações passam a acompanhar os grupos:
     - a cena com todas as fixtures de um grupo vira "cena do grupo". O que é igual em todas vai para o grupo; o que difere fica como exceção. No seu projeto, as 7 cenas de cor viram cenas do grupo "ParLeds 18x15w". A ParLed 1 da "Blue" e da "White" fica como exceção (252 e 250); se foi sem querer, o "Voltar ao grupo" resolve em um toque;
     - o grupo de canais que é exatamente "um atributo de todas as fixtures de um grupo" passa a acompanhar o grupo. No seu projeto, os 9;
     - o EFX cuja lista é exatamente um grupo, na ordem da grade e com os mesmos ajustes, passa a ter o grupo. No seu projeto, o "EFX 1" fica com o "Grupo 11".
   - **Não converter:** o que existe continua por fixture. Para passar uma cena ao grupo, é preciso refazê-la.
2. **Remover um grupo de fixtures usado em criações.**
   - **Congelar** (recomendado): os valores do grupo passam para as fixtures que estavam nele, o item do EFX vira a lista delas e o grupo de canais vira uma lista fixa. A luz não muda.
   - **Tirar das criações:** a cena perde os valores do grupo e o EFX perde o item. O desfazer traz de volta.

## Fica de fora

- **Sequências:** os passos continuam guardando os valores por fixture. Ao criar um passo, ele copia a cena já com os grupos aplicados, mas o passo não acompanha o grupo depois. Se você usar sequências em show, fazemos depois.
- **Grupo misturado (ParLed e Beam juntos):** o dimmer, as cores, o pan e o tilt valem para todos. Já a roda de cor, o gobo e as macros têm números diferentes em cada modelo, então o mesmo valor pode dar resultados diferentes. As paletas da F12b resolvem isso.
- **Cabeças de uma fixture** (os pixels de uma barra): o grupo vale para a fixture inteira, sem separar as cabeças. Nenhuma fixture do seu rig tem várias cabeças.
- **O leque e a ordem de seleção** (P2 e P3): F12c.

## Testes automáticos

- **Servidor:**
  - a cena do grupo acende todas as fixtures dele; a exceção vence o grupo, canal a canal; dos dois grupos, vence o que vem depois;
  - dois canais do mesmo tipo (os prismas do Beam) recebem cada um o seu valor;
  - incluir uma fixture no grupo com a cena rodando a acende na hora, e tirar a apaga; o mesmo na Prévia;
  - o grupo de canais que acompanha: a lista de canais refeita ao mudar o grupo, as fixtures e o patch; o slider de nível e a Mesa simples seguem;
  - o EFX com um grupo: a ordem da grade, só as fixtures com pan ou tilt e a defasagem pelo número atual;
  - remover um grupo usado em criações (pela decisão 2);
  - a migração, com um projeto igual ao seu (12 ParLeds com uma exceção, os 9 grupos de canais e o EFX), que acende os mesmos valores antes e depois.
- **No navegador, contra o seu `pnpm dev`:** o editor de cena com um grupo, as exceções e o "Voltar ao grupo", o grupo de canais por grupo de fixtures e o EFX com um grupo. Como sempre, com backup dos seus arquivos antes, elementos temporários e comparação depois.
- `pnpm check` passando.

## Validação (com você)

Não precisa de fixtures novas: tirar ParLeds do grupo simula um show com menos.
- [x] O "RSena Palco" abre, e as cenas de cor e o slider "Dimmer ParLeds" acendem igual a antes.
- [x] Tirar 4 ParLeds do grupo "ParLeds 18x15w": as cenas de cor, o slider e a Mesa simples passam a acender só 8. Pôr de volta: as 12, sem editar nenhuma criação.
- [x] Numa cena do grupo, uma ParLed numa cor diferente (a exceção) vence o grupo, e o "Voltar ao grupo" a devolve.
- [x] O EFX num grupo: a "Defasagem" separa as fixtures do grupo (na prévia do editor, as bolinhas se afastam); tirar um Beam do grupo e o EFX seguir só com o outro, na 3D View. _Validado depois de três ajustes: a defasagem, a correção dos campos que voltavam a 0 e a fixture sozinha como exceção do grupo. Com os Beams de verdade, na validação final._ Com os Beams de verdade, na [validação final](../validacao-final.md). _Na validação (2026-10-21): com a propagação "Juntas", as fixtures do grupo fazem o mesmo movimento, sem dinamismo. O usuário pediu a defasagem entre as fixtures do grupo e outros recursos de EFX (ver o registro do DECISOES)._
- [x] Remover um grupo usado numa cena não apaga a luz dela (decisão 2: congela). Teste num grupo de teste, para não desfazer a conversão (ou desfaça com Ctrl + Z).
- [ ] `pnpm check` passa.

## Referências estudadas

- O QLC+ não tem esse conceito: as cenas dele guardam os valores por fixture, e o grupo de fixtures só serve ao RGB Matrix e à seleção.
- grandMA3 (na pesquisa de 2026-10-20): as cues e os presets guardam os valores por fixture, mas o grupo é a forma de escolher, e os "recipes" ligam um grupo a valores e se refazem quando o grupo muda. A F10b leva a ideia do recipe para a cena, sem a linha de comando.
- Do projeto: `scene_program`, `running_programs`, `preview_levels` e `check_efx` em `crates/rlc-server/src/apply.rs`; o editor de cena (`StudioScene.vue`), o de EFX (`StudioEfx.vue`), o de grupo de canais (`FixturesChannelGroup.vue`) e o passo da sequência (`StudioChaser.vue`).

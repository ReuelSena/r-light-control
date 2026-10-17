# F5 — Patch, projeto e programador

**Status:** mini-plano aprovado em 2026-10-17, com as decisões como recomendado, menos a 2: sem projeto anterior, o app abre um projeto em branco. O show de exemplo fica gravado na pasta dos projetos ("Show de exemplo"), para abrir pela lista. Entregue em 2026-10-17, aguardando a validação do usuário.

**Entregue**
- **Projetos:**
  - salvar, "Salvar como" e abrir pela lista, na pasta `Documentos\R Light Control\` (no seu PC, `R:\Documentos\R Light Control\`);
  - `.rlc.bak` da versão anterior;
  - arquivo inválido ou de versão mais nova recusado sem mexer no projeto aberto.
- **Ao iniciar:** a recuperação; senão, o último projeto; senão, um projeto em branco. O "Show de exemplo" foi gravado na pasta quando o seu `pnpm dev` reiniciou com o código novo.
- **Recuperação:** a cópia é gravada no segundo seguinte a cada mudança e apagada ao salvar, abrir ou criar um projeto.
- **Prévia:** botão no cabeçalho do editor de cena e "PRÉVIA" na barra de cima. Este aparelho desliga a prévia que ligou ao trocar de criação ou sair do Studio.
- **Localizar:** em Fixtures → No projeto e no editor de cena. Acompanha a fixture escolhida, para percorrer o palco tocando uma a uma.
- **Desfazer e refazer:**
  - histórico no servidor;
  - um arrasto vira um passo só;
  - aviso "Desfeito" e "Refeito";
  - o console em operação deixa as teclas para os elementos.
- **Diferenças do plano:**
  - **Recuperação:** gravada no máximo 1 s depois da mudança, e não 5 s.
  - **Lista de projetos:** relida a cada segundo, para um arquivo copiado para a pasta aparecer sem reiniciar.
  - **Localizar no editor de cena:** fica na fileira das fixtures, ao lado de "Adicionar fixtures", e não ao lado de "Rodar", porque age nas fixtures escolhidas ali.
  - **Projetos (lista do menu):** não marca o item escolhido. Assim, o segundo toque no mesmo projeto (para descartar alterações e abrir) chega.
- **Testes:**
  - automáticos:
    - arquivo: salvar e ler idêntico, `.bak`, nomes e arquivos recusados;
    - hub: primeira vez em branco com o exemplo na pasta, "Salvar como" e abrir, desfazer e refazer com arrasto e operação ao vivo;
    - motor: ordem das camadas;
    - prévia e localizar nos endereços certos, e o pânico desligando os dois;
  - no navegador, contra o seu `pnpm dev`:
    - abrir o exemplo pela lista;
    - Ctrl + Z e Ctrl + Shift + Z num nome de fixture, com o aviso;
    - o console em operação ignorando e, na edição, desfazendo;
    - prévia levando a cena para a saída (255, 255, 140, iguais à cena) e desligando ao trocar de cena;
    - localizar acendendo o dimmer e acompanhando a escolha;
    - a cópia de recuperação aparecendo;
    - "Salvar como";
    - o menu no celular;
    - ao reiniciar, o app abriu o último projeto salvo.
  - Depois do teste: o app num projeto em branco, a pasta só com o "Show de exemplo", sem recuperação e com o `settings.json` como estava.

## Objetivo

Salvar o trabalho e programar olhando a luz. Depois da F5:
- o projeto é salvo e aberto de verdade, e nada se perde se o app cair ou for fechado sem salvar;
- o editor de cena (o programador, E12) pode mandar para a luz a cena que está sendo editada;
- um botão acende uma fixture para encontrá-la no palco;
- qualquer alteração pode ser desfeita (Ctrl + Z) e refeita (Ctrl + Shift + Z).

## O que já existe (F2 e F3)

- **Patch:** conflitos sinalizados, endereçamento automático e arrasto na grade.
- **Grupos de canais:** no projeto, usados pela Mesa simples.
- **Programador:** as ferramentas do editor de cena (seleção, intensidade, cor, posição, faixas de valores) já gravam na cena.
- **Operações:** tipadas, com revisão, e celular e desktop sincronizados.

Esses itens só entram na validação.

## O que entra

### 1. Arquivo de projeto (A5)

- **Formato:** `.rlc`, JSON com `{ "format": "r-light-control/project", "schemaVersion": 1, "project": … }`. As definições das fixtures já vão dentro (A4).
- **Pasta:** `Documentos\R Light Control\`, um arquivo por projeto, com o nome do projeto.
- **Salvar:** escrita atômica, com um arquivo temporário que toma o lugar do antigo. Antes disso, a versão anterior vira `.rlc.bak`.
  - O QLC+ apaga o arquivo antes de pôr o novo no lugar, e não guarda cópia.
- **Salvar como:** pelo nome, como no protótipo congelado. O projeto passa a ter esse nome. Um nome que já existe é recusado.
- **Abrir:** pela lista dos projetos da pasta, do mais recente para o mais antigo. Com alterações não salvas, pede um segundo toque, como o "Novo".
- **Erros:**
  - Um arquivo inválido é recusado, e o projeto aberto continua como está. O QLC+ apaga o projeto atual antes de ler o novo.
  - Um arquivo de uma versão mais nova do app também é recusado.
  - As migrações (A5) começam quando houver a versão 2.

### 2. Salvamento automático e recuperação

- Com alterações não salvas, o servidor grava uma cópia de recuperação em `%APPDATA%\r-light-control\recuperacao.rlc`. A gravação acontece no máximo 5 s depois da última mudança. O QLC+ espera 30 s e não tem recuperação.
- Salvar, abrir ou criar um projeto apaga a cópia de recuperação.
- Ao iniciar, se a cópia existe, o app abre por ela:
  - com o caminho original e as alterações marcadas como não salvas;
  - o menu Projeto avisa "Recuperado da última sessão".
- **O que abre ao iniciar:**
  1. a recuperação, se houver;
  2. senão, o último projeto aberto;
  3. senão (primeira vez), um projeto em branco. O show de exemplo é gravado na pasta na primeira vez, para abrir pela lista.

### 3. Prévia: a cena em edição na luz (o programador)

- Um botão **"Prévia"** no editor de cena manda os valores da cena para a luz enquanto ela é editada. Cada mudança aparece na hora. É o "Function Preview" do QLC+ 5 (no QLC+ 4, o contrário: "Blind").
- **Uma prévia por vez, para todos os aparelhos:**
  - o estado fica no servidor, como o padrão de teste;
  - a barra de cima mostra "PRÉVIA" enquanto ela está ligada, como mostra "TESTE".
- **Desliga sozinha** ao sair do editor ou trocar de cena, e no pânico.
- **Camadas da saída**, de baixo para cima:
  1. valor padrão;
  2. prévia;
  3. Mesa simples (como no QLC+, a mesa vence);
  4. localizar;
  5. Grand Master e blackout.
- Até a F6, "Rodar" continua sem acender nada. Na F6 entram a mistura entre as criações e a edição da cena rodando.

### 4. Localizar uma fixture (o "highlight" do QLC+)

- Um botão **"Localizar"** acende as fixtures escolhidas:
  - dimmer e vermelho, verde, azul e branco a 100%;
  - shutter e o resto no valor padrão (o Strobo do Beam já fica em "luz ativa");
  - pan e tilt não mudam.
- É liga e desliga, para uma ou várias fixtures, e fica acima da Mesa simples.
- Fica em dois lugares:
  - **Fixtures → No projeto**, na fixture escolhida, para conferir o patch;
  - **editor de cena**, nas fixtures escolhidas.
- O QLC+ grava esses valores na cena quando ela está em edição. Aqui eles não entram na cena.

### 5. Desfazer e refazer (pedido do usuário)

- **Ctrl + Z** desfaz a última alteração e **Ctrl + Shift + Z** a refaz, em todas as telas e menus.
- **Exceção:** com o console na frente e em operação (a tela aberta, o painel ativo do modular ou a janela externa dele), as duas combinações vão para os elementos, que podem usá-las. Na edição do console, elas desfazem e refazem, como nas outras telas (ex.: um botão movido).
- **O que entra:** tudo o que é editado nos menus e fica salvo:
  - o projeto inteiro: fixtures, patch, grupos, criações do Studio, console, palco, universos e o nome;
  - a biblioteca de fixtures (salvar, importar, remover).
- **O que não entra:**
  - a operação ao vivo: Mesa simples, Grand Master, blackout, rodar e parar, valores dos elementos do console, prévia, localizar e padrão de teste;
  - as preferências de cada aparelho (zoom, formato do console);
  - o acesso pela rede local, o PIN e o tipo da interface USB, que agem na hora sobre a rede e o hardware.
- **Um arrasto desfaz de uma vez:** mudanças seguidas no mesmo lugar, com menos de meio segundo entre elas, viram um passo só (ex.: um fader do editor de cena).
- **Histórico:**
  - um só, no servidor, para todos os aparelhos: o Ctrl + Z desfaz a última alteração, venha de onde vier (A2);
  - os últimos 100 passos;
  - "Novo" e "Abrir" começam um histórico novo;
  - desfazer marca o projeto como alterado, e o salvamento automático grava.
- **Num campo de texto,** o Ctrl + Z desfaz o que está sendo digitado, como em qualquer campo. Fora do campo, desfaz a alteração já gravada.
- **Aviso curto** "Desfeito", "Refeito" ou "Nada para desfazer", porque a alteração pode estar em outra tela.
- **Protocolo:** operações `undo` e `redo`. O servidor guarda, em cada passo, como estavam os pedaços do estado que a operação mudou (os mesmos tópicos do `evt`). Desfazer devolve esses pedaços e avisa todos os aparelhos.

### 6. Telas (G2)

Mudanças pequenas, nos padrões do design system:
- **Menu Projeto:** Salvar, "Salvar como" pelo nome e a lista **Projetos** (no lugar de "Recentes"). Sai o aviso "chegam na F5". Entra a linha "Recuperado da última sessão" quando for o caso.
- **Editor de cena:** "Prévia" e "Localizar" (botões com LED) ao lado de "Rodar".
- **Fixtures → No projeto:** "Localizar" na fixture escolhida.
- **Barra de cima:** "PRÉVIA" enquanto a prévia está ligada.
- **Aviso** de desfazer e refazer, no mesmo lugar do aviso das recusas.

## Decisões para você

1. **Projetos numa pasta só** (`Documentos\R Light Control\`), abertos pela lista, também no celular (recomendado).
   - Não há o explorador de arquivos do Windows para abrir de outro lugar: para isso, copie o arquivo para a pasta.
   - A alternativa é abrir pelo diálogo do Windows, só no desktop.
2. **Ao iniciar:** recuperação, senão o último projeto, senão o exemplo (recomendado). **Resposta:** senão, um projeto em branco.
3. **Fechar sem salvar não perde nada** (recomendado). Na próxima vez, o app abre com as alterações, marcadas como não salvas.
   - Não há a pergunta "Salvar antes de sair?". Para descartar, use "Novo" ou abra outro projeto.
4. **Prévia desligada ao abrir uma cena; um toque liga** (recomendado, como no QLC+ 5). Assim, abrir uma cena no meio do show não muda o palco.
   - A alternativa é ligada ao abrir, como no QLC+ 4: mais rápido para programar, mais arriscado ao vivo.
5. **A Mesa simples fica acima da prévia**, como no QLC+ (recomendado). Um canal levantado na mesa não muda pela cena; "Zerar" o solta.
6. **As mudanças de tela da seção 6.**
7. **Desfazer:**
   - um histórico só para todos os aparelhos (recomendado);
   - o que entra e o que não entra, como na seção 5 (recomendado);
   - "Novo" e "Abrir" começam um histórico novo (recomendado).
8. **Celular e tablet:** sem teclado, não há Ctrl + Z. Recomendo ficar só no teclado, como você pediu. Se quiser botões de desfazer e refazer na barra de cima, eles entram junto.

## Fica de fora

Nada disso apareceu nas telas aprovadas:
- **Modificadores de canal** (curvas por canal).
- **HTP/LTP forçado e "pode fazer fade"** por canal.
- **Fixtures que atravessam universos.**
- **"Dump"** (gravar os valores da saída numa cena). Aqui o programador já é o editor de cena.

## Testes automáticos

- **Arquivo:**
  - salvar e abrir de volta dá um projeto idêntico;
  - o `.bak` guarda a versão anterior;
  - arquivo inválido ou de versão mais nova é recusado sem mexer no projeto aberto;
  - nome repetido no "Salvar como" é recusado.
- **Recuperação:** a cópia é gravada depois de uma mudança, apagada ao salvar e usada ao iniciar, com as alterações marcadas como não salvas.
- **Desfazer e refazer:**
  - desfazer e refazer de cada tipo de alteração volta ao estado exato;
  - um arrasto vira um passo só;
  - a operação ao vivo não entra;
  - remover da biblioteca e desfazer devolve o arquivo;
  - "Novo" e "Abrir" limpam o histórico.
- **Motor:** a ordem das camadas (padrão, prévia, mesa, localizar, Grand Master e blackout).
- **Servidor:**
  - endereços da prévia e do localizar;
  - a prévia desliga ao trocar de cena e no pânico;
  - o localizar não entra na cena.
- `pnpm check` passando, e um teste no navegador.

## Validação (com você)

Antes de começar:
- **Reinicie o `pnpm dev`.** O app abre em branco: o seu rig está no "Show de exemplo", no menu Projeto → Projetos.
- **Matar o processo:** no Gerenciador de Tarefas, finalize "r-light-control" (com o `pnpm dev`, rode-o de novo depois). Na volta, o menu Projeto mostra "Recuperado da última sessão".
- **Desfazer:** clique fora dos campos antes do Ctrl + Z. Dentro de um campo, ele desfaz só a digitação.

- [ ] 20 fixtures em 2 universos, com um conflito sinalizado no Patch e resolvido pelo endereçamento automático.
- [ ] Salvar, fechar o app e abrir de novo: o projeto volta idêntico.
- [ ] Matar o processo (Gerenciador de Tarefas) com alterações não salvas: ao abrir, elas voltam.
- [ ] Prévia: editar uma cena com a prévia ligada muda a luz na hora. Desligada, nada muda.
- [ ] Localizar acende a fixture certa no palco.
- [ ] Celular e desktop sincronizados (inclusive a prévia e o localizar).
- [ ] Ctrl + Z e Ctrl + Shift + Z desfazem e refazem em todas as telas (fixtures, patch, Studio, console em edição, palco, biblioteca).
- [ ] Com o console em operação, Ctrl + Z aciona o elemento que usa essa tecla, e não desfaz.
- [ ] `pnpm check` passa.

## Referências estudadas

- `engine/src/{doc,fixture,channelmodifier,genericdmxsource,universe}.cpp`
- `qmlui/{app,contextmanager,fixturemanager,functionmanager,sceneeditor}.cpp`, `qmlui/tardis/tardis.cpp`
- `ui/src/{app,sceneeditor}.cpp` (QLC+ 4: backup, recentes e "Blind")

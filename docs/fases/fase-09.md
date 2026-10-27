# F9 — RGB Matrix

**Status:** mini-plano aprovado em 2026-10-27, com as sete recomendações (na 5, "Não precisa": sem Shutter). Entregue no mesmo dia (ver [Entregue](#entregue)), aguardando a validação, que vai junto com a da F10c e a da F10d.

## Objetivo

Um grupo de fixtures (a grade 2D de cabeças, F8) vira uma **tela de pixels**: um algoritmo desenha, a cada passo, uma cor por célula, e o motor leva essas cores aos canais das cabeças. Entram:
- **algoritmos nativos** em Rust: Plain, Text e Image;
- **scripts do QLC+** (JavaScript), importados pelo usuário e rodando sem poder derrubar o motor;
- a **prévia animada** do editor, igual ao que sai na luz.

## O que já existe

- **Contrato (`RgbMatrix` em `rlc-core`):** grupo, algoritmo (texto), cores, ordem, sentido e tempos. O servidor já confere o grupo (cada cabeça uma vez, dentro da grade) e o RGB Matrix perde o grupo se ele for removido.
- **Editor (`StudioRgbMatrix.vue`):** prévia, grupo, algoritmo, cores, ordem, sentido e tempos. A prévia usa cinco algoritmos de **amostra em TypeScript** (`matrixColor`), que não acendem nada: `apply.rs` devolve `None` para o RGB Matrix.
- **Motor:** a árvore `Program` já roda cenas, chasers, coleções e EFX. O EFX é o molde: um _runner_ por criação, com a mistura por canal, a prioridade, a pausa e o fade in e out (F6, F8, F10c). O elemento "Tempos" já manda no ciclo do EFX, com "Seguir o BPM".
- **O que o QLC+ tem e vamos ter igual** (`engine/src/rgbmatrix.cpp`): o algoritmo devolve um **mapa** (linha × coluna de cores `0xRRGGBB`) por passo; o passo dura o tempo "Passo"; cada canal que muda faz fade até o valor novo; as cores inicial e final dão o degradê dos passos.

## O que entra

### 1. Motor e saída (rodada 1)

- **Programa novo, `RgbMatrix`**, na árvore `Program`, como o EFX: o servidor traduz o grupo em células (cabeça → endereços DMX dos canais de cor) e o motor guarda o passo atual, o mapa e o fader de cada canal.
- **Do mapa aos canais, como no QLC+:**
  - **RGB** (padrão): R, G e B da cabeça; sem RGB, CMY (`cyan`, `magenta`, `yellow` da cor); sem nenhum dos dois, a cabeça não acende. O dimmer, o branco, o âmbar e o UV **não são tocados**.
  - **Dimmer, Branco, Âmbar e UV:** um canal só, com o cinza da cor (0,299 R + 0,587 G + 0,114 B).
  - O Shutter do QLC+ fica de fora (decisão 5).
- **Passo:** dura o tempo "Passo" (hoje `holdMs`), em segundos ou em batidas (com "Seguir o BPM", como o ciclo do EFX); em batidas, troca na batida. Um passo novo por tick, no máximo.
- **Ordem:** Loop, Uma vez e Vai e volta (o QLC+ não tem Aleatória no RGB Matrix; decisão 6). Sentido: Frente ou Trás. O Vai e volta não repete os extremos, como no QLC+.
- **Cores:** a cor do passo vai da inicial à final em inteiros (`inicial + delta × i / (n − 1)`), como no QLC+. O algoritmo diz quantas cores aceita (1 a 5).
- **Transição entre passos, como no QLC+:** sem crossfade entre mapas; cada canal que muda vai do valor de agora ao novo, com o **fade in** (ou o **fade out**, quando o novo é 0). Fade maior que o passo nunca chega ao alvo (como lá).
- **Entrar e sair, como as cenas e o EFX:** os canais partem do que está embaixo; ao parar, descem no fade out até o que está embaixo (decisão 2 da F6; o QLC+ desce a 0).
- **Mistura:** os canais de cor são de intensidade (HTP), com a prioridade, a pausa, o slider de playback e o submaster da criação (F10, F10c), como os de uma cena. Pausada, o passo e os fades congelam.
- **Mudar a criação rodando** (cores, tempos, ordem) vale na hora, sem reiniciar; mudar o algoritmo, o texto, a imagem ou uma propriedade recomeça do passo 0.
- **Elemento "Tempos":** vale para o RGB Matrix da lista: fade in, fade out e a duração do passo (em batidas, seguindo o master), como no EFX.

### 2. Algoritmos nativos (rodadas 1 e 3)

- **Plain** (rodada 1): preenche a grade com a cor. Um passo, uma cor.
- **Text** (rodada 3): duas cores (o texto muda de cor por passo, não por pixel), três estilos como no QLC+: **Letras** (um passo por letra, centrada), **Horizontal** (rola 1 px por passo) e **Vertical** (uma letra por vez, rolando), com o deslocamento X e Y.
  - O QLC+ usa QFont e QPainter. Aqui, **uma fonte de pontos 5×7 própria, embutida** (letras sem acento, números, pontuação e as vogais e o ç do pt-BR), com limiar de 1 bit e avanço fixo. **Não fica pixel a pixel igual ao QLC+.**
- **Image** (rodada 3): o pixel vem direto da imagem, sem cor; estilos **Estática**, **Horizontal** e **Vertical** (rolam um pixel por passo, dando a volta, como no QLC+); alfa 0 vira preto.
  - A imagem (PNG, JPEG ou o primeiro quadro de um GIF) **vai dentro do projeto**, no máximo 256 KB, reduzida à grade ao desenhar. O GIF animado e a opção "Animation" do QLC+ ficam de fora.

### 3. Scripts do QLC+ (rodada 2)

- **O que se aceita** (os 60 de `resources/rgbscripts/`, APIs 1, 2 e 3): `rgbMap(width, height, rgb, step)`, `rgbMapStepCount(width, height)`, `name`, `author`, `apiVersion`, `acceptColors`, as **propriedades** do API 2 (`name:x|type:list|display:…|values:a,b|write:f|read:f`, com `list`, `range`, `float` e `string`) e `rgbMapSetColors` e `rgbMapGetColors` do API 3. É JavaScript ES5; o QuickJS (crate `rquickjs`) cobre.
- **Importar:** "Importar script" no editor, vários `.js` de uma vez, também do celular. O script vai **dentro do projeto** (nome, autor, API e texto), como as definições de fixture (A4): um projeto nunca quebra por falta de script. **Não há pasta de scripts no PC** (decisão 3).
- **Cada criação tem a sua instância** do script (eles guardam estado: `plasma` e `fire` avançam o tempo a cada chamada; oito reiniciam no `step === 0`). O motor chama o `rgbMap` **em ordem, uma vez por passo**, como o QLC+.
- **Fora da thread do motor (decisão 4):** uma **thread de scripts** roda o JS e entrega os mapas ao motor, que só os lê. Ela **pede o passo seguinte um passo antes**: o motor sabe qual será (a ordem é determinística). Se o mapa não chegou a tempo, o motor mantém o atual naquele tick.
- **Limites** (o QLC+ não tem nenhum: um laço infinito trava o timer dele): **250 ms por chamada** e **32 MB por instância**, pelo interrupt handler e pelo limite de memória do QuickJS. Estourou ou deu erro: a criação **para de trocar passos** (os canais descem no fade out), o erro aparece no editor, e o resto do app segue.
- **As propriedades** valem como o QLC+: o valor vai ao `write` do script como texto, e os valores ficam na criação (`properties`).
- **Os scripts do QLC+ não vêm no app**, como as fixtures (E3, G3). Se um script for copiado para o repositório (exemplo ou teste), a atribuição Apache-2.0 entra no `NOTICE`.

### 4. Contrato, arquivo e protocolo

- **`RgbMatrix`:** o `algorithm` vira um tipo (`plain`, `text`, `image` ou `script` com o nome); ganham `control` (RGB, Dimmer, Branco, Âmbar, UV), `properties` (nome e valor, em texto), o bloco do Text (texto, estilo, X e Y) e o da Image (bytes, estilo, X e Y). `colors` passa a ter até 5 cores. O projeto ganha a lista de **scripts importados**.
- **Projeto na versão 11:** a migração acrescenta os campos novos. Os algoritmos de amostra antigos (`chase`, `stripes`, `gradient`, `random`) abrem como Plain; a ordem Aleatória abre em Loop; o resto fica.
- **Protocolo:** `saveFunction` continua gravando a criação inteira; novas operações: `importScript` (como o `importQxf`: devolve o nome ou o erro, com a recusa de arquivo que não é script) e `removeScript` (só sem criação usando). **`previewMatrix`** devolve o mapa de um passo (a prévia, decisão 1) e o número de passos, no `ack`. O erro de um script rodando vem no estado, junto da criação. `docs/protocolo.md` atualizado.

### 5. Telas (G2)

- **Editor de RGB Matrix:**
  - a prévia vem do servidor (`previewMatrix`), com o mesmo mapa que sai na luz; a função `matrixColor` e os cinco algoritmos de amostra de `utils/functions.ts` saem;
  - "Algoritmo" lista Plain, Text, Image e os scripts importados, com "Importar script…" e "Remover";
  - o bloco do algoritmo mostra o que ele pede: o texto, o estilo e o deslocamento; a imagem; as **propriedades do script** (listas, faixas), e o **erro** quando há;
  - as cores aparecem na quantidade que o algoritmo aceita; "Controle" (RGB, Dimmer, Branco, Âmbar, UV); a ordem mostra só Loop, Uma vez e Vai e volta.
- **Studio, console e 3D View:** nada muda; o RGB Matrix roda como as outras criações e a 3D View mostra o que sai do motor.

### 6. Exemplos no show de exemplo (G8)

No `exemplo.json` e no "Show de exemplo" da sua pasta, numa pasta "RGB Matrix", com os ParLeds:
- **Cor fixa (Plain)** e **Degradê** com duas cores, em "Vai e volta";
- **Texto rolando** (Horizontal) com o nome do show;
- **Imagem** pequena rolando;
- **Dois scripts escritos por mim**, simples e originais (uma onda e uma chuva de pontos), importados no projeto, um deles com propriedade (a velocidade ou o tamanho);
- uma cena **"Base"** com o dimmer dos ParLeds, porque o modo RGB não acende o dimmer, como no QLC+.

## Decisões a aprovar

1. **A prévia vem do servidor, não da tela.** A tela pede o mapa de cada passo (`previewMatrix`).
   - _Alternativa:_ fazer as contas também em TypeScript (como o EFX). Para o Plain é fácil, mas o Text (fonte), a Image e os scripts (QuickJS) precisariam de uma segunda versão cada, sem garantia de bater.
   - **Recomendo o servidor** (G7): uma implementação só, e a prévia é exatamente o que sai. O custo é um pedido por passo, aceitável (o passo tem dezenas de ms, no mínimo).
2. **Entrar e sair pela regra das cenas** (parte de baixo e volta a ela no fade out), e não descer a 0 como o QLC+. É a regra única do motor desde a F6, e a prioridade e o HTP dependem dela.
   - _Alternativa:_ descer a 0, como o QLC+. Em cima de uma cena de base, a matriz apagaria a base.
3. **Scripts só dentro do projeto,** importados pelo usuário, sem biblioteca em pasta (as fixtures têm porque são 1.781; os scripts são 60).
   - _Alternativa:_ pasta `%APPDATA%\r-light-control\scripts\`, como a de fixtures: importa uma vez e vale em todos os projetos, mas é mais um lugar de armazenamento, de migração e de tela.
   - **Recomendo dentro do projeto** (G7): mais simples, e o projeto é autossuficiente (A4). A pasta entra depois se o reimportar incomodar.
4. **O JavaScript roda numa thread à parte, com o passo seguinte pedido antes e limites de 250 ms e 32 MB por chamada.** Erro ou estouro para a criação, sem afetar o motor (é o critério de validação da fase).
   - _Alternativa:_ rodar o JS na própria thread do motor (como o QLC+): mais simples, mas um script lento perde ticks de todas as saídas, e um laço infinito a trava.
5. **Modos de saída: RGB (com CMY), Dimmer, Branco, Âmbar e UV; sem Shutter** (que o QLC+ tem). É um canal de estroboscópio, e o cinza como velocidade de strobo é um uso raro. Entra se você pedir.
6. **Sem ordem Aleatória no RGB Matrix** (o QLC+ também não tem). O contrato compartilha o tipo com o chaser, e o editor passa a mostrar só as três.
7. **Text e Image na rodada 3, com a fonte própria de 5×7 e a imagem dentro do projeto.** Se preferir, a rodada 3 sai da F9 e vai para o Backlog, e a F9 fica com o Plain e os scripts (que já cobrem quase todos os efeitos do QLC+; os scripts incluem `marquee`, `stripes`, `gradient`…).

## Fica de fora

- O **RGB Audio** do QLC+ (FFT do áudio): o áudio está fora da v1 (E5).
- O **GIF animado** e o estilo "Animation" da Image; o **Shutter** (decisão 5); a **ordem Aleatória**.
- O **blend mode** do QLC+ (Máscara, Aditivo, Subtrativo): a mistura é a do motor (HTP e prioridade).
- O **Tap** do passo e a ressincronização "por tempo" dentro de uma batida do QLC+ (linhas 767-788 do `rgbmatrix.cpp`): o passo em batidas troca na batida, e o TAP do BPM é o do master.
- A **Animation** do console (F11) e os sliders das cores e das propriedades do QLC+ 5 ao vivo: entram na F11, que já tem o elemento.
- Fixtures de **várias cabeças** sem grade: continua valendo o grupo de fixtures (cada cabeça uma vez, F8).

## Testes automáticos

- **Motor (`rlc-engine`):**
  - o mapa vira os canais: RGB, CMY, cinza (Dimmer, Branco, Âmbar e UV); o RGB não toca o dimmer nem o branco; a cabeça sem canal de cor não acende;
  - os passos: Loop, Uma vez e Vai e volta (sem repetir os extremos), Frente e Trás; o degradê das cores; o passo em segundos e em batidas (trocando na batida) e **no máximo um passo por tick**;
  - os fades por canal: fade in ao subir, fade out ao zerar; entrar a partir do que está embaixo e parar no fade out; a prioridade, o HTP, o slider e a pausa; mudar as cores rodando sem reiniciar;
  - na coleção e no passo de um chaser (como o EFX).
- **Scripts (`rlc-script`, crate novo):** carregar os 60 scripts do QLC+ (nome, API, cores, propriedades); `rgbMap` em ordem com o estado do script; as propriedades `list` e `range` com o `write`; **laço infinito abortado** em 250 ms e **memória estourada**, sem derrubar nada; o erro volta para a criação. Casos portados de `engine/test/rgbscript` (`property`, `evaluateException`, `evaluateInvalidApiVersion`).
- **Text e Image:** `staticLetters`, `horizontalScroll` e `verticalScroll` de `engine/test/rgbtext` (com as medidas da nossa fonte); Image: o wrap por módulo, o alfa 0 e o limite de 256 KB.
- **Servidor:** a tradução do grupo em células e endereços; a conferência do RGB Matrix salvo; `importScript`, `removeScript` e `previewMatrix`; a migração da versão 10.
- **No navegador, contra o seu `pnpm dev`:** o editor (os algoritmos, as propriedades, o erro), a prévia batendo com a luz e o 3D. Com backup antes e comparação depois.
- `pnpm check` passando.

## Validação (com você)

Abrindo o **Show de exemplo**, na pasta **RGB Matrix** do Studio (o painel 8 × 8 e os ParLeds estão no universo "Fundo" e no "Palco", sem saída real). O modo RGB não acende o dimmer, como no QLC+: ligue a cena **Base dos ParLeds** junto de "Cor fixa nos ParLeds" e de "Onda nos ParLeds".
- [ ] **Cor fixa nas barras** e **Cor fixa nos ParLeds** acendem as cabeças (a prévia do editor mostra a mesma cor); parar volta ao que estava embaixo, no fade out.
- [ ] **Degradê rolando nas barras** (script do API 3, duas cores): o degradê anda uma coluna por passo; trocar a cor inicial ou a final no editor muda na hora, sem reiniciar.
- [ ] **Onda nos ParLeds**: mudar a propriedade **Comprimento** muda a onda (recomeça do primeiro passo).
- [ ] **Faíscas nas barras**: a propriedade **Densidade** (Baixa, Média, Alta) muda quantas células acendem.
- [ ] **Texto rolando no painel**: "R LIGHT" passa pelo painel de 8 × 8, e o editor mostra o mesmo; trocar para **Letras** e **Rolando ↑**; escrever outro texto (as letras com acento saem sem o acento).
- [ ] **Imagem rolando no painel**: os corações vermelhos passam pelo painel; **Escolher imagem…** com um PNG seu (reduzido a no máximo 128 × 128).
- [ ] **Importar script…**: importar vários `.js` da sua pasta do QLC+ (`...\Scripts\rgbscripts`) de uma vez, escolher alguns e rodar; um arquivo que não é script é recusado com o motivo.
- [ ] A **prévia 8 × 8** bate com a do QLC+ instalado no seu PC, para o mesmo script.
- [ ] Um **script com laço infinito** (um `.js` com `while (true) {}`, importado de propósito) é abortado em 250 ms: o editor mostra o erro, a criação para, e o resto do app não trava.
- [ ] O **TAP do BPM** e o elemento **Tempos** (com "Seguir o BPM") mudam a velocidade do passo.
- [ ] Os projetos antigos abrem (migração); `pnpm check` passa.
- Na [validação final](../validacao-final.md): nada com os Beams; o RGB Matrix só mexe nas cores.

## Entregue

- **Contrato (`rlc-core`):** `RgbMatrix` com o `algorithm` como tipo (`plain`, `text`, `image` ou `script`), o `control` e as cores; o projeto ganha `scripts` (os importados). Projeto na versão 11, com a migração (os algoritmos de amostra viram Plain; a ordem Aleatória vira Loop; `scripts` vazio). Operações `importScript`, `removeScript` e `previewMatrix`; o `Snapshot` ganha `failures` (as criações que pararam por erro).
- **Motor (`matrix.rs`, `text.rs`):** um fader só para todas as cabeças (entra a partir do que está embaixo e sai no fade out); RGB, CMY e o cinza (dimmer, branco, âmbar e UV); cor do passo entre a inicial e a final; Loop, Uma vez e Vai e volta, para frente e para trás; passo em segundos ou em batidas; um passo por quadro, no máximo; pausa; edição na hora. O texto usa a fonte de pontos 5 × 7 própria; a imagem dá a volta pelo módulo.
- **Scripts (`rlc-script`):** QuickJS (`rquickjs` 0.14) numa thread por RGB Matrix rodando. O motor pede o mapa do passo seguinte assim que o atual aparece e nunca espera: se não chegou, o passo atual continua. Limites de 250 ms por chamada e 32 MB (o laço infinito e o estouro de memória viram erro). Os 60 scripts do QLC+ abrem, e os passos 0 a 2 de cada um voltam em mapas 8 × 8.
- **Servidor:** a tradução do grupo em células e canais (`matrix_cells`), a decodificação da imagem, as conferências do RGB Matrix salvo e a prévia (`previewMatrix`): os nativos pelo `apply`, e os scripts pelo hub (`script.rs`, uma sessão por criação). O erro de um script rodando vai para `failures` e some quando a criação roda de novo.
- **Telas:** o editor com a prévia do servidor (num componente à parte, `StudioRgbMatrixPreview.vue`: o relógio dela redesenhava o editor a cada passo, e os campos voltavam ao valor salvo enquanto se digitava, como já tinha acontecido no EFX), o algoritmo (Cor fixa, Texto, Imagem e os scripts), "Importar script…" (vários), "Remover script", as propriedades do script, o texto e o estilo, "Escolher imagem…" (o navegador lê e reduz a imagem), o deslocamento X e Y, os canais controlados, as cores que o algoritmo usa (1 na cor fixa, 2 no texto, nenhuma na imagem, as do script) e o aviso de erro. O elemento Animation do console escolhe o algoritmo também.
- **Exemplos (G8):** pasta "RGB Matrix" no `exemplo.json` e no "Show de exemplo" do usuário (a recuperação, na versão 11, e o arquivo salvo, que o app migra ao abrir): "Cor fixa nas barras", "Cor fixa nos ParLeds" (com a cena "Base dos ParLeds"), "Degradê rolando nas barras", "Onda nos ParLeds", "Faíscas nas barras", "Texto rolando no painel" e "Imagem rolando no painel" (o painel 8 × 8 são as 2 barras do exemplo e mais 6, no universo "Fundo"). Os 3 scripts e a fonte foram escritos para o projeto.
- **Testes:** 68 no motor (os do RGB Matrix: canais, fades, edição, pausa, Single Shot, a ordem dos passos, os scripts em ordem, a propriedade, o script quebrado e o texto), 23 no `rlc-script` (inclui os 60 scripts reais do QLC+, o laço infinito e a memória) e 63 no servidor (a tradução, as conferências, importar e remover scripts, a prévia de script, texto e imagem, os exemplos e a migração da versão 10). Contra o servidor sem sinal de saída (`rlc-headless`), com o seu projeto migrado: os 7 exemplos acendem e apagam, e a prévia de cada um bate com a grade. No navegador, contra o `nuxt dev`: o editor de cada exemplo, sem erros na página.

### Diferenças do plano

- **O Plain tem um passo só** (como no QLC+), então o "Degradê" do exemplo é um script do API 3 ("Degradê rolando"), e não o Plain com duas cores.
- **A imagem vai como pixels RGBA em base64** (no máximo 128 × 128 e 256 KB), lidos pelo navegador ao escolher o arquivo (PNG, JPEG, GIF, o primeiro quadro, WebP…), e não como os bytes do arquivo: o servidor não precisa de decodificadores de imagem.
- **Os exemplos de script são três** (uma onda, um degradê e as faíscas), no lugar de "uma onda e uma chuva de pontos".
- **Uma thread por sessão de script**, e não uma só para todas: um script lento atrasa só a criação dele.
- **O erro vai para o `Snapshot`** (`failures`) para o editor mostrar o motivo de uma criação que parou.

## Referências estudadas

- Do projeto: o EFX como molde (`crates/rlc-engine/src/efx.rs`, `running_programs` e `efx_expand` em `apply.rs`), os tempos, a prioridade e a pausa (F6 e F10c), o contrato (`RgbMatrix`, `FixtureGroup`, `GroupCell` e as cabeças de `Mode` em `rlc-core`) e o editor atual (`StudioRgbMatrix.vue`).
- QLC+ 5.3.0 (só leitura): `engine/src/{rgbmatrix,rgbalgorithm,rgbplain,rgbtext,rgbimage,rgbscript,rgbscriptv4,rgbscriptscache,rgbscriptproperty}` e `resources/rgbscripts/` (60 scripts: 9 de API 1, 35 de API 2 e 16 de API 3, ~13,9 mil linhas, ES5), com os testes de `engine/test/`. O QLC+ não tem cache de passos (só do nome → arquivo), nem timeout nem limite de memória no JS.

# Decisões do projeto

Regras e decisões que valem entre sessões. Toda decisão nova entra aqui com data. Uma decisão alterada é editada no lugar, com a data da mudança.

## Gerais (regras do usuário)

- **G1. Trabalho em fases com validação.** Executar → o usuário valida → ajustar → só então a próxima fase. Nenhuma fase começa sem a anterior aprovada.
  - **Validação final (2026-10-19, pedido do usuário):** ele ainda não pode testar com os moving beams. Os testes que dependem deles saem do checklist da fase e vão para [docs/validacao-final.md](docs/validacao-final.md), feita numa rodada só antes do release (F16).
- **G2. Interface e padrões primeiro.**
  - Os recursos são implementados sobre telas já aprovadas no protótipo (F2).
  - Mudança de tela passa primeiro pelo protótipo e pelo design system, para evitar retrabalho.
- **G3. O QLC+ é referência, não dependência.**
  - Algoritmos são reimplementados, não decompilados nem copiados.
  - Quando um algoritmo for portado de perto, ou houver reuso de assets (fixtures, meshes, rgbscripts), registrar a atribuição Apache-2.0 em `NOTICE`.
- **G4. Recursos do Virtual Console sob demanda.** O usuário vai pedir recursos extras durante o desenvolvimento. Cada pedido vira um item no `PLANO.md` e uma decisão aqui.
- **G5. Commits só quando o usuário pedir.**
- **G6. Idioma.**
  - Conversa, documentação e textos da UI em pt-BR.
  - Identificadores de código em inglês.
  - Comentários em pt-BR.
- **G7. Código enxuto e legível.**
  - A menor quantidade de código possível, sem minificar demais. Um desenvolvedor deve ler e entender com facilidade.
  - Não criar helpers, abstrações ou camadas para um único uso ou pouquíssimos usos, principalmente quando o código direto ocupa menos linhas que a definição do helper.
  - Abstrair só com repetição real ou exigência do projeto (ex.: a fronteira `EngineClient` mock/real).

## Específicas (técnicas)

- **E1. Stack.** Tauri 2 + Rust (motor, drivers USB, servidor HTTP/WebSocket embutido) + Nuxt 4 em modo SPA (`ssr: false`).
  - Motivos: timing do Open DMX, app leve e caminho para Android na v2.
- **E2. UI ↔ motor só por WebSocket e HTTP.**
  - A janela Tauri abre a mesma UI que o celular acessa pela rede local.
  - O IPC do Tauri fica restrito a recursos nativos (ex.: diálogo de arquivo, zoom da janela). Cada recurso liberado tem uma capability própria em `app/src-tauri/capabilities/`.
- **E3. Formato próprio** (JSON versionado) para projeto e fixture.
  - Importação apenas de fixtures `.qxf` do QLC+, convertidas para o formato próprio.
  - Não há importação de `.qxw`.
  - **Biblioteca de fixtures (2026-10-17):**
    - As fixtures entram só pela importação de `.qxf` feita pelo usuário ou criadas à mão. A biblioteca do QLC+ não é embutida nem lida automaticamente.
    - A biblioteca do usuário fica gravada no PC, um arquivo por definição, em `%APPDATA%\r-light-control\fixtures\`. Na primeira vez, ela começa com as genéricas do exemplo.
    - Excluir da biblioteca não mexe no projeto, que tem a sua cópia (A4).
  - **Conversão do `.qxf` (2026-10-17):**
    - Alias e imagens de gobo ficam de fora, com aviso no relatório.
    - Nas rodas e macros de cor sem cor no arquivo, a cor da faixa vem do nome dela ("Red", "Vermelho"…).
    - O canal fino só vale depois de um canal grosso do mesmo tipo no modo (logo depois ou mais à frente, como em "Pan, Tilt, Pan fino, Tilt fino"), como o QLC+ junta os 16 bits.
- **E4. Interfaces da v1:** Enttec DMX USB Pro e compatíveis, e Open DMX (FTDI sem microcontrolador). Mais uma saída "Dummy" para desenvolver sem hardware.
  - **Driver (2026-10-17):** só o D2XX da FTDI, que vem no driver oficial (o Windows instala sozinho). Sem reserva pela porta COM, como no QLC+. A biblioteca (`ftd2xx.dll`) é carregada do driver instalado, não vai embutida no app.
  - **Tipo da interface (2026-10-17):**
    - pelo nome USB, como no QLC+: "DMX USB PRO" é a Pro; o resto começa como Open DMX;
    - uma Open DMX automática, ao abrir, recebe a pergunta do número de série no protocolo da Pro: se responder, vira Pro (para as compatíveis com outro nome);
    - o tipo pode ser escolhido à mão no In/Out, e fica guardado para a interface (pelo número de série) no `settings.json`.
  - **Interfaces compatíveis:** com chip FTDI (o D2XX só fala com elas). As de chip CH340 ou CP2102 não funcionam (também não funcionam no QLC+).
  - A interface do usuário (genérica, nome "INTERFACE DMX - NUMBSTAR", série A81IPPE0) é Open DMX: é assim que o QLC+ a detecta, sem tipo forçado nas configurações dele.
- **E5. Recursos da v1:** Cenas, Chasers, Sequências, Coleções, EFX, RGB Matrix, Virtual Console e entrada MIDI.
  - Fora da v1: Show/timeline com áudio, Script, Art-Net/sACN, OSC.
- **E6. Design inspirado na Soundcraft Ui24R:** escuro, cara de mesa de som, faders grandes, touch-first, operável no navegador.
  - Visual skeuomórfico: botões, faders, displays e painéis imitam componentes físicos (referência: `docs/design/ui24r-mix.png`).
  - **Só tema escuro.** Não há tema claro.
  - **Cores semânticas vivas, de contraste alto:** matiz pura com saturação e brilho máximos. Exemplos: vermelho `#FF0000`, verde `#00FF00`, amarelo `#FFFF00`, laranja `#FF8000`, ciano `#00FFFF`.
  - O filete de LED apagado fica bem fraco, para o aceso se destacar.
  - **Sem textos de descrição ou ajuda na tela.** Quando uma explicação for necessária, ela fica num ícone "i" ao lado do título da seção, página ou diálogo, que abre um balão (`RlInfo`). Avisos (ex.: conflito de endereço), resultados e telas vazias continuam como texto.
- **E9. Fader.**
  - Arrasto **relativo**: o fader não salta para onde o dedo tocou.
  - Um botão **FINO** logo abaixo de cada fader reduz o deslocamento, para ajustes de precisão.
  - No desktop, a roda do mouse ajusta fader e knob: 1% por clique, 0,1% com FINO.
- **E10. Exibição de valores.** Em % (0–100) por padrão, com opção de DMX (0–255) nas configurações.
  - **Zoom (2026-10-16):** cada aparelho escolhe o zoom geral da interface no ⚙ e o zoom do console ao lado do "Editar".
    - O zoom geral é sempre um zoom de verdade, que mantém medidas, toques, arrastos e balões coerentes: no app desktop, o zoom nativo da janela; no celular e no tablet, a largura do viewport, que o navegador encaixa na tela na escala escolhida. No celular (2026-10-17), o 100% equivale a 75% da escala do navegador, que deixava a interface grande demais. No navegador do computador, vale o zoom do próprio navegador. O `zoom` do CSS foi descartado porque desalinha os balões e os arrastos.
    - O zoom do console aumenta ou diminui as células da grade; se passar da tela, a grade rola.
- **E11. Virtual Console montado pelo usuário.** Como no QLC+, o usuário posiciona e configura cada elemento.
  - Aparência e comportamento dos botões são propriedades de cada elemento: destaque (aceso ou LED), cor, modo (liga/desliga ou flash), ação (função, blackout geral, blackout de fixtures ou grupos, stop all) e ícone.
    - O botão mostra só o texto. O ícone é opcional: ao ligar, começa com o do tipo da criação (ou o da ação) e pode ser trocado por qualquer um da biblioteca (lucide), com busca.
  - Os botões fixos do sistema (FINO, navegação, abas) seguem os padrões do design system.
  - **Layouts por formato de tela (VC-2):** o usuário cria e ajusta o console para cada formato, e cada aparelho abre o layout do seu formato.
    - Os elementos são compartilhados: criação, cor e entradas externas valem em todos os formatos. Cada formato guarda só a posição e o tamanho do elemento, ou não o mostra.
    - Três formatos fixos, escolhidos pela largura da tela: celular (< 640 px), tablet (< 1024 px) e desktop. O ⚙ de cada aparelho pode fixar outro.
    - Grade de 32, 64 e 96 colunas (2026-10-20: o dobro das de antes, a pedido do usuário, para um posicionamento mais fino), para ajuste fino de posição; a célula é quadrada e acompanha a largura. Na edição, a célula mantém o tamanho da operação (o painel de propriedades não aperta os elementos).
  - **Cores dos elementos:** as cores vivas, a versão escura de cada uma (fundo para texto na cor viva) e cinzas do branco ao preto. O elemento Texto tem cor de fundo e de borda.
  - **Fades ao vivo (VC-3):** elemento "Tempos" (como o Speed Dial do QLC+), com a lista das criações que controla e os tempos que ajusta (fade in, fade out e duração, com tap). Sem valor, valem os tempos gravados nas criações.
    - **Ligado às cenas na F6 (2026-10-18, pedido do usuário):** o fade in e o fade out do elemento valem na entrada e na saída das cenas da lista, inclusive numa troca de cenas. Mexer com a cena rodando vale para a saída dela.
    - O QLC+ grava o tempo na função. Aqui o tempo gravado não muda: o elemento vale só ao vivo.
    - Com dois elementos para a mesma cena, vale o último mexido.
    - **Nos chasers (2026-10-18, F7):** o fade in, o fade out e a duração do elemento valem para todos os passos dos chasers e das sequências da lista, em qualquer modo (no QLC+, só nos tempos comuns). Numa coleção da lista, valem para as criações dela. Num chaser em batidas, a duração em segundos não vale: manda o BPM.
    - **Seguir o BPM (2026-10-19, pedido do usuário na validação da F7):** o elemento "Tempos" tem a chave "Seguir o BPM", ligada por padrão (também nos elementos que já existiam).
      - Ligada: fade in, fade out e duração em batidas (0, 1/8, 1/4, 1/2, 1, 2, 4, 8 e 16), com o tempo real vindo do BPM master. O TAP do elemento é o mesmo do master. Um tap muda o BPM, e todos os elementos e chasers acompanham na hora.
      - Os chasers da lista passam a andar nas batidas, inclusive os em segundos; os passos de fração de batida (1/2, 1/4) também caem no tempo.
      - Desligada: segundos e TAP próprio, como antes. Trocar a chave volta o elemento aos tempos gravados.
      - Motivo: o BPM master e os segundos do elemento, cada um com o seu TAP, confundiam.
  - **Cue list (2026-10-18, F7):** ligada ao chaser de verdade já na F7: play, stop, anterior, próximo e toque num passo, com o passo atual vindo do motor. Os faders de crossfade manual do QLC+ ficam para a F11.
- **E12. Organização da interface.** Menus:
  - **Console:** Virtual Console, com a seção master (Grand Master, Blackout, Stop All e o BPM com tap) fixa ao lado; no celular, uma faixa em cima.
  - **Fixtures:** criação, importação, configuração e agrupamento dos equipamentos.
    - Dois tipos de grupo: **de fixtures** (grade 2D de cabeças, para o RGB Matrix e o EFX) e **de canais** (como os "Channel Groups" do QLC+: canais de quaisquer fixtures, do mesmo atributo ou misturados, usados em faders do console e da mesa simples e nas cenas).
    - **Grupo de fixtures (2026-10-19, F8):** cada cabeça entra uma vez só, como no QLC+; pintar uma cabeça que já está no grupo a muda de lugar. No EFX, "Adicionar fixture" tem "Selecionar grupo", que inclui as fixtures do grupo com pan ou tilt na ordem da grade (linha por linha), a ordem da propagação.
    - **Ordem e cópia dos grupos (2026-10-20, pedido do usuário):** nas duas listas, clicar, segurar e arrastar uma linha a leva para outra posição (no toque, segurar e arrastar), sem botão de reordenar. A ordem da lista é a ordem do projeto: vale em todas as telas que mostram os grupos (Mesa simples, console, EFX…). "Duplicar", ao lado de "Remover", cria uma cópia no fim da lista, com o nome "… (cópia)", e a escolhe.
    - Na lista, o grupo de canais mostra o ícone do atributo quando todos os canais são do mesmo tipo, e um ícone genérico quando são misturados.
  - **Studio:** todas as criações (cenas, chasers, sequências, coleções, EFX, RGB Matrix…). Cada tipo tem um ícone próprio, usado em todo o projeto para identificá-lo.
    - **Editor de EFX (2026-10-19, F8):** a ordem não tem "Aleatória" (como no QLC+), e a largura e a altura vão de 0 a 127 para cada lado do centro (127 = o curso inteiro, como no QLC+).
    - **Pastas dentro de pastas (2026-10-20, pedido do usuário):** uma pasta pode ter outras dentro, sem limite de níveis. O botão "Nova pasta" de uma pasta cria a nova dentro dela; arrastar uma pasta a leva para outra (ou para fora, soltando no título do tipo), e uma pasta nunca vai para dentro de si mesma nem das pastas dela. Em cada nível aparecem primeiro as pastas e depois as criações soltas. Remover uma pasta leva o que estava dentro (criações e pastas) para a pasta de cima, ou para o tipo. A contagem de uma pasta inclui as subpastas. O projeto continua na versão 4: o campo `parent` ausente vale "sem pasta de cima".
    - A lista agrupa as criações por tipo (sem filtro de tipo; só a busca). Dentro de cada tipo há pastas. Arrastar uma criação a move para uma pasta do mesmo tipo, ou para fora (soltando no título do tipo): com o mouse, clicar e arrastar; no toque, segurar e arrastar.
    - Ao criar uma coleção com criações rodando, o app pergunta se ela nasce vazia ou com as que estão rodando.
  - **Patch.**
  - **3D View:** visualização, configuração e posicionamento dos equipamentos para o preview.
    - Vistas 3D, topo, frente e lateral. Com "Editar posição" ligado, arrastar uma fixture a move em qualquer vista: no topo pelo chão, na frente em X e altura, na lateral em Z e altura e, no 3D, de frente para a câmera. Os números ficam no painel.
    - Câmera com o mouse (2026-10-16): o botão do meio gira (na vista 3D) ou desloca (nas outras), o direito desloca e só a roda aproxima. O botão esquerdo só escolhe (e arrasta o escolhido com "Editar posição"). No toque, um dedo gira ou desloca e a pinça aproxima.
    - Aba **Elementos** (2026-10-16): elementos de palco que complementam a prévia (box truss de 1, 1,5 e 3 m, praticável 2 × 1 m, escada, painel LED, tela de projeção, caixa de grave, caixa ativa, retorno, pedestal de microfone, pessoa, bateria, teclado, púlpito e planta). Com um elemento escolhido, Ctrl + D duplica ao lado (na direção da largura dele) e Del o exclui; os botões Duplicar e Excluir fazem o mesmo no toque.
    - O toque e o arrasto valem só para o que é da aba aberta: fixtures na aba Fixtures, elementos na aba Elementos (na aba Palco, nenhum dos dois).
    - O tamanho do palco, as posições e os elementos ficam no projeto; a qualidade do desenho e a fumaça, em cada aparelho.
    - **Só o 3D (2026-10-19, pedido do usuário):** o botão "Só o 3D" esconde o painel de fixtures, elementos e palco e deixa a visualização ocupando a tela toda.
  - **Mesa simples:** banco de faders, como no QLC+, para mexer em canais DMX específicos ou grupos de canais fora do console.
    - Em tela larga (a partir de 1024 px), como no QLC+: faders de canal em cima e a faixa dos grupos de canais embaixo. No celular, os grupos ficam numa aba.
    - O fader de um grupo grava nos canais do grupo, e os faders desses canais acompanham. O "Zerar" dos grupos solta os canais deles.
    - Os 512 canais do universo ficam numa faixa só, sem páginas. Ctrl + roda do mouse rola a faixa (sem Ctrl, a roda mexe no fader).
  - **Monitoramento:** os universos mandando sinais, mais dados do app e da máquina (uso de CPU etc.).
  - **In/Out:** uma tela só, sem abas, com duas seções:
    - **Saídas:** universos e interfaces DMX;
    - **Entradas:** só identifica os controladores MIDI conectados por USB.
  - **Atalhos padrão do teclado (2026-10-16):**
    - Ctrl + − e Ctrl + + mudam o zoom geral; com o console na frente (tela aberta, painel ativo do modular ou janela externa dele), mudam o zoom do console.
    - Seção master, no teclado numérico: − blackout, + parar tudo, Enter tap do BPM. Valem com o console na tela, também na edição.
    - Console: Alt + E liga e desliga a edição; Alt + I mostra ou esconde as dicas de atalho (as etiquetas das teclas nos elementos e na seção master), escolha guardada em cada aparelho. O botão das dicas fica ao lado do "Editar". Funcionam também com o foco num campo, porque não escrevem nada.
    - **Edição do console (2026-10-20, pedido do usuário):** com elementos escolhidos, Delete os remove; Ctrl + D os duplica; Ctrl + C copia e Ctrl + V cola (nesta página e formato, no primeiro lugar livre; vale também em outra página ou formato). As setas os deslocam 2 células; com Shift, 4 (o dobro); com Ctrl, 1 célula, o menor passo da grade. O deslocamento segue as regras do arrasto (não sai da grade nem sobrepõe, e entra ou sai de molduras) e repete com a tecla apertada. Com a grade de 2026-10-20 (células pela metade), o Ctrl + seta anda meia célula de antes. Valem com o console em edição e sem foco num campo de texto nem diálogo aberto. Ctrl + D, Ctrl + C, Ctrl + V e Delete não são reservados: em operação, os elementos podem usá-los.
    - **Desfazer e refazer (2026-10-17):** Ctrl + Z desfaz e Ctrl + Shift + Z refaz qualquer alteração, em todas as telas e menus. Com o console na frente e em operação, as duas combinações vão para os elementos (que podem usá-las); na edição do console, desfazem e refazem.
    - Essas teclas e combinações são reservadas: nenhum elemento pode usá-las. O Ctrl + Z e o Ctrl + Shift + Z não são reservados. O teclado numérico tem nome próprio ("Num 1", "Num -"), então as teclas da fileira de cima das letras (−, + e Enter inclusive) continuam livres para os elementos.
  - **Entradas externas por elemento:** o controle MIDI (com "aprender") ou a tecla do teclado que aciona um elemento do console é definido na edição do próprio elemento, como no QLC+. Não há tela separada de mapeamento MIDI.
  - **Configurações (2026-10-16):** num painel lateral aberto pelo ⚙, sobre a tela atual, para ajustar algo sem sair de uma criação ou edição. Não são uma tela navegável: não entram nos menus, no modular nem em janela externa.
  - **Onde se programa:** as ferramentas de atributo (intensidade, cor, posição, efeitos) ficam no editor de cena do Studio, como no QLC+ 5. Não há aba "Programador".
    - No editor de cena, o toque escolhe **uma fixture** para receber os valores. **Ctrl + clique** (ou **toque longo**, no celular) soma outras **do mesmo tipo** (mesmo modelo e modo), e o que se mexe vale para todas as escolhidas. Os grupos de canais da cena mexem em várias de uma vez: o fader do grupo grava nos canais das fixtures do grupo que estão na cena (não inclui outras).
    - A aba **Canais** vem primeiro. Intensidade, Cor, Posição e Efeitos (shutter/strobe, gobo, prisma, zoom, foco) só aparecem quando a fixture tem esse tipo de canal.
    - "Adicionar fixtures" é o primeiro botão da fileira de fixtures e abre uma lista com nome, universo e canais de cada uma, com os atalhos "Desmarcar todos", "Selecionar do mesmo tipo" (o toque vale para todas do mesmo modelo) e "Selecionar grupo" (inclui as fixtures de um grupo de fixtures).
    - Os grupos de canais começam desligados. Ligar um grupo traz para a cena todas as fixtures dele; para tirar uma, usa-se "Adicionar fixtures".
    - Fade in e fade out ficam no fim do editor de cena: no show, o console ajusta os fades ao vivo (VC-3).
  - **Celular:** Console · Studio · Mesa simples · Fixtures · Mais (com os demais menus). O ⚙ fica na barra de cima.
    - **Tela cheia (2026-10-16):** no celular e no tablet, um botão na barra de cima põe o app em tela cheia no navegador, como nos apps das mesas digitais. Só aparece onde o navegador permite (o iPhone não permite).
  - A **seção master** (GM, Blackout, Stop All) fica só no Console. Exceções (2026-10-16):
    - **botão de pânico na barra superior**, em todas as telas: um toque para todas as criações; dois toques seguidos também zeram a Mesa simples inteira;
    - **"Parar todas" no Studio**, que para só as criações.
- **E13. Interface modular (opcional).**
  - Painéis encaixáveis e redimensionáveis, como no VS Code ou no encaixe de janelas do Windows.
  - Qualquer painel pode ir para uma **janela externa do Windows**, para usar outro monitor (ex.: 3D View num monitor dedicado). A janela externa é a mesma tela conectada ao motor como mais um cliente, então continua funcionando mesmo com a janela principal minimizada.
  - O app **começa no modo simples** (uma tela por vez). O modular é ativado opcionalmente. No celular, é sempre o simples.
- **E7. Android como host USB fica para a v2** (Tauri Android + plugin Kotlin com usb-serial-for-android). Na v1 o celular acessa pelo navegador na rede local.
- **E8. Referência do QLC+** clonada em `reference/qlcplus`, fixada na tag `QLC+_5.3.0` e fora do git do projeto.
  - Essa árvore contém a UI v4 (`ui/`), a UI v5 (`qmlui/`) e o `engine/` compartilhado.
  - Só leitura: nunca compilar nem executar código de lá.

## Arquitetura

- **A1. Threads.**
  - Uma thread de motor a 50 Hz com deadline. Ela é dona do estado de runtime e não usa locks no caminho quente.
  - Os comandos chegam por canal e são drenados a cada tick.
  - Cada universo publica o frame mais recente. Cada interface DMX tem sua própria thread, que lê esse frame no próprio ritmo: Enttec Pro a 44 Hz; Open DMX a 30 Hz com break de 110 µs e MAB de 16 µs.
  - Se o motor falhar, as saídas repetem o último frame.
  - **Interface desplugada (2026-10-17):** a interface perde a alimentação e a linha DMX fica sem sinal. O que cada aparelho faz (apagar ou manter o último estado) é configuração dele, não do app. Ao replugar, a saída volta com o estado atual.
- **A2. Estado.**
  - O projeto tem um único escritor. Cada operação tipada é validada, incrementa a revisão e é difundida a todos os clientes.
  - O servidor é a autoridade. A UI só é otimista no controle que está sob o dedo.
  - **Edição simultânea (2026-10-17):** a última gravação vale, em tudo (inclusive no palco). Não há trava nem mescla.
  - **Estado no servidor (2026-10-17):** a partir da F3, todo o estado fica no servidor e o motor de mentira sai da UI. Até a F6, rodar uma criação não acendia nada. Desde a F6 (2026-10-18), as cenas acendem; desde a F7 (2026-10-18), os chasers, as sequências e as coleções também; desde a F8 (2026-10-19), o EFX. O RGB Matrix acende a partir da F9.
  - **Sem salvar até a F5 (2026-10-17):** o servidor abre com o show de exemplo; "Novo projeto" funciona, Abrir e Salvar ficam desligados. O que se muda se perde ao fechar o app.
  - **Projetos (2026-10-17, F5):**
    - Arquivos `.rlc` numa pasta só, `Documentos\R Light Control\`, salvos pelo nome e abertos pela lista, também no celular. Salvar guarda a versão anterior em `.rlc.bak`.
    - Ao iniciar, o app abre a recuperação; senão, o último projeto; senão, um projeto em branco. O show de exemplo fica gravado na pasta na primeira vez.
    - Com alterações não salvas, uma cópia de recuperação é gravada a cada mudança (no máximo 1 por segundo). Fechar sem salvar não perde nada: não há a pergunta "Salvar antes de sair?".
  - **Prévia (2026-10-17, F5):** o editor de cena é o programador (E12). O botão "Prévia" manda a cena em edição para a luz; começa desligado ao abrir a cena (como no QLC+ 5) e desliga ao sair do editor. Uma prévia por vez, para todos os aparelhos.
    - Camadas da saída, de baixo para cima: valor padrão, criações rodando (F6), prévia, Mesa simples, localizar, Grand Master e blackout. A prévia fica por cima das criações e mostra a cena exatamente como está.
  - **Mistura das criações (2026-10-17, F6):**
    - HTP como no QLC+: o dimmer e as cores (de todas as fixtures) misturam pelo maior valor. Azul e vermelho juntos dão magenta. Para uma cor por vez, a moldura solo.
    - LTP no resto: vence a criação iniciada por último.
    - Ao parar, no fade out: a intensidade desce até 0, e o LTP volta ao que está embaixo (outra criação rodando ou o valor padrão). O QLC+ mantém o LTP onde está; a regra fica num lugar só do motor, para voltar ao jeito dele se o usuário preferir.
    - Fades lineares em inteiros, como no QLC+, a partir do valor que está saindo. Rodas de cor e de gobo, prisma, strobo/shutter, macros, efeitos, velocidades, manutenção e "outros" não fazem fade: mudam no começo do fade in e no fim do fade out.
    - Canal com fino na fixture: fade em 16 bits, com o fino em 0 quando a cena não o tem.
    - Editar uma cena rodando vale na hora, sem fade e sem reiniciar.
    - Stop All e "Parar todas" usam o fade out de cada criação. O pânico (já no primeiro toque), o "Novo" e o "Abrir" cortam na hora.
    - Grand Master só no modo "reduzir a intensidade". Os modos "limitar" e "todos os canais" do QLC+ ficam de fora.
  - **Chasers, sequências e coleções (2026-10-18, F7):**
    - O motor roda os chasers (tempo dos passos, ordem e batidas), como o `ChaserRunner` do QLC+.
    - O passo dura o fade in mais o hold. Na troca, o passo que sai faz o fade out enquanto o próximo faz o fade in (crossfade), partindo do valor que está saindo.
    - Os tempos do passo (Comum ou Por passo) valem para tudo o que ele dispara; no Padrão, cada criação usa os seus. No hold, "Padrão" vale como "Comum", como no QLC+.
    - **Fim do Single Shot:** opção de cada chaser e sequência: apagar depois do último passo (o padrão, como no QLC+) ou manter o último passo aceso até alguém parar o chaser.
    - Só aparece rodando o que o usuário disparou (o chaser, a coleção). O que eles disparam não acende o LED do botão nem do Studio, e roda à parte: uma cena pode rodar sozinha e dentro de um chaser ao mesmo tempo, sem uma mexer na outra.
    - Uma criação não contém a si mesma (direto ou por outra), como no QLC+.
    - **BPM:** o motor gera as batidas; o tap recomeça a batida no último toque, que conta como uma batida. Num chaser em batidas, os passos trocam sempre numa batida, ou numa fração dela quando o passo é menor que uma batida (2026-10-19; o QLC+ só conta batidas inteiras).
  - **EFX (2026-10-19, F8):**
    - O motor anda cada fixture pelo desenho (as contas do QLC+), e o ponto vira pan e tilt, em 16 bits com o fino. O que passa do curso fica na ponta. O EFX só mexe no pan e no tilt; a luz vem de uma cena.
    - Cada fixture tem um fader da mistura, como uma cena: no fade in, sai da posição em que está e entra no desenho; ao parar, volta no fade out ao que está embaixo (a decisão 2 da F6). No QLC+, o desenho cresce do centro, e não há fade out.
    - **Relativo:** o desenho é somado ao que está embaixo, com o zero em 127: as criações rodando, em qualquer ordem de início, e a prévia (o programador). A camada fica entre a prévia e a Mesa simples.
    - **Propagação:** "Defasadas" põe cada fixture 1/n da volta à frente da anterior (com 4, 90°, como a prévia do QLC+; o motor do QLC+ usa 1/(n+1)). "Em sequência", cada uma começa o desenho do início 1/n da volta depois da anterior e, até a vez dela, fica onde está.
    - Mudar a duração da volta (no editor, no elemento "Tempos" ou no BPM, com a volta em batidas) continua do ponto onde está, sem salto. "Uma vez" anda uma volta e termina; "Vai e volta" vira o sentido a cada volta, sem salto, também na linha.
  - **Localizar (2026-10-17, F5):** acende as fixtures escolhidas (dimmer e RGBW a 100%, shutter no padrão, pan e tilt parados), sem gravar na cena.
  - **Desfazer (2026-10-17, F5):** um histórico só, no servidor, para todos os aparelhos (100 passos). Entram o projeto e a biblioteca; a operação ao vivo, as preferências de cada aparelho, a rede local, o PIN e o tipo da interface ficam de fora. "Novo" e "Abrir" começam um histórico novo. Só pelo teclado por enquanto.
  - **Blackout (2026-10-17):** como no QLC+, zera só os canais de intensidade (dimmer e cores) e mantém o resto (pan, tilt, gobo), para os moving heads não se mexerem. Canais sem fixture contam como intensidade.
  - **Valor padrão do canal (2026-10-17):** cada universo parte dos valores padrão das fixtures patchadas (ex.: o Strobo do Beam 7R em 255). A Mesa simples fica por cima, e o canal solto (inclusive pelo "Zerar") volta ao padrão.
  - **Estado ao vivo não é salvo (2026-10-17):** criações rodando, Mesa simples, valores dos elementos do console e seção master ficam fora do projeto. Ao reabrir, tudo começa parado e zerado, como no QLC+.
  - **Preferências de cada aparelho (2026-10-17):** zoom, formato do console, dicas de atalho, qualidade e fumaça do 3D ficam no navegador de cada aparelho, fora do projeto e do contrato.
- **A3. Protocolo WebSocket.**
  - Endpoint único `/ws`, com envelope JSON para comandos e eventos.
  - Frames binários para dados de alta taxa (universos, preview).
  - Snapshot completo ao (re)conectar.
  - **Avisos de mudança (2026-10-17):** o `evt` leva o pedaço do estado que mudou, não a operação aplicada. Assim a UI não repete as regras do motor.
  - A conexão só passa a exigir PIN na F11 (PIN → token).
  - Detalhes em `docs/protocolo.md` (a partir da F2).
- **A4. IDs.**
  - UUIDv7 nas entidades salvas. **Quem gera (2026-10-17):** a UI, para já escolher o que acabou de criar; o servidor confere se é único. A UI tem gerador próprio, porque `crypto.randomUUID` não existe no celular em HTTP.
  - Definições de fixture embutidas no projeto (slug + hash), para que um projeto nunca quebre por falta de definição.
- **A5. Arquivos.**
  - JSON com `{format, schemaVersion}`. Ao carregar, uma cadeia de migrações converte versões antigas.
    - **Versão 2 do projeto (2026-10-18, F7):** a opção "manter o último passo" dos chasers e sequências. A versão 1 abre com ela desligada (apagar no fim).
    - **Versão 3 (2026-10-19, F7):** a chave "Seguir o BPM" do elemento "Tempos". As versões anteriores abrem com ela ligada.
    - **Versão 4 (2026-10-19, F8):** o EFX sem a ordem aleatória e com a largura e a altura até 127. O EFX em "Aleatória" abre em "Repetir", e o tamanho acima de 127 abre em 127.
    - **Versão 5 (2026-10-20):** a grade do console com o dobro de colunas. As posições e os tamanhos dos elementos dobram, e eles ficam do mesmo jeito na tela.
  - Escrita atômica (arquivo temporário + rename) e `.bak`.

## Registro

| Data | Decisão |
|---|---|
| 2026-10-15 | Plano inicial aprovado (G1–G7, E1–E8, A1–A5). |
| 2026-10-15 | F0 validada. `docs/padroes-codigo.md` e o padrão de idioma (G6) aprovados. Porta padrão do servidor: 8090 (configurável em `settings.json`). |
| 2026-10-15 | Decisões da F1: visual skeuomórfico da Ui24R e só tema escuro (E6), fader relativo com botão FINO (E9), valores em % com opção de DMX (E10). |
| 2026-10-16 | Ajustes pedidos na validação da F1: roda do mouse em fader e knob (E9); botão com segundo modo de destaque, `highlight="led"` (só o filete acende e o botão fica afundado). |
| 2026-10-16 | Cores semânticas vivas no sistema todo (E6): matiz pura com saturação e brilho máximos. O filete de LED aceso usa a cor pura com brilho; o apagado fica a 30%. |
| 2026-10-16 | F1 validada: 60 fps no celular e 100 no desktop. Decisões: aparência e comportamento dos botões do VC configuráveis por elemento (E11, pedido VC-1); os testes de toque automatizados ficam fora do projeto por enquanto. |
| 2026-10-16 | F1 commitada. Decisões da F2: menus Console, Fixtures, Studio, Patch, 3D View e Mesa simples (E12); seção master só no Console (E12); interface modular opcional com janelas externas (E13); console com layouts por formato de tela (E11, pedido VC-2); protótipo entregue uma rodada por vez. |
| 2026-10-16 | Mini-plano da F2 aprovado: menus Monitoramento e In/Out (E12); ferramentas de atributo no editor de cena do Studio (E12); começa no modo simples (E13); navegação do celular como proposta. |
| 2026-10-16 | Ajustes pedidos na validação da rodada 1 da F2: grupos de canais com ícone por atributo (E12); abas agrupadas num trilho, como controle segmentado, no lugar de botões soltos (design system). |
| 2026-10-16 | In/Out sem abas, com as seções Saídas e Entradas; o mapeamento de MIDI e de teclado fica em cada elemento do console (E12). |
| 2026-10-16 | Rodada 1 da F2 aprovada e commitada (com arrastar fixtures na grade do Patch). |
| 2026-10-16 | Editor de cena: uma fixture por vez, várias pelos grupos de canais; aba Canais primeiro e as de atributo só quando fazem sentido; lista de fixtures com universo e canais (E12). |
| 2026-10-16 | Sem descrições na tela: explicações só no ícone "i" ao lado do título (E6, `RlInfo`). O fader de grupo da cena só mexe nas fixtures da cena. |
| 2026-10-16 | Editor de cena reorganizado: "Adicionar fixtures" com atalhos (desmarcar todos, mesmo tipo, grupo), divisor antes dos grupos de canais, e ligar um grupo de canais traz as fixtures dele para a cena (E12). |
| 2026-10-16 | Editor de cena: multisseleção de fixtures do mesmo tipo com Ctrl + clique ou toque longo; o que se mexe vale para todas as escolhidas (E12). `RlButton` ganha `@longpress`. |
| 2026-10-16 | Aba "Beam" do editor de cena renomeada para "Efeitos" (vale para PARs e moving heads) (E12). |
| 2026-10-16 | Studio: lista agrupada por tipo, sem filtro de tipo, com pastas por tipo e arrastar entre pastas (E12). |
| 2026-10-16 | Studio: coluna da lista mais larga (360 px), botão "Parar todas" (exceção ao Stop All só no Console) e coleção nova a partir das criações rodando (E12). |
| 2026-10-16 | Fades no fim do editor de cena. Pedido VC-3: fades ajustados ao vivo no console (G4, PLANO). |
| 2026-10-16 | Botão de pânico na barra superior: 1 toque para as criações; 2 toques também zeram a Mesa simples (exceção ao Stop All só no Console, E12). Abaixo de 1536 px a barra esconde o nome R·LIGHT e o texto USB. |
| 2026-10-16 | Mesa simples: em tela larga, canais em cima e grupos de canais embaixo; no celular, grupos numa aba (E12). |
| 2026-10-16 | Mesa simples: o fader de grupo grava nos canais do grupo e os faders de canal acompanham (E12). |
| 2026-10-16 | Mesa simples sem páginas: os 512 canais numa faixa, e Ctrl + roda do mouse rola a faixa (E12, design system). |
| 2026-10-16 | Rodada 2 da F2 aprovada e commitada. |
| 2026-10-16 | VC-2: elementos compartilhados entre os formatos (só a posição muda), três formatos fixos pela largura, com troca no ⚙ (E11). VC-3: elemento "Tempos" com a lista das criações que controla (E11). |
| 2026-10-16 | Rodada 3 da F2 entregue: Console (operação, edição, VC-1, VC-2 e VC-3) e 3D View. |
| 2026-10-16 | Edição do console: o painel de propriedades fica sempre reservado, para o layout não mudar ao escolher um elemento (design system). |
| 2026-10-16 | Console: grade com o dobro de colunas (16, 32 e 48); paleta dos elementos com versões escuras e cinzas; Texto com fundo e borda (E11). |
| 2026-10-16 | Console: botões sem ícone por padrão; ícone opcional escolhido na biblioteca inteira, com busca (E11, VC-1). O Texto pode ter 1 célula de altura. |
| 2026-10-16 | Console: o elemento acionado por tecla mostra a tecla numa etiqueta no canto, na operação e na edição (design system). |
| 2026-10-16 | Zoom geral da interface no ⚙ (zoom nativo da janela do app desktop; IPC liberado só para isso) e zoom do console ao lado do "Editar" (E2, E10). |
| 2026-10-16 | 3D View: "Editar posição" permite mover as fixtures em todas as vistas, não só no topo; desligado, o arrasto só mexe a câmera (E12). |
| 2026-10-16 | Atalhos padrão: Ctrl + −/+ para o zoom (o do console quando ele está na frente) e −, + e Enter do teclado numérico para blackout, parar tudo e tap; teclas reservadas, com o teclado numérico separado da fileira de cima (E12). |
| 2026-10-16 | 3D View: aba Elementos com elementos de palco (truss, caixas, pessoa, praticável, painel LED e decorativos), Ctrl + D duplica ao lado e Del exclui; cada aba move só o que é dela (E12). |
| 2026-10-16 | 3D View: a câmera vai para o botão do meio do mouse (girar ou deslocar) e o zoom fica só na roda; o botão esquerdo só escolhe (E12). |
| 2026-10-16 | Barras de rolagem no padrão do design system, para o app todo: trilho afundado e cursor em relevo, sem setas (design system). |
| 2026-10-16 | Configurações num painel lateral aberto pelo ⚙, sobre a tela atual, no lugar da tela navegável (E12). |
| 2026-10-16 | Console: botão de dicas de atalho ao lado do "Editar", que mostra ou esconde as etiquetas das teclas; atalhos Alt + E (editar) e Alt + I (dicas), reservados (E12). |
| 2026-10-16 | Rodada 3 da F2 aprovada e commitada. |
| 2026-10-17 | Revisão do contrato v0 e do protocolo: a última gravação vale; IDs UUIDv7 gerados pela UI e conferidos pelo servidor; `evt` com o pedaço do estado que mudou; estado ao vivo não é salvo; preferências por aparelho fora do contrato; PIN só na F11 (A2–A4). |
| 2026-10-17 | Celular: o zoom de 100% passa a ser 75% da escala do navegador; os outros degraus seguem a mesma proporção (E10). |
| 2026-10-16 | Celular e tablet: botão de tela cheia na barra de cima e zoom da interface no ⚙ (pela largura do viewport) (E10, E12). |
| 2026-10-17 | Mini-plano da F3 aprovado: todo o estado no servidor; sem salvar até a F5; só o driver D2XX; blackout como no QLC+ (A2, E4). Interface do usuário: genérica, nome USB "Numbstar", tratada como Open DMX. |
| 2026-10-17 | F2 validada e concluída, com as telas congeladas em `docs/telas/`. Início da F3 (estudo das referências e mini-plano). |
| 2026-10-17 | Rodada 1 da F3 entregue: servidor como autoridade, motor a 50 Hz e saída Dummy. Controles otimistas sob o dedo (A2); frame binário com o id do universo (A3). |
| 2026-10-17 | Rodada 1 da F3 validada pelo usuário. |
| 2026-10-17 | Rodada 2 da F3: tipo da interface pelo nome, pela pergunta da Pro e à mão no In/Out (guardado por interface); D2XX carregada do driver instalado. Show de exemplo com o rig do usuário (12 ParLed 18x15w e 4 Beam 7R) num universo na Numbstar (E4). |
| 2026-10-17 | Importação de `.qxf`: continua simulada até a F4 (sem recusa nem conversão à mão). O usuário vai importar o Strobo 1000w ele mesmo, pelo app, na F4. |
| 2026-10-17 | F3 validada e concluída com a Numbstar e os ParLed. O teste do instalador num PC limpo fica para a F16. Sem sinal (interface desplugada), apagar ou manter o último estado é configuração de cada aparelho (A1). |
| 2026-10-17 | Mini-plano da F4 aprovado: a biblioteca do QLC+ nunca é embutida (fixtures só importadas ou criadas à mão); biblioteca do usuário gravada no PC; valor padrão do canal aplicado na saída; cor da faixa pelo nome; Excluir e importar vários `.qxf` de uma vez na Biblioteca (E3, A2). |
| 2026-10-17 | F4 entregue: importador `rlc-qxf`, biblioteca gravada no PC, valor padrão na saída. As 1.781 fixtures do QLC+ 5.3.0 importam sem recusa (E3). |
| 2026-10-17 | F4 validada pelo usuário e concluída. |
| 2026-10-17 | Pedido do usuário: desfazer e refazer (Ctrl + Z e Ctrl + Shift + Z) em todas as telas, na F5, menos com o console em operação (E12). Sai do backlog da v2. |
| 2026-10-17 | Mini-plano da F5 aprovado: projetos numa pasta só, recuperação automática, prévia desligada ao abrir a cena, Mesa simples acima da prévia, localizar e desfazer com histórico único (A2, E12). Sem projeto anterior, o app abre em branco. |
| 2026-10-17 | F5 entregue: projetos em arquivo, recuperação, prévia, localizar e desfazer. A recuperação é gravada no segundo seguinte à mudança, e a lista de projetos é relida a cada segundo (A2, A5). |
| 2026-10-17 | F5 validada pelo usuário e concluída. Início da F6 (estudo das referências e mini-plano). |
| 2026-10-17 | Mini-plano da F6 aprovado: cores em HTP como no QLC+ (moldura solo para uma cor por vez); ao parar, o LTP volta ao que está embaixo; rodas e gobo sem fade; prévia acima das criações; Stop All com fade out e pânico na hora; Grand Master só "reduzir" (A2). |
| 2026-10-18 | F6 entregue: cenas misturadas no motor (HTP/LTP), fades em inteiros e em 16 bits, edição ao vivo. O pânico corta já no primeiro toque (`panic { all }`) (A2, A3). |
| 2026-10-18 | Pedido na validação da F6: o elemento "Tempos" (VC-3) passa a controlar o fade in e o fade out das cenas da lista já na F6, sem mexer nos tempos gravados (E11). |
| 2026-10-18 | F6 concluída a pedido do usuário, com 8 itens validados; os não testados (posição entre cenas, fades nos Beams, tick com 4 universos e o elemento "Tempos") serão ajustados depois, se precisar. Início da F7 (estudo das referências e mini-plano). |
| 2026-10-18 | Mini-plano da F7 aprovado: o motor roda os chasers, crossfade como no QLC+, só o que o usuário disparou aparece rodando, Tempos em todos os passos, cue list ligada já na F7, sem criação que contém a si mesma, hold "Padrão" = "Comum" (A2, E11). Pedido do usuário: o fim do Single Shot é uma opção do chaser (apagar ou manter o último passo). |
| 2026-10-18 | F7 entregue: chasers, sequências e coleções acendem; batidas do BPM no motor; cue list ligada ao chaser; projeto na versão 2 (A5). Um chaser sem passos fica rodando sem acender, e "anterior" no primeiro passo de um Single Shot não faz nada. |
| 2026-10-19 | Pedido na validação da F7: o elemento "Tempos" ganha "Seguir o BPM" (ligada por padrão): tempos em batidas do BPM master, TAP único com o master, chasers da lista nas batidas, passos de fração de batida no tempo (E11, VC-3; projeto na versão 3, A5). |
| 2026-10-19 | F7 concluída a pedido do usuário, sem a validação (exceção ao G1): os itens dela e os da F6 que ficaram serão testados junto com a F8. Início da F8 (estudo das referências e mini-plano). |
| 2026-10-19 | Mini-plano da F8 aprovado: o EFX entra e sai como as cenas; o relativo desenha em volta das criações e da prévia; defasagem de 1/n da volta; editor de EFX sem "Aleatória", com largura e altura até 127 e grupos em "Adicionar fixture" (A2, E12). |
| 2026-10-19 | F8 entregue: EFX no motor, nos chasers, nas coleções e no elemento "Tempos"; grupos de fixtures conferidos no servidor; projeto na versão 4 (A2, A5). |
| 2026-10-19 | Pedido do usuário: os testes com os moving beams vão para um plano de validação final (`docs/validacao-final.md`), antes do release (G1). Para lá foram os da F6 (a posição entre cenas e os fades nos Beams), o deslizar dos Beams no chaser da F7 e os do EFX da F8. |
| 2026-10-19 | F8 commitada a pedido do usuário, antes da validação. O que falta validar (com e sem os moving beams) está nas Pendências do `PLANO.md`; o que ficou de fora da F7 e da F8, no Backlog (só se o usuário pedir). |
| 2026-10-19 | 3D View: botão "Só o 3D", que esconde o painel de fixtures, elementos e palco e deixa a visualização na tela toda (E12, pedido do usuário). |
| 2026-10-20 | Pedidos do usuário: reordenar (arrastando) e duplicar grupos de fixtures e de canais; pastas dentro de pastas no Studio; atalhos de teclado na edição do console (Delete, Ctrl + D, Ctrl + C, Ctrl + V, setas com Shift e Ctrl) (E12; `moveFixtureGroup`, `moveChannelGroup` e `parent` na pasta no protocolo). |
| 2026-10-20 | Pedido do usuário: posicionamento mais fino no console. A grade passa a ter 32, 64 e 96 colunas (o dobro), com tamanhos mínimos e iniciais dobrados; projeto na versão 5, que dobra as posições dos elementos (E11, A5). |
| 2026-10-20 | F7 e F8 validadas pelo usuário, sem os Beams. Foram para a validação final, por escolha dele, o tick com 4 universos (F6), a sequência e a coleção (F7); a F8 foi concluída (G1). |

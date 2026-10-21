# Ajustes rápidos: Localizar com as outras mais fracas e fader de BPM

**Status:** mini-plano aprovado em 2026-10-21, com as duas decisões como recomendado (a intensidade das outras ajustável no ⚙, com padrão de 20%; o BPM num knob no celular e no tablet). Em implementação. Dois pedidos seus (2026-10-21), da pesquisa da grandMA ([pesquisa-grandma.md](../pesquisa-grandma.md): P6 e E3). Vêm antes da F10.

**Entregue e validado por você em 2026-10-21.**

**Entregue**
- **Motor (`rlc-engine`):**
  - o `Highlight` leva os valores, os canais das fixtures localizadas e a intensidade das outras;
  - a intensidade (dimmer e cores) das outras cai pela conta do Grand Master, depois do Localizar e antes do Grand Master e do blackout;
  - o comando `Tempo` muda o BPM sem recomeçar a batida.
- **Servidor:**
  - `setHighlight` com `others`, e o estado `highlight` com as `fixtures` e o `others`;
  - a operação nova `setTempo`, com os mesmos limites do `setBpm` (30 a 300).
- **Telas:**
  - o fader de BPM ao lado do GM na coluna master, com a escala em BPM e o FINO de 1 BPM;
  - o knob de BPM na faixa do celular;
  - "Localizar: intensidade das outras" no ⚙ (padrão 20%, de 0 a 100%, em % ou DMX);
  - o `RlFader` ganhou `marks` e `fineStep`.
- **Diferença do plano:** o `setBpm` é o tap e recomeça a batida a cada chamada. Isso faria os chasers em batidas engasgarem enquanto o fader é arrastado. O fader usa uma operação própria, `setTempo`, que muda a velocidade sem recomeçar a batida (a parte da batida que falta continua a mesma). O tap segue como era.
- **Testes:**
  - automáticos:
    - no motor: as outras a 20%, a 0% e a 100%, a localizada inteira (também os canais que o Localizar não mexe), o pan e o strobo iguais, o Grand Master por cima, o Localizar desligado; o `Tempo` mantendo a fase da batida, e o tap recomeçando;
    - no servidor: todos os canais do Beam vão ao motor, o `others` chega, e o tap e o fader ficam entre 30 e 300;
  - de ponta a ponta, contra o `pnpm dev` com o seu "RSena Palco" (a Numbstar estava desplugada, nada acendeu de verdade):
    - pelo WebSocket: a ParLed localizada a 255 e as outras (ParLed e Beam) a 40 de 200; a 0% e a 100%; o Grand Master a 50% por cima; a roda de cor e o pan do Beam sem mudança; o Beam localizado; o pânico desligando;
    - no navegador (desktop): os dois faders, a escala do BPM, o arrasto, a roda (2,7 BPM; com FINO, exatamente 1), o toque duplo (120) e o TAP movendo o fader;
    - no celular (390, 360 e 320 px): a faixa cabe sem rolagem, e o knob muda o BPM; no tablet (768 px), a coluna master;
    - na tela Fixtures: o Localizar liga com 20%, o ⚙ muda para 25% e 0% na hora, o toque duplo volta a 20%;
  - `pnpm check` passa.
  - Depois do teste, o `settings.json`, a recuperação, a biblioteca e os projetos ficaram iguais aos de antes, e o projeto aberto não mudou.
- **Ficou para depois:** refazer os prints congelados da seção master e do ⚙ em `docs/telas/`. Eles foram tirados com o show de exemplo, e abri-lo no servidor apagaria a sua cópia de recuperação (o trabalho não salvo do "RSena Palco"). Dá para fazer depois que você salvar o projeto.

## Objetivo

Depois deste passo:
- ligar o **Localizar** deixa a(s) fixture(s) escolhida(s) em 100% e **todas as outras com a intensidade reduzida** (20% do que estavam dando, por padrão), para ver qual é qual no palco;
- a seção master do console ganha um **fader de BPM**, ao lado do TAP, com o botão FINO.

## O que já existe

- **Localizar (F5):** `setHighlight` com a lista de fixtures. O servidor calcula os valores (`highlight_levels`: dimmer e RGBW a 100%, shutter no padrão, o resto parado) e o motor os põe por cima da Mesa simples, antes do Grand Master e do blackout. Os dois botões (editor de cena e Fixtures) usam `useLocate`.
- **BPM (F7):** `master.bpm`, inteiro de 30 a 300, mudado por `setBpm`. A seção master mostra "N BPM" e o TAP (`tapBpm`), sem fader. `RlFader` tem o FINO embutido; `RlKnob` não tem.
- **Blackout:** já separa os canais de intensidade (dimmer e cores) dos outros, e a mesma separação serve para reduzir as "outras".

## O que entra

### 1. Localizar com as outras mais fracas

- **Motor:** o `Highlight` ganha o nível das outras e a lista de todos os canais das fixtures localizadas. Com o nível menor que 100%, o motor reduz os canais de intensidade (dimmer e cores) de todo o resto pela mesma conta do Grand Master (`round(valor × nível / 255)`), depois do Localizar e antes do Grand Master e do blackout. Pan, tilt, gobo e o resto das outras ficam como estão, como no blackout. Valem as criações rodando, a prévia e a Mesa simples.
- **Servidor:** `setHighlight` leva o nível (`others`, 0 a 255) e o guarda junto da lista, para reenviar ao motor quando o patch ou as criações mudam. A lista das fixtures localizadas continua igual para os aparelhos.
- **Tela:** o nível é uma preferência do aparelho, no ⚙ Configurações ("Localizar: intensidade das outras", em % ou DMX, como o resto, E10). Padrão 20%. Com **0%**, vira o "solo" da grandMA: só a localizada acende.
- Os dois botões Localizar mandam o nível do aparelho.

### 2. Fader de BPM na seção master

- **Faixa:** 30 a 300 BPM, a do servidor e a do tap. O tap continua do mesmo jeito, e o fader acompanha o BPM de qualquer origem (tap, outro aparelho).
- **Desktop (coluna master):** o fader do BPM fica ao lado do Grand Master, cada um com o seu FINO. A coluna alarga um pouco, e o resto (blackout, parar tudo, TAP) continua embaixo.
- **Celular e tablet (faixa em cima):** um knob de BPM, como o do GM, ao lado do número e do TAP.
- **`RlFader`:** ganha duas opções, usadas só aqui: a escala com os valores certos do BPM (60, 120, 180, 240 e 300, no lugar dos %) e o passo do FINO na roda e nas setas, que no BPM é de 1 (o BPM é inteiro: sem isso o FINO não mudaria nada).
- **BPM guardado:** inteiro, como hoje. O ajuste mais fino é de 1 BPM.

### 3. Servidor, contrato e protocolo

- `Op::SetHighlight` ganha `others`; a regra e o teste ficam em `apply.rs`. Os tipos do TypeScript são regenerados (`cargo test -p rlc-core`).
- `docs/protocolo.md` com a operação nova.

### 4. Telas (G2)

Muda a seção master (tela congelada da F2) e o painel ⚙. Atualizo `docs/design-system.md` (as duas opções do `RlFader`) e as telas em `docs/telas/`.

## Decisões para você

1. **Intensidade das outras no Localizar.**
   - **Ajustável no ⚙, padrão 20%** (recomendado): cada aparelho escolhe o seu, e 0% serve de solo.
   - **Fixa em 20%:** sem ajuste na tela, o valor fica no código.
2. **O BPM no celular e no tablet.**
   - **Knob, como o do GM** (recomendado): o mesmo desenho da faixa de cima. O knob não tem FINO, então o ajuste preciso fica com o TAP.
   - **Botões −1 e +1** ao lado do número: ajuste exato de 1 em 1, sem knob.

## Fica de fora

- Solo e "outras mais fracas" por **grupo** de fixtures. Hoje vale para as fixtures escolhidas, como o Localizar.
- Mudar o Grand Master ou o blackout.
- Vários masters de BPM nomeados (a outra parte do E3).

## Testes automáticos

- **Motor:**
  - com o nível em 20%, a intensidade das outras cai a 20% e as fixtures localizadas ficam em 100%;
  - pan, tilt e o resto das outras não mudam;
  - o nível 255 (100%) não muda nada, e o 0 deixa só as localizadas acesas;
  - o Grand Master e o blackout continuam valendo por cima;
  - o Localizar desligado devolve tudo ao normal;
  - vale sobre criações rodando e sobre a Mesa simples.
- **Servidor:** a lista dos canais das fixtures localizadas (todos os canais, não só os que o Localizar muda), `setHighlight` com o nível, o nível reenviado ao motor quando o patch muda.
- **UI:** `pnpm check` (typecheck e lint). No navegador, contra o seu `pnpm dev`: o Localizar com o nível mudando no ⚙; o fader de BPM, o FINO, o tap movendo o fader; a faixa no celular. Depois do teste, as edições são desfeitas.

## Validação (com você)

Não precisa dos Beams. Validado por você em 2026-10-21.
- [x] Localizar uma ParLed com uma cena rodando: ela fica em 100% e as outras a 20%. Desligar volta tudo.
- [x] Trocar a escolha: o foco acompanha.
- [x] Mudar a intensidade das outras no ⚙: vale na hora, e 0% deixa só a localizada acesa.
- [x] O Grand Master e o blackout continuam por cima.
- [x] Fader de BPM: arrasta, o FINO ajusta de 1 em 1, e o TAP move o fader.
- [x] Celular: a faixa do master mostra o controle de BPM sem apertar o resto.
- [x] `pnpm check` passa.

## Referências estudadas

- O código do projeto: `highlight_levels` e `SetHighlight` (`apply.rs`), `Command::Highlight` e o `reduce` do Grand Master (`rlc-engine`), `useLocate`, `ConsoleMaster.vue`, `RlFader.vue`, `RlKnob.vue`.
- grandMA3 (manual 2.4): Highlight, Lowlight e Solo (os nomes dos três aparecem no índice de novidades; a página que os define não foi lida) e o Speed Master, com o fader de velocidade (`masters_speed`).

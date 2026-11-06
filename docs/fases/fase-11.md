# F11 — Virtual Console II

**Status:** a parte do console foi decidida pelo usuário e entregue em 2026-11-06, para a validação com o rig montado antes de um evento no mesmo dia. A parte de segurança (papéis, PIN → token, checagem de Origin) ficou para depois, a pedido dele.

## Objetivo

Todos os elementos do console funcionando em operação, para validar o software inteiro com o rig.

## O que já existia

- Botões (liga/desliga, Flash, Temp, Só esta, blackouts, Stop All, Voltar e Pausar), sliders (nível, playback, submaster e velocidade), cue list, moldura e moldura solo, "Tempos" e Texto: todos ligados ao servidor desde a F10 e a F10c.
- O relógio (hora, cronômetro e contagem regressiva) roda em cada aparelho.
- O XY Pad e o fader do Animation só guardavam o valor, sem mexer na luz.

## O que entrou

### 1. XY Pad

- Leva o **pan (X)** e o **tilt (Y)** das fixtures da lista pelo curso inteiro, em 16 bits com o fino.
- A lista aceita **fixtures e grupos de fixtures** (rig flexível, N1): vale quem está no grupo na hora. Remover o grupo deixa no pad as fixtures que estavam nele.
- Funciona como um slider de nível: sem fade, conta da hora em que foi mexido (LTP) e fica abaixo da prévia, da Mesa simples e do Localizar. O pânico de dois toques o solta.
- Projeto na **versão 13**: o XY Pad ganha `groups` (vazio nos antigos).

### 2. Animation (RGB Matrix)

- O fader é o **playback** do RGB Matrix, como no QLC+: sair do zero liga, a altura é a intensidade e voltar a zero desliga, sem os fades gravados (como o slider de playback).
- O **Play** liga cheio. Desligar pelo Play também solta o fader.
- Os submasters alcançam o Animation, e o "Só esta" e o Stop All baixam o fader dele, como os sliders de playback.

### 3. Macro (E6)

- Elemento novo, com cara de botão: um toque executa uma **lista de ações em ordem**. Fica aceso enquanto roda, e outro toque a interrompe.
- Ações: ligar uma criação, parar uma criação, Go de um chaser ou de uma sequência (começa o parado; rodando, vai ao passo seguinte), Stop All, ligar ou desligar o blackout, mudar o BPM e esperar (em segundos).
- Roda no servidor (o aparelho só manda o toque), então continua mesmo se o aparelho cair.
- O pânico, o "Novo", o "Abrir" e um Stop All de fora da macro a interrompem. O Stop All de dentro dela, não.
- Aceita tecla e MIDI na entrada principal, como os botões.

## Decisões (do usuário, 2026-11-06)

1. XY Pad absoluto, com fixtures e grupos de fixtures.
2. Fader do Animation como playback.
3. As macros entram já, junto com o resto do console.

## Fica de fora

- A segurança da F11 (papéis, PIN → token, Origin): depois das Paletas (F12b), a pedido do usuário.
- O XY Pad relativo e com limites de curso.
- O cronômetro e a contagem regressiva compartilhados entre os aparelhos.

## Validação (com você)

- [ ] O XY Pad move os Beams pelo grupo "Moving heads" (_com os Beams: [validação final](../validacao-final.md), se não der hoje_).
- [ ] O fader do Animation liga, dosa e desliga um RGB Matrix; o Play liga cheio.
- [ ] Uma macro com espera: liga uma cena, espera 2 s, dá Go num chaser; um segundo toque a interrompe.
- [ ] Os outros elementos do console no rig montado: botões, sliders, cue list, moldura solo, "Tempos" e relógio.
- [ ] `pnpm check` passa.

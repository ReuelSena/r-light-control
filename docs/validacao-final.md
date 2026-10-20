# Validação final

Os testes que dependem dos moving beams (os 4 Beam 7R do Rig RSena). O usuário ainda não pode fazer testes com eles, então esses testes ficam todos para uma rodada só, antes do release (F16). A regra está no G1 do [DECISOES.md](../DECISOES.md) (2026-10-19).

**Status:** aguardando os moving beams.

Quando uma fase nova tiver um teste que depende dos moving beams (ex.: o "pan e tilt conferem com a fixture real" da F15), o teste sai do checklist da fase e entra aqui.

## Antes de começar

- O app (o `pnpm dev` ou o instalador) com o projeto do rig e a Numbstar no universo "Rig RSena".
- Duas cenas que posicionam os Beams em lugares diferentes, com o dimmer aceso e fade de 3 s.
- Um EFX com os 4 Beams, ou com um grupo deles. O EFX só mexe no pan e no tilt: rode junto uma cena com o dimmer dos Beams aceso.

## F6 — Cenas

Itens do [fase-06.md](fases/fase-06.md):
- [ ] Na posição dos Beams, vence a última cena iniciada; ao parar a última, eles voltam para a posição da anterior.
- [ ] Os Beams andam sem degraus nos fades de 3 s, e a roda de cor e o gobo trocam de uma vez.
- [ ] Parar a única cena de posição: os Beams voltam à posição padrão durante o fade out.

## F7 — Chasers

Item do [fase-07.md](fases/fase-07.md):
- [ ] No chaser de 8 passos com crossfade de 2 s, os Beams deslizam de uma posição para a outra.

## F8 — EFX

Itens do [fase-08.md](fases/fase-08.md):
- [ ] Círculo nos 4 Beams com 90° entre eles ("Defasadas", ou "Juntas" com Início 0°, 90°, 180° e 270°).
- [ ] EFX relativo em volta da posição de uma cena e da prévia: o desenho acompanha a posição que você mexe.
- [ ] Os desenhos com largura, altura, centro e rotação: o movimento bate com a prévia da tela.
- [ ] Fade in e fade out do EFX.
- [ ] "Em sequência", "Vai e volta", "Uma vez", "Inverter" e "Para trás".
- [ ] O EFX num passo de chaser e numa coleção, junto com a cena do dimmer.
- [ ] Elemento "Tempos" com "Seguir o BPM": a volta em batidas acompanha o TAP.

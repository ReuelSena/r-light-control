# Validação final

Os testes que dependem dos moving beams (os 4 Beam 7R do Rig RSena), que o usuário ainda não pode fazer, e os que ele deixou para o fim por escolha própria. Ficam todos para uma rodada só, antes do release (F16). A regra está no G1 do [DECISOES.md](../DECISOES.md) (2026-10-19).

**Status:** aguardando os moving beams. Os três itens que não dependem deles podem ser feitos antes.

Quando uma fase nova tiver um teste que depende dos moving beams (ex.: o "pan e tilt conferem com a fixture real" da F15), o teste sai do checklist da fase e entra aqui.

## Antes de começar

- O app (o `pnpm dev` ou o instalador) com o projeto do rig e a Numbstar no universo "Rig RSena".
- Duas cenas que posicionam os Beams em lugares diferentes, com o dimmer aceso e fade de 3 s.
- Um EFX com os 4 Beams, ou com um grupo deles. O EFX só mexe no pan e no tilt: rode junto uma cena com o dimmer dos Beams aceso.

## Adiados por escolha do usuário (2026-10-20)

Não dependem dos Beams: podem ser feitos nos ParLed e na Numbstar.
- [ ] **F6:** o tick abaixo de 2 ms com 4 universos. No Monitoramento, com um quarto universo criado no In/Out e cenas e chasers rodando. O usuário não pôde testar em 2026-10-20 (o teste automático mede 0,68 ms no app compilado, com 48 moving heads no EFX).
- [ ] **F7:** a sequência, com cada passo acendendo os seus valores.
- [ ] **F7:** a coleção, que dispara e para tudo junto, cada criação com os seus fades.

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

## F10b — EFX em grupo

Itens do [fase-10b.md](fases/fase-10b.md) (validados na tela e na prévia, sem os Beams):
- [ ] O EFX num grupo de Beams: tirar um Beam do grupo e o EFX seguir só com os outros.
- [ ] A "Defasagem" do grupo separa os Beams no movimento (ex.: 90° com 4, um atrás do outro).
- [ ] Um Beam do grupo incluído também sozinho, com "Inverter" e outro "Início", é a exceção: anda do jeito dele, no lugar dele no grupo.

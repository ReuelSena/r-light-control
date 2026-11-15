# Subgrupos de fixtures (G4, pedido do usuário, 2026-11-15)

Mini-plano, **aguardando aprovação** (G1).

Pedido: dentro de um grupo de fixtures, criar subgrupos para escolher só aquelas fixtures ao adicionar numa cena (e nas outras criações). Na tela Fixtures/Grupos, uma **árvore de grupos** mostra de imediato quais existem.

## Decisões (respondidas pelo usuário)

1. **Subgrupo = recorte do pai.** Só pode ter cabeças que estão no pai. Tirar uma cabeça do pai a tira dos subgrupos dele (e dos filhos deles).
2. **Sem limite de níveis**, como as pastas do Studio.

## Decisões propostas (a confirmar)

3. O subgrupo tem a **própria grade** (largura, altura, posições), com um subconjunto das cabeças do pai. Assim o EFX e o RGB Matrix seguem a ordem da grade do subgrupo sem mudança no motor.
4. **Selecionar o pai inclui tudo** (inclusive o que está nos subgrupos); selecionar um subgrupo inclui só as cabeças dele. Não há herança de valores: um subgrupo usado numa cena é um grupo como outro qualquer (cena por grupo, F10b).
5. **Remover um grupo:** os subgrupos sobem para o pai dele (ou para a raiz), como nas pastas. O "congelar" da F10b vale só para o grupo removido.
6. Um grupo não vira subgrupo de si mesmo nem de um descendente.

## Mudanças

- **Contrato (`rlc-core`):** `FixtureGroup.parent: Option<Id>` (ausente = raiz). Projeto na versão 21; a migração só sobe a versão (G7).
- **Servidor (`apply.rs`):**
  - `check_group` confere que as cabeças do subgrupo existem no pai e que não há ciclo;
  - ao salvar um grupo, remove as cabeças que saíram dos descendentes (`follow_groups`);
  - `RemoveFixtureGroup` faz os filhos subirem;
  - `MoveFixtureGroup` passa a aceitar o novo pai (arrastar na árvore).
- **UI:**
  - `FixturesGroups.vue`: a lista de grupos de fixtures vira **árvore** (recuo, seta de abrir/fechar, ícone de grade). Botão "Novo subgrupo" ao lado de "Remover"; arrastar leva o grupo para dentro de outro ou para a raiz (como as pastas do Studio).
  - Editor do subgrupo: a grade só deixa pintar cabeças do pai.
  - Seletores de grupo (cena, EFX, RGB Matrix, Phaser, console, grupo de canais): lista na ordem da árvore, com recuo.
- **Exemplo (G8):** no `exemplo.json` e no "Show de exemplo" salvo: "ParLeds" com os subgrupos "Esquerda" e "Direita" (6 cabeças em 2 × 3 cada), e uma cena que usa só "Esquerda".

## Validação com você

- [ ] A árvore mostra grupos e subgrupos de imediato, com abrir e fechar.
- [ ] Criar um subgrupo só aceita cabeças do pai.
- [ ] "Selecionar grupo" na cena com "Esquerda" inclui só as fixtures dela; com "ParLeds", todas.
- [ ] Tirar uma fixture do pai a tira do subgrupo.
- [ ] Arrastar move subgrupos entre pais; remover o pai faz os filhos subirem.
- [ ] Projetos antigos abrem; `pnpm check` passa.

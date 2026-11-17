# F13 — Editor de fixtures

**Status:** mini-plano escrito em 2026-11-17, **aguardando a aprovação** (G1). Prioridade nº 1 do usuário depois da rodada de validação (DECISOES, 2026-11-17).

## Objetivo

Criar e editar fixtures à mão na Biblioteca, sem depender de um `.qxf`: canais, faixas de valores, modos, cabeças e dados físicos. Vale também para ajustar uma fixture importada.

## O que já existe

- `FixturesEditor.vue` (288 linhas, nasceu como protótipo na F2): abas Geral, Canais, Modos e Dados físicos; faixas com aviso de lacuna, sobreposição e inversão; modos com a ordem dos canais; aparência do feixe (F15b). Salva por `saveFixtureDef`.
- `FixturesLibrary.vue`: Novo, Importar `.qxf`, Editar, Remover.
- Servidor (`apply.rs`, `Op::SaveFixtureDef`): confere **só** o id e a aparência do feixe. O resto é aceito como vem.

## O que falta (a F13 entrega)

1. **Validação no servidor.** Hoje uma definição inválida entra na biblioteca e no projeto.
2. **Cabeças:** o contrato tem `Mode.heads` (barras de LED, pixels), mas o editor não edita.
3. **Canal:** o valor padrão (`default`) e a cor da faixa (`color`, das rodas de cor) não têm campo. Não há como reordenar ou duplicar canais.
4. **Definição em uso:** editar uma definição que o projeto usa pode quebrar fixtures, cenas e grupos (ver a decisão 1).
5. **Modos:** não há duplicar nem aviso de que o modo está em uso.
6. **Biblioteca:** não há duplicar definição (variante de uma fixture) e o id gerado pode colidir ou sair com hífens nas pontas.

## Decisões (recomendação em negrito)

1. **Definição em uso pelo projeto.** A fixture do projeto guarda o modo pelo nome, e as cenas, os grupos de canais e as criações guardam o canal pela posição no modo. Mudar a lista de canais de um modo em uso desloca endereços e cria conflitos.
   - **Recomendado:** o servidor recusa mudar a lista de canais (quantidade, ordem, quais) e renomear ou remover um modo que alguma fixture do projeto usa, com a mensagem dizendo quais fixtures. O resto é livre: nomes, atributo, valor padrão, faixas, cabeças, dados físicos, feixe, tipo, outros modos e modos novos. O editor mostra "em uso por N fixtures" no modo e oferece "Duplicar modo" para criar a variante.
   - Alternativa: liberar tudo e deixar o projeto quebrar (rápido, mas perigoso ao vivo).
2. **Id e conflito.** **O id nasce de fabricante e modelo, sem hífens nas pontas, e não muda mais** (editar o modelo não troca o arquivo). Criar uma definição nova com fabricante e modelo que já existem é recusado ("já existe, edite ou duplique"); o `.qxf` continua substituindo, como hoje (E3).
3. **Regras do servidor ao salvar** (todas com mensagem em pt-BR):
   - fabricante, modelo e nome de canal e de modo não vazios; nomes de modo únicos;
   - de 1 a 512 canais por modo; índice de canal do modo existe e aparece uma vez só no modo;
   - valor padrão e faixas de 0 a 255, com início ≤ fim; faixa sobreposta é recusada (lacuna continua só aviso na tela, como no `.qxf`);
   - cor de faixa no formato `#rrggbb`;
   - cabeça: posições existem no modo e cada posição fica em no máximo uma cabeça;
   - o canal fino só vale depois de um canal grosso do mesmo atributo no modo (regra do E3): **aviso na tela, e o servidor aceita** (para não recusar `.qxf` importados).
4. **Cabeças no editor.** Na aba Modos, uma lista de cabeças: cada uma escolhe os canais dela dentre os do modo. Atalho "Uma cabeça por RGB" (tripla R, G, B seguida) para as barras de LED; o resto é à mão.
5. **Canal.** Campos novos: valor padrão (0–255) e cor em cada faixa (só em roda de cor e macro de cor). Ao trocar o atributo de um canal novo (sem faixas editadas), o nome e a faixa única vêm do atributo (como o preset do QLC+). Botões: subir, descer e duplicar canal (os modos acompanham a mudança de índice, como já fazem ao remover).
6. **Dados físicos: sem campos novos (G7).** Lâmpada, tipo de lente, tipo de foco, conector DMX e tamanho do layout do QLC+ não são usados por nada aqui. Ficam os nove que existem.
7. **Fora desta fase:** alias de canal (E3), exportar `.qxf`, GDTF (F13b), capabilities com valores por faixa além da cor (os "presets de capability" do QLC+).

## Contrato

- Nenhum tipo muda e o projeto continua na versão 22 (só a validação é nova). Atualizar `docs/protocolo.md` (o `saveFixtureDef` e as recusas).
- O servidor ganha uma função `check_fixture_def(def, library, project)` em `apply.rs` ou num módulo irmão, se passar de ~80 linhas (G7).

## Exemplos (G8)

G8 vale para criações; a F13 não mexe nelas. Em troca, o `exemplo.json` ganha **uma fixture de barra de LED com 3 cabeças RGB** (a "Barra 3 × RGB", com as cabeças definidas), para o usuário ver o editor de cabeças com um caso pronto. O "Show de exemplo" salvo não precisa dela: ela vem na biblioteca do exemplo.

## Checklist de execução

- [ ] 1. Aprovação do mini-plano e respostas às decisões 1 a 4.
- [ ] 2. Servidor: `check_fixture_def` com as regras da decisão 3 e o bloqueio de definição em uso (decisão 1), com testes (cada regra, e os `.qxf` da biblioteca de referência continuam entrando: o teste de importação já abre todos).
- [ ] 3. Editor, aba Canais: valor padrão, cor da faixa, preset do atributo, subir, descer e duplicar.
- [ ] 4. Editor, aba Modos: cabeças, "Duplicar modo", aviso "em uso por N fixtures".
- [ ] 5. Biblioteca: id estável, recusa de duplicata, "Duplicar" definição; mensagens do servidor mostradas ao salvar.
- [ ] 6. Barra de LED de exemplo no `exemplo.json`; `docs/protocolo.md`; `pnpm check`.
- [ ] 7. Teste no navegador (cópia dos arquivos do usuário antes, desfazer no fim).

## Validação com você

- [ ] Criar do zero um par RGBW de 7 canais (Dimmer, Vermelho, Verde, Azul, Branco, Strobo, Macro), com valor padrão no Strobo; patchar, mexer na Mesa simples e acender numa cena.
- [ ] Criar uma barra de LED de 3 cabeças RGB com o atalho, patchar e pintar as 3 cabeças num grupo de fixtures.
- [ ] Uma roda de cor com cores nas faixas aparece certa no editor de cena.
- [ ] Salvar com defeito (faixa sobreposta, canal repetido no modo, nome vazio) é recusado, com a mensagem dizendo o quê.
- [ ] Editar uma fixture em uso: mudar nome, faixa e valor padrão funciona; tirar um canal do modo em uso é recusado, e "Duplicar modo" resolve.
- [ ] Importar um `.qxf`, editar e salvar; a fixture do projeto recebe a mudança.
- [ ] Duplicar uma definição e criar uma variante; criar uma igual a outra é recusado.
- [ ] `pnpm check` passa.

# F15b — 3D realista

Plano aprovado: `C:\Users\Eliseu\.claude\plans\pode-seguir-e-criar-purring-bee.md` (cópia das decisões abaixo).
Pedido do usuário (2026-11-13): feixe por tipo de fixture (PAR = wash macio e sem foco; Beam = raio duro), canais ao vivo (zoom, foco, shutter), prisma com rotação, gobo, mancha no chão, bloom e modelos glTF do QLC+ (opção A). O usuário pediu para executar a fase inteira de uma vez, sem validar rodada por rodada (exceção ao G1).

## Regras para quem continuar
- Atualizar este checklist a cada tarefa pequena concluída (marcar `[x]` e anotar arquivos tocados e pendências em "Registro").
- Decisões novas: seção "Decisões tomadas sem perguntar" (o usuário revisa no fim).
- G6/G7: pt-BR nos textos e comentários, identificadores em inglês, código enxuto. Sem texto de ajuda na tela (só `RlInfo`).
- `pnpm check` deve passar no fim de cada rodada. Não commitar (G5).
- Só `View3dStage.vue` concentra o desenho; rodadas que o editam não rodam em paralelo.

## Decisões do plano
1. Perfil visual por tipo (`stage.looks`, `BeamLook`: softness, core, reach), padrões em `utils/beam.ts`; sem override por fixture. Ângulo vem de `physical.lensMin/MaxDeg` e do canal Zoom.
2. Modelos: `.dae` do QLC+ (moving_head, par, scanner, strobe) → `.glb` em `ui/public/models/`; barra de LED e o resto procedurais. Atribuição no `NOTICE` (G3).
3. Bloom só em qualidade média e alta. Sem dependência nova (GLTFLoader e EffectComposer vêm do `three`).
4. Contas num lugar só: `ui/app/utils/beam.ts`.

## Checklist

### R1 — Perfil e canais ao vivo
- [x] Contrato: `BeamLook` e `Stage.looks` (`crates/rlc-core/src/lib.rs`), tipos TS gerados.
- [x] Servidor: validação em `SaveStage` (`apply.rs`), projeto v18 com migração e teste (`project.rs`).
- [x] `ui/app/utils/beam.ts`: padrões por `FixtureKind`, `beamLook`, `beamState` (zoom → ângulo, foco → borda, shutter → pisca).
- [x] Shader do feixe com borda suave (`uSoft`), núcleo (`uCore`), alcance; ângulo ao vivo por escala do cone (sem recriar geometria).
- [x] Painel "Aparência" por tipo na aba Palco da `View3dScreen.vue` + textos em `ui/i18n/locales/`.
- [ ] `pages/dev/beams.vue`: comparação PAR × Beam. _Pulada na R1: a página é um spike instanciado (1 draw call), sem relação com o shader por fixture._
- [x] `pnpm check` passa.

### R2 — Mancha no chão e pós-processamento
- [x] Feixe termina no chão; disco aditivo com gradiente radial no ponto de contato.
- [x] Bloom (UnrealBloomPass) + tone mapping ACES por `prefs.quality`; piso levemente refletivo.
- [x] `pnpm check` passa.

### R3 — Prisma, rotação e gobo
- [x] Prisma: N cones em leque (3/4/5/6/8) pela capability do canal Prism; rotação pelo canal de rotação.
- [x] Gobo: texturas geradas em canvas projetadas na mancha, com rotação; roda de gobo pelas capabilities.
- [x] `pnpm check` passa.

### R4 — Modelos glTF
- [x] Converter os 4 `.dae` → `.glb` (`ui/public/models/`), Y-up, nós base/braço/cabeça conferidos (convenções e tabela em `ui/public/models/README.md`).
- [x] `build()` carrega e clona o modelo por `kind` (cache), liga pan/tilt aos nós, fallback procedural.
- [x] `NOTICE` atualizado.
- [x] `pnpm check` passa.

### Ajuste 1 (2026-11-14) — realismo e perfil por fixture
Pedido do usuário depois de ver a entrega: o glow ficou de neon/laser; o facho do Beam é cônico (o real é fechado, quase um cilindro); o PAR sai "fininho" (o real sai da face da lente e abre largo). Os ajustes passam a ser **por fixture**, na edição da fixture (tela Fixtures, `FixturesProject.vue`), e não mais por tipo na 3D View. Decisões: o perfil por tipo vira só o padrão no código; o ângulo do Beam tem limite por tipo no padrão (a definição do QLC+ nem sempre é fiel); o usuário ajusta por fixture.
- [x] A1. Contrato: `BeamLook` por fixture (`fixture`, `brightness`, `angleMin`, `angleMax`, `softness`, `core`, `reach`), `Stage.looks` por fixture, validação, projeto v19 (descarta os perfis por tipo da v18), limpar os looks de fixtures removidas, `exemplo.json`, testes. `beam.ts`: `defaultLook(def, fixtureId)` e `beamLook(stage, fixture, def)`.
- [x] A2. Desenho: tronco de cone (raio da lente), perfil núcleo + halo, intensidade quase constante no alcance, brilho com saturação suave (sem somar até o branco), mancha mais fraca, bloom bem mais fraco (força ~0,15–0,25, limiar ~0,9), PAR sai da face da lente. Usa `brightness` e os ângulos por fixture.
- [x] A3. UI: seção "Aparência do feixe" na edição da fixture (Fixtures), com "aplicar a todas do mesmo modelo" e "padrão"; remover o painel por tipo da aba Palco da 3D View; textos pt-BR.
- [x] A4. `pnpm check`; DECISOES.md (E12, A5 v19, Registro), PLANO.md (F15b) e este arquivo atualizados; exemplo (G8) com valores por fixture.

### Ajuste 2 (2026-11-14) — perfil do feixe na definição da fixture (Biblioteca)
Pedido do usuário: a aparência do feixe não é por fixture do projeto; fica na **definição** da fixture, editada na Biblioteca, para valer em qualquer projeto que a use. Decisão: `FixtureDef.beam: Option<BeamLook>` (`None` = padrão por tipo calculado pela UI); o perfil sai do projeto (`Stage.looks` removido, projeto v20, perfis das v18 e v19 descartados). `SaveFixtureDef` valida os limites e já atualiza a cópia embutida no projeto.
- [x] B1. Contrato, servidor, dados e `beam.ts`: `BeamLook` sem `fixture`; `FixtureDef.beam`; `Stage.looks` removido; validação em `SaveFixtureDef` (teste novo); migração v20 (`stage.looks` removido) com teste; `exemplo.json` com `beam` nas genéricas (PAR/wash e moving spot); `beamLook(def)` e `defaultLook(def)`; seção "Aparência do feixe" tirada de `FixturesProject.vue`. `pnpm check` ok.
- [x] B2. UI na edição da definição da Biblioteca (`FixturesEditor.vue`, aba "Dados físicos", o editor que `FixturesLibrary.vue` abre): knobs com as chaves `fixtures.look.*` (`defaultLook(def)` como padrão, "Padrão" grava `beam: null`).
- [x] B3. Docs: `DECISOES.md` (E3/E12, A5 v20, Registro) e `PLANO.md` (F15b).

### Fechamento
- [x] Exemplo (G8) no `exemplo.json` (perfis e um chaser com zoom/prisma/gobo). _O "Show de exemplo" salvo do usuário não foi atualizado (dados reais, app talvez aberto): ele abre o exemplo novo ou pede a conversão._
- [x] `PLANO.md` (fase F15b), `DECISOES.md` (E12, A5 v18, registro) e `CLAUDE.md` atualizados.
- [x] Checklist de validação para o usuário (abaixo) preenchido.

## Registro (o que foi feito, por data)
- 2026-11-13: contrato, servidor e migração v18 prontos; `cargo test -p rlc-core` e testes `version_*` do servidor passam.
- 2026-11-13 (R1): `utils/beam.ts`, shader com `uSoft`/`uCore`, ângulo e alcance por escala do cone (sem remontar), painel "Aparência" na aba Palco (só pt-BR existe), `pnpm check` ok. Corrigido um `collapsible_if` do clippy em `project.rs` (migração v18).
- 2026-11-13 (R2): mancha no chão em `View3dStage.vue` (eixo do cone cruza y=0; cone limitado a esse ponto ou ao alcance, `uSpan` no shader mantém o degradê; elipse pela inclinação); pós-processamento em `View3dPost.vue` (novo). `pnpm check` ok. Sem teste visual (ver decisões).
- 2026-11-13 (R4, assets): `ui/public/models/{moving_head,par,scanner,strobe}.glb` (29, 73, 58 e 35 KB) e `README.md` com nós, pivôs e lente; `NOTICE`. Conversão por script descartável fora do repositório (Edge + ColladaLoader/GLTFExporter do `three`); conferidos recarregando com o GLTFLoader.

- 2026-11-13 (R3): `utils/beam.ts` ganhou `prismState`, `spinSpeed`, `goboIndex` e `spinName` (puras; o projeto não tem vitest, conferidas num script avulso com as capabilities do exemplo). `View3dStage.vue`: cada feixe tem 8 cones (geometria e material comuns) num grupo `fan` que gira no eixo, cada um com a sua mancha; gobos em canvas (pontos, estrela, raios, círculos, linhas) como máscara da mancha, com rotação acumulada. `pnpm check` ok. Sem teste visual.

- 2026-11-13 (fechamento): `exemplo.json` ganhou `stage.looks` (colorChanger macio: borda 1, núcleo 0,4, alcance 7; movingHead duro: 0,05, 2 e 12), a pasta "3D realista" (cenas s12 a s15) e o chaser c3 (pasta própria "3D realista", porque a pasta é de um tipo só). `cargo test -p rlc-server` ok (o teste da v17 limpa os perfis do exemplo antes de comparar).
- 2026-11-13 (R4, integração): `View3dStage.vue` carrega os `.glb` (GLTFLoader, `${baseURL}models/`, uma Promise por arquivo) para moving head, scanner, PAR (colorChanger e dimmer) e strobo; o corpo procedural (`shell`) aparece até o modelo chegar (ou para sempre, se falhar) e é trocado sem remontar a cena (`swapModel`). Pan/tilt nos nós `arm`/`head`; leque, cones e vidro da lente vão para o nó `lens`; manchas continuam na cena (contas no mundo, valem com a raiz girada do scanner e do strobo). Geometria dos clones comum (`userData.shared`, o `dispose` não a apaga). `pnpm check` ok. Sem teste visual.
- 2026-11-14 (A1): `BeamLook` por fixture (brilho, ângulos mín. e máx., borda, núcleo, alcance); validação em `SaveStage` (fixture existente, uma por fixture, limites); `RemoveFixtures` limpa os looks; projeto v19 (descarta os perfis por tipo); `exemplo.json` com perfis em rpar1 a 3 e rbeam1 a 4; testes novos (`beam_looks_are_per_fixture…`, `version_18…`); `beam.ts` com `defaultLook(def, id)`, `beamLook(stage, fixture, def)` e `beamState` usando `angleMin/angleMax` (sem o parâmetro `physical`). Para compilar: `View3dScreen.vue` perdeu o painel por tipo (as chaves `view3d.look.*` do pt-BR ficaram, para o A3 reaproveitar ou remover) e `View3dStage.vue` só passa fixture e def. `pnpm check` ok.

- 2026-11-14 (A3): seção "Aparência do feixe" em `FixturesProject.vue` (knobs com LCD; Padrão só aparece com ajuste; "Aplicar ao mesmo modelo" copia para as fixtures de mesmo `defId` e modo numa só `saveStage`); chaves `fixtures.look.*` no lugar das `view3d.look.*`. `pnpm check` ok, sem teste visual.
- 2026-11-14 (A2): `View3dStage.vue`: o feixe vira tronco de cone (cilindro aberto de raio 1 e altura 1; o vertex shader põe o raio = lente + comprimento × tan(ângulo/2), o comprimento vem da escala Y de cada cone do leque; uniforms `uLens`, `uHalf`, `uReach` no lugar de `uLength`/`uSpan`); lente de 0,10 m no PAR e 0,05 m no moving head e no scanner (o vidro procedural tem o mesmo raio). Perfil núcleo fino (`exp(-30ρ²)`, peso 0,2·`core`) + halo fraco (0,25, borda por `softness`), intensidade `(1 - 0,3·t)` com fade nos últimos 15% do alcance, saturação `1 - exp(-x)` por canal; `brightness` multiplica cone e mancha, `smoke` só o cone. Mancha: raio = lente + t·tan, brilho × `clamp(0,5/raio, 0,15, 1,5)` × `(1 - 0,5·t/alcance)`, saturação suave. Os dois shaders incluem `colorspace_fragment`. `View3dPost.vue`: bloom 0,2 (média) e 0,3 (alta), raio 0,3, limiar 0,9; sem ACES. `pnpm check` ok, sem teste visual.

- 2026-11-14 (B1): `rlc-core` (`FixtureDef.beam`, `BeamLook` sem `fixture`, `Stage` sem `looks`), `apply.rs` (validação do `beam` em `SaveFixtureDef`, sem a de `SaveStage`/`RemoveFixtures`), `project.rs` (v20), `exemplo.json` (`beam` em `generico-par-rgbw`, `generico-par-rgb` e `generico-moving-spot`; `stage.looks` removido), `rlc-qxf` (`beam: None`), `beam.ts`, `View3dStage.vue`, `FixturesProject.vue`, `FixturesLibrary.vue` (`beam: null` no `newDef`). As chaves `fixtures.look.*` do pt-BR ficaram sem uso até a B2 (o lint não acusou). Os modelos do usuário (ParLed 18x15w e Beam 7R) ficaram sem `beam` (os testes de `rlc-qxf` os comparam com o `.qxf`) e usam o padrão por tipo; no exemplo, os ParLeds/Beams reais perdem o ajuste que tinham em `stage.looks`.

- 2026-11-14 (B2): seção "Aparência do feixe (3D)" no `FixturesEditor.vue` (6 knobs com LCD, grade de 3 colunas e 6 em tela larga; edita `draft.beam`, que só grava no Salvar; "Padrão" só com `draft.beam` e grava `null`; some em tipos sem cone). Chave `fixtures.look.sameModel` removida e `info` ajustado. `pnpm check` ok.

## Decisões tomadas sem perguntar
- B1: o exemplo não dá `beam` aos modelos do usuário (comparados com o `.qxf` nos testes); só às genéricas.
- A2: o tronco é feito no vertex shader (a escala Y dá o comprimento; o raio sai de `uLens + t·uHalf`), e não por reconstrução de vértices: a geometria continua uma só para os 8 cones, e o raio da lente não cresce com o comprimento.
- A2: tone mapping desligado também em média e alta (o ACES puxava o azul e o vermelho fortes para o branco); o OutputPass só converte para sRGB. Os shaders do feixe e da mancha passam pelo `colorspace_fragment`, então as três qualidades têm as mesmas cores (antes, a baixa escrevia o valor linear direto e as outras o convertiam).
- A2: o raio da lente é por tipo no código (PAR 0,10 m; moving head e scanner 0,05 m), fora do `BeamLook`. O `core` não muda mais o brilho da mancha (só o do cone).
- R4: o clone usa o material `body` de cada fixture (o realce da escolha continua), e não o do `.glb`; a lente que brilha é o mesmo disco (strobo: placa) de antes, dentro do `lens`.
- R4: escala pelo tamanho físico só com os três `physical.*Mm` > 0 (o menor fator, como o QLC+), aplicada num grupo por fora da raiz (a raiz guarda a translação que pendura). O leque recebe 1/escala para o alcance continuar em metros do palco.
- R4: scanner: a cabeça (espelho) inclina a metade do tilt e o leque a outra metade em volta do mesmo eixo X; o feixe sai com o tilt inteiro do `tiltMaxDeg` (o curso é o do feixe), e o espelho gira a metade, como um espelho de verdade. Pan 1×.
- R4: um modelo que falha ao carregar não é tentado de novo até recarregar a página.
- Fechamento: o Beam 7R não tem canal de foco nem de rotação do gobo (tem "Frost" e um canal "No function"); por isso as cenas de foco e de gobo girando também usam o grupo "Moving heads" (moving spot genérico, que tem foco e rotação do gobo). O Beam usa zoom, prisma (canal 5 e rotação no segundo canal `prism`) e a roda de gobo ("Red Sun").
- R3: o canal de rotação do prisma é um segundo canal `prism` com "rot"/"spin"/"gir" no nome (o Beam 7R tem "Prism Rotation"; não há atributo próprio). Sem ele, a capability do prisma que fala em rotação gira devagar (0,5 rad/s).
- R3: facetas = o primeiro número do nome da capability que seja 3, 4, 5, 6 ou 8 ("Prisma 2 / 5 facetas" = 5); sem ele, 3. Prisma desligado: valor 0 ou capability "Default/Desligado/Off/Open/Aberto/Sem…".
- R3: rotação (prisma e gobo) até 1 volta/s, de 10% no começo da faixa a 100% no fim; anti-horário quando o nome diz "left/esquerda/anti/counter/reverse"; parada em "stop/position/posição/index". Canal de uma faixa só (ex.: "Rotação do gobo" 0–255): proporcional ao valor, horário.
- R3: gobo pela ordem das figuras distintas (sem "Aberto/Default" e sem "Wheel/Roda/Scroll", a roda girando, que vale como aberto); "Shake X" usa a figura de X; módulo 5 texturas. No cone, o gobo só reduz o brilho a 0,6.
- R3: o leque abre 5° por feixe e a luz se divide por N (cone e mancha). O degradê do cone (`uSpan`) e a cor da mancha são comuns aos feixes do leque (diferem pouco).
- R4: todos os modelos pendurados, com a origem no topo (`y = 0`) e o feixe para -Y, como o resto do app; scanner girado 90° e strobo 180° em X na raiz (os eixos locais de `arm`/`head` ficam os do QLC+: pan = `rotation.y`, tilt = `rotation.x`). Alça do strobo endireitada para ficar em pé.
- R4: escala de tamanho típico (moving head 0,36 m de largura, PAR 0,25 m, scanner 0,6 m de comprimento, strobo 0,43 m); o tamanho físico da fixture pode escalar a raiz por igual.
- R4: marcador `lens` (Object3D vazio) em cada modelo, com o feixe no -Y local dele, para o cone e o emissivo; a lente não tem malha própria em nenhum dos quatro.
- R2: pós-processamento num componente filho do `TresCanvas` (`View3dPost.vue`) que troca a função de render pelo `useLoop().render` com `EffectComposer` (RenderPass, UnrealBloomPass, OutputPass); ACES vem do `renderer.toneMapping`, aplicado pelo OutputPass. Baixa: render padrão e sem tone mapping. Mudar a qualidade reconstrói o composer.
- R2: a mancha não leva fumaça (só o cone); brilho = cor × intensidade × (0,4 + 0,5·núcleo) × fator do ângulo × queda com a distância.
- R2: sem teste visual: não rodei o `pnpm dev` (usa as pastas reais do usuário); validado só por typecheck, lint e leitura. Com o OutputPass, a cor dos materiais customizados (shader do feixe) passa a ser codificada em sRGB, então podem parecer mais claros: ajustar na validação visual.
- Perfil guardado no projeto, por tipo (não por fixture).
- Íris fica de fora (convenção do canal varia entre fixtures); entra se o usuário pedir.
- Executada a fase inteira sem validar cada rodada (pedido do usuário).

## Validação com você
Abra o show de exemplo novo (o salvo antigo não tem os perfis nem a pasta) e a 3D View. Criações: pasta "3D realista" (cenas) e o chaser "Beams: zoom, prisma e gobo". Rodar uma cena acende os 4 Beams (grupo "Beams") e os 4 Moving heads.

- [ ] PAR (ParLeds): feixe macio, sem borda dura e sem foco; o painel "Aparência" da aba Palco mostra o perfil "Color changer" do exemplo (borda 1, núcleo 0,4, alcance 7).
- [ ] Beam 7R: raio duro e fino ("Moving head": borda 0,05, núcleo 2, alcance 12); mudar os valores no painel muda o feixe na hora e fica no projeto.
- [ ] Zoom e foco ao vivo: rodar "Beams: zoom estreito e foco" e "Beams: zoom aberto e foco suave" e depois mexer no Zoom e no Foco pela Mesa simples; o cone abre e fecha, e o foco suaviza a borda (só nos Moving heads, que têm o canal).
- [ ] Shutter: no Strobo do canal, o feixe pisca.
- [ ] Prisma: "Beams: prisma girando" abre 3 a 8 feixes em leque; a rotação (Beam: segundo canal Prisma, valores 193 a 255) gira o leque.
- [ ] Gobo: "Beams: gobo girando" mostra a figura na mancha; nos Moving heads o canal de rotação gira a figura.
- [ ] Mancha no chão: cada feixe termina numa mancha (elipse inclinada com o pan e o tilt), com a cor e a intensidade da fixture.
- [ ] Bloom: brilho nos feixes e nas lentes nas qualidades média e alta; em baixa, sem bloom e sem tone mapping.
- [ ] Desempenho: as 16 fixtures do rig (ParLeds e Beams) a 60 fps no desktop e a 30 fps ou mais no celular, em qualidade baixa e média.
- [ ] Modelos: Moving head (pan gira a base, tilt gira a cabeça, no sentido certo), PAR, scanner e strobo aparecem pendurados, no tamanho certo; barra de LED e o resto continuam procedurais.
- [ ] Painel "Aparência" na aba Palco: um perfil por tipo, sem erro ao salvar; abrir um projeto antigo (sem perfis) usa os padrões.
- [ ] O chaser "Beams: zoom, prisma e gobo" passa pelas 4 cenas em laço (4 s cada, fade de 1 s).
- [ ] Cores: o feixe não ficou claro demais com o tone mapping (ver o registro da R2); se ficou, ajustar.

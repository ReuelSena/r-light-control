# F4 — Fixtures: formato e importador .qxf

**Status:** mini-plano aprovado em 2026-10-17, com as decisões 1 a 5 como recomendado. A biblioteca do QLC+ nunca é embutida: as fixtures entram só importadas pelo usuário ou criadas à mão. Entregue e validada pelo usuário em 2026-10-17.

**Entregue**
- **Importador (`crates/rlc-qxf`):**
  - o ParLed 18x15w e o Beam 7R saem iguais à conversão feita à mão no exemplo, com o Strobo do Beam em 255;
  - o Strobo 1000w sai com 11 canais e sem avisos;
  - as 1.781 fixtures do QLC+ 5.3.0 importam sem recusa.
- **Avisos na biblioteca do QLC+:** 536 das 1.781 fixtures têm algum aviso:
  - 387 imagens de gobo;
  - 158 canais HSV (matiz, saturação…), marcados como "outro";
  - 114 canais finos sem o grosso antes;
  - 65 com Alias;
  - 28 com dados físicos por modo.
- **Biblioteca:**
  - gravada em `%APPDATA%\r-light-control\fixtures\`, um `.json` por definição;
  - o seu `pnpm dev` já criou a pasta com as 6 genéricas ao reiniciar com o código novo;
  - "Remover" (dois toques, como nas outras telas), importar vários `.qxf` de uma vez, relatório por arquivo e lista em ordem alfabética.
- **Valor padrão na saída:**
  - o universo parte dos padrões, e a Mesa simples fica por cima;
  - o fader do canal solto mostra o padrão;
  - no editor de cena, os knobs de lima, índigo e temperatura de cor aparecem na aba Cor quando a fixture tem esses canais.
- **Diferenças do plano:**
  - **Canal fino:** vale com o canal grosso em qualquer posição antes dele, não só logo antes. Na biblioteca do QLC+, 205 canais de pan e tilt finos vêm em "Pan, Tilt, Pan fino, Tilt fino", e o QLC+ junta esses pelo tipo.
  - **Arquivo próprio:** a definição fica dentro de `fixture` (`{format, schemaVersion, fixture}`), e não em campos soltos ao lado de `format`.
  - **Aviso de substituição:** aparece também quando só o projeto já tinha a fixture. Por isso, ao importar o ParLed e o Beam no show de exemplo, o relatório avisa que a cópia do exemplo foi substituída.
- **Testes:**
  - automáticos: 7 do importador e 4 com as suas fixtures e a biblioteca do QLC+; no servidor, patch com os padrões, importação substituindo, remoção, id inválido e a biblioteca gravada e relida; no motor, o padrão sob a Mesa simples e no blackout;
  - no navegador, contra o `pnpm dev`:
    - importar os três de uma vez, com o relatório certo;
    - Strobo removido da lista e do disco;
    - os 4 Beams saindo com o Strobo em 255, sem nada na Mesa simples;
    - fader do canal 2 em 100%;
    - a Biblioteca no celular.
  - Depois do teste, as importações foram removidas, e a pasta da biblioteca voltou a ter só as 6 genéricas.

## Objetivo

Trocar a importação simulada pela de verdade. Depois da F4:
- qualquer `.qxf` do QLC+ vira uma definição no formato próprio, com um relatório do que precisa de atenção;
- a biblioteca do usuário fica gravada no PC e sobrevive a fechar o app;
- o canal tem valor padrão, e o Beam 7R acende sem mexer no Strobo.

## O que entra

### 1. Formato de fixture (`rlc-core`, E3)

- **Arquivo próprio:** `{ "format": "r-light-control/fixture", "schemaVersion": 1, ...definição }`, um por definição. As migrações (A5) começam a valer quando houver a versão 2.
- **Valor padrão do canal** (`default`, de 0 a 255): o `Default` do `.qxf`. O Strobo do Beam 7R fica em 255 ("luz ativa").
- **Atributos novos**, para nada importante cair em "outro":
  - `lime` e `indigo` (cores de LED que o QLC+ tem): são intensidade, então o blackout e o Grand Master valem para eles;
  - `colorTemperature`: os canais de CTO, CTC e CTB.
- **Tipo novo:** `effect`, que inclui os tipos Effect e Flower do QLC+.
- **O que não muda:** identificador da definição = slug de fabricante + modelo (`rsena-beam-7r`), como na conversão à mão.

### 2. Importador (`crates/rlc-qxf`)

Uma função: recebe o XML e devolve a definição e os avisos, ou a recusa. Usa o `roxmltree` (leitor de XML pequeno e sem dependências).

**Conversão** (regras tiradas do `engine/src/qlc*.cpp` da 5.3.0):

| QLC+ | Formato próprio |
|---|---|
| Intensity sem cor, ou `IntensityMasterDimmer`/`IntensityDimmer` | `dimmer` |
| Intensity com cor, ou `Intensity{Red…Lime}` | a cor (`red` … `lime`) |
| Pan, Tilt (inclui `PositionXAxis`/`YAxis`) | `pan`, `tilt` |
| Speed | `speed` |
| Colour: `ColorMacro`, `ColorRGBMixer` | `colorMacro` |
| Colour: `ColorCTOMixer`, `ColorCTCMixer`, `ColorCTBMixer` | `colorTemperature` |
| Colour sem preset, ou `ColorWheel` | `colorWheel` |
| Gobo: `GoboIndex` | `goboRotation` |
| Gobo sem preset, ou `GoboWheel` | `gobo` |
| Shutter: `ShutterIris*` | `iris` |
| Shutter sem preset, ou `ShutterStrobe*` | `shutter` |
| Beam: `BeamFocus*`, `BeamZoom*` | `focus`, `zoom` |
| Beam sem preset, Effect | `effect` |
| Prism | `prism` |
| Maintenance | `maintenance` |
| Nothing, `NoFunction`, Hue/Saturation/Lightness/Value | `other` |

- **Canal fino:** os presets `…Fine` e o `Byte="1"`, só quando há um canal grosso do mesmo tipo antes dele no modo (logo antes ou mais atrás, como em "Pan, Tilt, Pan fino, Tilt fino"), como o QLC+ faz para juntar os 16 bits.
  - Exemplo: o "Gobo Wheel" do seu Beam 7R está com `Byte="1"` no `.qxf`. No QLC+ isso não faz efeito; aqui ele fica normal, com um aviso.
- **Faixas de valores:**
  - Canal com preset ou sem faixas no arquivo: ganha uma faixa de 0 a 255 com o nome do canal, como na conversão à mão.
  - Faixas sobrepostas: a de depois sai, como no QLC+, com aviso. Buracos (como no "Lamp Control" do Beam) são aceitos.
- **Cor das faixas:**
  - vem do `ColorMacro` do arquivo (`Res1`);
  - nas rodas e macros de cor sem cor no arquivo, vem do nome da faixa ("Red", "Vermelho", "Pink 2"…), como na conversão à mão do Beam. Nomes compostos ("White / Red") ficam sem cor.
- **Modos e cabeças:** os canais do modo pelo nome e as cabeças pela posição, como no QLC+. Nome de canal que não existe sai do modo, com aviso.
- **Dados físicos:** medidas, peso, consumo, lente e pan/tilt máximos. O físico próprio de um modo é ignorado, com aviso quando difere do geral.
- **Fica de fora, com aviso no relatório:**
  - **Alias.** São as faixas que trocam um canal do modo por outro conforme o valor. Existem em 65 das 1.781 fixtures do QLC+.
  - **Imagens de gobo.** Ficam para a F15, junto com os gobos do 3D.
- **Recusa** (o app avisa e nada entra na biblioteca): arquivo que não é XML, que não é `FixtureDefinition`, ou sem canais ou modos.

### 3. Biblioteca do usuário

- **Gravada no PC:** um arquivo por definição em `%APPDATA%\r-light-control\fixtures\`, com escrita atômica (A5). Lida ao abrir o app. Um arquivo estragado é ignorado e anotado no log.
- **Primeira vez:** a pasta começa com as 6 genéricas do exemplo.
  - Os seus três (ParLed, Beam 7R e Strobo) você importa pelo app; é a validação.
  - O show de exemplo continua com as cópias dele (A4).
- **Importar de novo** a mesma fixture (mesmo fabricante e modelo) substitui a da biblioteca e a cópia do projeto, como o "Salvar" do editor. O relatório avisa.
- **Excluir da biblioteca:** operação nova `removeFixtureDef`. O projeto continua com a cópia dele.
- **Pelo celular:** o "Importar .qxf" já manda o arquivo pelo WebSocket. Vou conferir se o seletor de arquivos do Android deixa escolher `.qxf`. Se não deixar, ele passa a aceitar qualquer arquivo e o servidor recusa o que não for `.qxf`.

### 4. Valor padrão na saída

- No motor, cada universo parte dos valores padrão das fixtures patchadas. A Mesa simples fica por cima, e o canal solto volta ao padrão.
- O "Zerar" da Mesa simples solta os canais. O Strobo do Beam volta a 255, e não a 0.
- O blackout e o Grand Master continuam valendo só para a intensidade. O Strobo fica em 255 no blackout, e o dimmer vai a zero.

### 5. Telas (G2)

São mudanças pequenas, nos padrões do design system:
- **Biblioteca:**
  - botão "Excluir" ao lado de "Editar" e "Adicionar ao projeto", com confirmação;
  - "Importar .qxf" aceita vários arquivos de uma vez, e o relatório mostra um bloco por arquivo (resumo, avisos ou recusa).
- **Mesa simples:** o fader de um canal solto mostra o valor padrão, e não 0.
- **Editor de fixture (protótipo):** os atributos e o tipo novos nas listas. O campo do valor padrão no editor fica para a F13.

## Decisões para você

1. **Embutir a biblioteca do QLC+ já convertida? Recomendo não, por enquanto.**
   - São 1.781 fixtures. Hoje a biblioteca vai inteira para cada aparelho ao conectar. Com todas embutidas, o celular baixaria vários MB a cada conexão, e o protocolo teria de mudar (lista resumida e definição sob pedido).
   - O seu rig é de fixtures próprias. Qualquer uma do QLC+ você importa da sua pasta `R:\Light\QLC+\Fixtures`, agora várias de uma vez.
   - Mesmo sem embutir, o importador é testado contra as 1.781 da referência.
   - **Resposta:** não embutir, nem agora nem depois. As fixtures entram só importadas ou criadas à mão.
2. **Biblioteca gravada no PC já na F4** (recomendado). Sem isso, o Strobo importado some ao fechar o app. Só o projeto continua sem salvar até a F5.
3. **Valor padrão aplicado na saída já na F4** (recomendado). Sem isso, o padrão fica só guardado e o Beam continua precisando do Strobo na mão.
4. **Cor das faixas pelo nome quando o arquivo não traz** (recomendado). A roda de cor do Beam aparece com as cores, como na conversão à mão. A alternativa é só a cor do arquivo; aí o Beam fica sem cores até você editar.
5. **As mudanças de tela da seção 5.**

## Testes automáticos

- **Importador:**
  - Os seus três `.qxf`, copiados para os testes. O ParLed e o Beam 7R têm de sair iguais à conversão à mão do exemplo, com o Strobo do Beam em 255. O Strobo 1000w sai com 11 canais.
  - Casos de borda: canal inexistente no modo, faixas sobrepostas, fino sem canal grosso, preset sem faixas, `ColorMacro` com cor, `Res` antigo, Alias, XML inválido e arquivo sem modos.
  - **As 1.781 fixtures do QLC+ 5.3.0** (`reference/qlcplus`) importam sem recusa. O teste roda quando a referência existe.
- **Servidor:** a biblioteca gravada e relida, importar de novo substituindo, excluir sem mexer no projeto.
- **Motor:** valor padrão no quadro, a Mesa simples por cima, o padrão de volta ao soltar, o blackout mantendo o Strobo.
- `pnpm check` passando.

## Validação (com você)

Antes de começar:
- **Reinicie o `pnpm dev`**, para o app abrir com o show de exemplo limpo. Os testes de navegador mexeram no estado em memória dele.
- **Os `.qxf`** estão em `R:\Light\QLC+\Fixtures\RSena\`. No "Importar .qxf", dá para escolher os três de uma vez.
- **Beam 7R:** a lâmpada liga com o "Lamp Control" (canal 16 do Beam) a partir de 128.

- [x] Importar o Strobo 1000w pelo app, adicionar ao projeto e controlar pela Mesa simples.
- [x] Importar o ParLed e o Beam 7R. O Beam acende só com o dimmer e a lâmpada ligada, sem mexer no Strobo.
- [x] O relatório faz sentido (o aviso do "Gobo Wheel" do Beam, por exemplo).
- [x] Conferir canais e modos de uma das três contra o editor de fixtures do QLC+.
- [x] Importar pelo celular.
- [x] Fechar e abrir o app: a biblioteca continua lá. Remover uma fixture da biblioteca.
- [x] `pnpm check` passa.

## Referências estudadas

- `engine/src/{qlcfixturedef,qlcfixturemode,qlcchannel,qlccapability,qlcphysical,qlcfixturehead,qlcfixturedefcache}.cpp`
- `resources/schemas/fixture.xsd`
- `engine/src/universe.cpp` (valor padrão do canal) e `fixture.cpp` (Alias)

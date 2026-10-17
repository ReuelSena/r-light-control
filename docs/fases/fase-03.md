# F3 — Motor base e saída USB DMX

**Status:** mini-plano aprovado em 2026-10-17 (decisões 1 a 4 como recomendado). Rodada 1 entregue em 2026-10-17, aguardando a validação do usuário.

**Rodada 1 entregue**
- **Contrato e protocolo:** o servidor é a autoridade (A2) e a UI é um cliente WebSocket. O envelope, os tópicos do `evt` e os frames binários estão em [`docs/protocolo.md`](../protocolo.md). O motor de mentira e os dados fictícios saíram da UI; o show de exemplo é `crates/rlc-server/src/exemplo.json`.
- **Motor (`rlc-engine`):** 50 Hz com prazo absoluto, Mesa simples ou padrão de teste, Grand Master (reduzir, como no QLC+) e blackout só na intensidade.
- **Saída Dummy (`rlc-dmx`):** a 44 Hz, com quadros por segundo, atraso e erros.
- **Servidor:** as 37 operações do contrato com validação, a lista de aparelhos conectados, o acesso pela rede local (recusa de fora quando desligado) e o Monitoramento real.
- **UI:**
  - UUIDv7 nos itens criados pela UI;
  - controles otimistas sob o dedo;
  - tela "Conectando…" e telas cobertas sem conexão;
  - aviso das recusas.
- **Telas:**
  - "TESTE" na barra quando o padrão de teste está ligado;
  - página `/dev/dmx`;
  - menu Projeto só com "Novo" até a F5;
  - In/Out sem o botão "Procurar" (a busca fica automática na rodada 2).
- **Diferenças do plano:**
  - O frame binário leva o id do universo, não a posição: um universo criado ou removido nunca troca os valores de lugar.
  - O motor não precisou de espera ativa: o `sleep` do Rust já usa o relógio de alta resolução do Windows, e a Dummy mede ~0,5 ms de atraso.
  - Os padrões de teste são a rampa e a perseguição (a rampa atrasada canal a canal).
- **Testes:**
  - automáticos: 5 do motor, 1 da saída, 6 das operações e 1 ponta a ponta (dois clientes, avisos antes da resposta, Mesa simples, Grand Master e blackout nos quadros);
  - no navegador: todas as telas; fader arrastado sem perder movimento; celular sincronizado; Monitoramento; aviso de recusa; queda e volta do servidor.

## Objetivo

Trocar o motor de mentira pelo motor de verdade, no servidor Rust, e pôr DMX real nas interfaces USB. Depois da F3:
- o que se mexe na Mesa simples, no Grand Master e no blackout sai na luz;
- todos os aparelhos (desktop, celular, janelas externas) veem a mesma coisa, ao vivo;
- existe o primeiro instalador.

## Duas rodadas, cada uma validada antes da próxima

### Rodada 1: servidor de verdade (sem hardware)

1. **Servidor como autoridade (A2, A3).**
   - O servidor guarda o estado completo (o `Snapshot`) e aplica as operações do contrato v0. O protótipo da F2 continua funcionando inteiro, agora com o estado no servidor.
   - Envelope de `docs/protocolo.md`: `hello`, `snapshot`, `cmd`, `ack`, `err` e `evt` com a revisão (`rev`).
   - O `evt` leva o pedaço do estado que mudou. Os tópicos são os campos do `Snapshot`, como `project.fixtures`, `project.console`, `desk`, `master`, `running`, `interfaces` e `stats`. Cada operação sabe quais tópicos muda.
   - Na UI, um cliente WebSocket entra no lugar do `MockClient`, sem mudar as telas. Se a conexão cai, a UI mostra que está reconectando e, ao voltar, recebe o estado completo.
   - O motor de mentira (`engine/mock.ts`) e os dados fictícios (`engine/fake.ts`) saem da UI. O show de exemplo vira um arquivo JSON dentro do servidor.
   - **IDs:**
     - a UI gera UUIDv7, com gerador próprio, e o servidor recusa um ID repetido;
     - operações que criam itens sem receber o ID (adicionar fixtures, universo) geram o ID no servidor.
2. **Motor (`crates/rlc-engine`, A1).**
   - Thread a 50 Hz com prazo absoluto: um atraso num tick é compensado no seguinte, sem deriva. No Windows, `timeBeginPeriod(1)` e espera ativa no fim de cada tick.
   - Os comandos chegam por canal e são aplicados a cada tick. A thread do motor não usa locks no caminho quente.
   - Universos de 512 canais. Nesta fase, a saída é a soma de:
     - **Mesa simples:** canais e grupos de canais;
     - **Grand Master:** no modo do QLC+ (reduz) e só nos canais de intensidade, `valor × GM`, arredondado. Os modos limitar e todos os canais ficam para a F6;
     - **Blackout:** os canais de intensidade vão a zero e os outros (pan, tilt, gobo…) ficam onde estão, como no QLC+. Assim os moving heads não se mexem no blackout;
     - **Pânico e "parar tudo":** param as criações. O pânico também zera a Mesa simples.
   - **Canais de intensidade:** dimmer e mistura de cor (vermelho, verde, azul, branco, âmbar, UV, ciano, magenta, amarelo). Canais sem fixture patchada também contam como intensidade, para o blackout apagar uma luz que só está na Mesa simples.
   - Cada universo publica o quadro mais recente. As saídas leem esse quadro no seu ritmo, sem dividir memória sem trava (um problema que o QLC+ tem).
3. **Saída Dummy:** interface virtual, sempre presente, para desenvolver e testar sem hardware. Mostra quadros por segundo e jitter como as reais.
4. **Valores ao vivo para as telas.**
   - Os valores de saída vão em frames binários a 25 Hz, só dos universos que mudaram. Cada frame tem 1 byte de tipo, 1 byte com o índice do universo no projeto e os 512 canais.
   - A 3D View, o Monitoramento e as strips usam esses valores.
5. **Monitoramento real:** CPU, memória, duração do tick e tempo ligado, mais quadros por segundo, jitter e erros de cada interface.
6. **Aparelhos conectados:** a lista do ⚙ mostra os clientes reais (nome e endereço) e a URL da rede local de verdade no QR.
7. **Teste de saída:** página `/dev/dmx` com padrões gerados no próprio motor (rampa em todos os canais e uma perseguição), para a validação de 10 minutos sem tremulação da rodada 2.

### Rodada 2: interfaces USB e instalador

1. **Saídas (`crates/rlc-dmx`), cada interface na sua thread (A1).** Todas usam o driver D2XX da FTDI, a 250 000 baud, 8N2, sem controle de fluxo.
   - **Enttec DMX USB Pro:** pacote `0x7E | rótulo | tamanho | dados | 0xE7`. O rótulo 6 leva o código de início `0x00` e os 512 canais. O número de série vem do rótulo 10. O quadro é enviado sempre, a 44 Hz.
   - **Open DMX:** break de 110 µs e MAB de 16 µs, com `FT_SetBreakOn`/`Off` e espera ativa, depois os 513 bytes, a 30 Hz. A thread roda com prioridade de tempo crítico.
     - Cada chamada ao USB leva perto de 1 ms, então o break real fica nessa ordem. Ainda está dentro do padrão DMX (break mínimo de 88 µs, sem limite prático para cima).
   - Prazo absoluto também nas saídas: o QLC+ não tem e oscila cerca de ±1 ms por quadro.
2. **Descoberta e hot-plug.**
   - A cada 1 s, a lista de aparelhos FTDI é lida de novo.
   - Pro ou Open DMX, pelo nome USB: "DMX USB PRO" é a Pro, e o resto é Open DMX, como no QLC+.
   - A interface é aberta pelo número de série, não pela posição na lista (outro problema do QLC+).
   - Desplugou: a interface fica desconectada na lista. Voltou: reabre sozinha em até 3 s e o universo continua patchado nela.
3. **In/Out ligado ao motor:** interfaces reais com estado, quadros por segundo, jitter e erros. Universo → interface pelo contrato atual (`updateUniverse`).
4. **Primeiro instalador:** `pnpm build` gera o instalador NSIS do Tauri, que baixa o WebView2 se faltar. A regra de firewall fica para a F16.

## O que vem do QLC+ (estudo das referências)

- **`plugins/dmxusb`:**
  - protocolo da Pro (rótulos 6 e 10, pacote e código de início);
  - break e MAB da Open DMX e as frequências padrão (44 Hz e 30 Hz);
  - linha 250 000 8N2;
  - detecção do modelo pelo nome USB.
- **`engine/src/universe.cpp`, `grandmaster.cpp`:** a conta do Grand Master (reduzir) e a regra de que o blackout só zera a intensidade.
- **`engine/src/mastertimer*.cpp`:** tick padrão de 50 Hz.
- **O que fazemos diferente:**
  - prazo absoluto no motor e nas saídas;
  - quadro trocado entre threads sem dividir memória;
  - abrir pelo número de série;
  - reconexão automática;
  - pedir 1 ms ao relógio do Windows (o QLC+ acaba pedindo 20 ms por um erro de conta).
- **Fora desta fase:**
  - segundo universo da Pro Mk2 (backlog v2);
  - RDM;
  - entrada DMX da Pro;
  - modelos DMXKing, DMX4ALL e outros.

## Decisões do usuário

1. **Todo o estado no servidor já na F3** (recomendado). As telas da F2 continuam funcionando, agora sincronizadas entre aparelhos.
   - Efeito colateral: até a F6, rodar uma cena, um chaser ou um EFX não acende nada. A 3D View e o Monitoramento mostram só a Mesa simples, com o Grand Master e o blackout.
   - A alternativa seria manter o motor de mentira para o Studio e o console, com duas fontes de verdade e código descartável. Não recomendo.
2. **Projeto inicial sem salvar até a F5** (recomendado). O servidor abre com o show de exemplo, "Novo projeto" funciona, e Abrir e Salvar ficam desligados até a F5. O que você mudar se perde ao fechar o app. A validação de hardware usa a Mesa simples, que não depende de fixtures.
3. **Só o driver D2XX, sem a saída pela porta COM** (recomendado). O plano previa a porta COM como reserva para a Open DMX. O QLC+ não usa porta COM para aparelhos FTDI no Windows, e o driver da FTDI, que o Windows instala sozinho, já traz o D2XX. Menos código e um caminho só.
   - A biblioteca da FTDI vai embutida no app (link estático), registrada no `NOTICE`. Falta conferir a licença na implementação.
4. **Blackout como no QLC+** (recomendado): zera a intensidade e mantém o resto. Canais sem fixture contam como intensidade.
5. **Hardware para a rodada 2:** uma interface genérica, sem marca, com chip FTDI.
   - Driver: o pacote oficial da FTDI (CDM 2.12.36.4). A biblioteca D2XX (`ftd2xx.dll` 3.2.21.1) já está instalada no PC.
   - Nome gravado na interface: "Numbstar". O QLC+ não tem regra para esse nome e trata como Open DMX tudo o que não se chama "DMX USB PRO". Por isso, a princípio é uma Open DMX.
   - A confirmar na rodada 2: o tipo que o QLC+ mostra para ela (Open ou Pro) e as fixtures do teste. Sem uma Enttec Pro, a Pro fica validada só pelos testes automáticos do pacote.

## Testes automáticos

- **Motor**, com relógio determinístico:
  - Mesa simples → quadro;
  - Grand Master só na intensidade, com a conta do QLC+;
  - blackout mantendo pan e tilt;
  - pânico;
  - um tick atrasado sendo compensado.
- **Servidor:**
  - cada operação muda o estado e os tópicos certos;
  - ID repetido recusado;
  - um cliente de teste recebe `snapshot`, `evt` e os frames binários.
- **`rlc-dmx`:** montagem do pacote da Pro e do quadro da Open DMX; a saída Dummy contando quadros.
- `pnpm check` passando, e os testes de navegador das telas da F2 rodando contra o servidor de verdade.

## Validação

**Rodada 1**
- [ ] Celular e desktop ao vivo: Mesa simples, Grand Master e blackout mudam nos dois na hora.
- [ ] Uma edição num aparelho (fixture, elemento do console, palco) aparece no outro.
- [ ] Ao derrubar e voltar a rede do celular, ele recebe o estado completo.
- [ ] Monitoramento com números reais e a saída Dummy a ~44 quadros por segundo.

**Rodada 2**
- [ ] A Pro e a Open DMX controlam fixtures reais.
- [ ] 10 min do teste de saída sem tremulação.
- [ ] A saída volta em até 3 s depois de replugar.
- [ ] Blackout instantâneo.
- [ ] A saída Dummy funciona sem hardware.
- [ ] O instalador funciona num PC limpo.
- [ ] `pnpm check` passa.

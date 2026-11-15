# Modelos 3D das fixtures

Convertidos dos `.dae` do QLC+ 5.3.0 (`reference/qlcplus/resources/meshes/fixtures/`, Apache-2.0, ver `NOTICE`) por um script descartável (ColladaLoader + GLTFExporter do `three`). Sem texturas, câmeras nem luzes; um material só (`body`, PBR cinza-escuro).

## Convenções (as mesmas nos quatro)

- Metros, Y para cima. A origem é o ponto de pendurar: topo do modelo em `y = 0`, eixo do pan em `x = z = 0` (no scanner, o centro do corpo). O corpo fica todo em `y < 0`.
- O nó raiz (com o nome do arquivo) carrega a translação para o topo e, no scanner e no strobo, a rotação que os pendura. Não zerar o transform da raiz.
- Pan: `arm.rotation.y`. Tilt: `head.rotation.x`. O pivô de cada um é a origem do próprio nó (vem assim do QLC+; nenhum pivô foi mudado). Como a raiz é que gira, os eixos locais do braço e da cabeça continuam os do QLC+.
- **`lens`**: um `Object3D` vazio na posição da lente, filho do nó que se move com ela. O feixe sai no **-Y local** dele, e em repouso (pan e tilt em 0) isso é -Y no mundo. Basta pôr o cone e o disco emissivo dentro dele.
- Escala: um tamanho típico de cada tipo. Para usar o tamanho físico da fixture (`physical.widthMm`…), escalar a raiz por igual pelo menor fator dos três eixos, como o QLC+ faz.

## Por modelo

| Arquivo | Tamanho (L × A × P, m) | Nós (hierarquia) | Pan / tilt | Lente (posição em repouso) |
|---|---|---|---|---|
| `moving_head.glb` | 0,36 × 0,49 × 0,27 | `moving_head` › `base` (`base_mesh`) › `arm` (`arm_mesh`) › `head` (malha) › `lens` | pivô do `arm` em y −0,253; do `head` em y −0,338 | fundo da cabeça, (0, −0,486, 0) |
| `par.glb` | 0,27 × 0,37 × 0,25 | `par` › `head` (`head_mesh`, `handle`, `lens`) | nenhum (peça única com a alça) | boca do PAR, (0, −0,365, 0) |
| `scanner.glb` | 0,28 × 0,34 × 0,60 | `scanner` › `base` (`base_mesh`, `handle`, `projector`, `arm` (`arm_mesh`) › `head` (`head_mesh`, `mirror`, `lens`)) | `arm` gira o espelho em volta do eixo do projetor; `head` inclina o espelho | centro do espelho, (0,003, −0,299, 0,139) |
| `strobe.glb` | 0,43 × 0,20 × 0,18 | `strobe` › `handle`, `base` (malha) › `lens` | nenhum | centro da face larga, (0, −0,203, 0); face de 0,40 × 0,18 |

- **PAR e strobo** não têm partes móveis separadas: a lente é uma face da malha (`head_mesh` no PAR, `base` no strobo), sem malha própria. O emissivo vai num disco ou plano novo dentro de `lens`.
- **Scanner:** o projetor manda a luz no espelho, e o feixe sai do espelho. No espelho de verdade, o feixe gira o dobro do ângulo do espelho; o `lens` acompanha só o ângulo do nó.

## Correções feitas na conversão

- Escalas: moving head ×0,45; PAR ×0,833; scanner ×0,7; strobo ×0,4 (os `.dae` vêm com tamanhos arbitrários: o moving head tinha 0,8 m de largura).
- Origem levada ao topo de cada modelo (no `.dae` ficava no meio do corpo).
- Scanner: no `.dae` fica em pé, com o feixe saindo na horizontal (+Z); a raiz gira 90° em X para pendurá-lo deitado, com a alça em cima e o feixe para baixo.
- Strobo: no `.dae` a face de luz olha para cima (+Y); a raiz gira 180° em X. A alça, que saía inclinada para trás, foi girada em volta do ponto em que prende no corpo até ficar em pé, em cima dele.
- Malhas sem nome ganharam `<nó>_mesh`; UVs e cores de vértice removidos.

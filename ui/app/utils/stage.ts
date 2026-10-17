import type { Component } from 'vue'
import {
  ChartNoAxesColumnIncreasing, Drum, Fence, Grid3x3, KeyboardMusic, Lectern, Mic, PersonStanding, Presentation,
  RectangleHorizontal, Speaker, Sprout, Volume1, Volume2,
} from 'lucide-vue-next'
import {
  BoxGeometry, CylinderGeometry, Euler, Group, Mesh, MeshStandardMaterial, Quaternion, SphereGeometry, Vector3, type BufferGeometry,
} from 'three'
import { mergeGeometries } from 'three/examples/jsm/utils/BufferGeometryUtils.js'
import type { FixturePosition } from '~/types/generated/FixturePosition'
import type { StageElement } from '~/types/generated/StageElement'
import type { StageElementKind } from '~/types/generated/StageElementKind'

// Elementos de palco da 3D View (E12): catálogo, modelos simples e o lugar de uma cópia.

/** Posição (m) e rotação (graus) de uma fixture ou de um elemento no palco. */
export type Placement = Omit<FixturePosition, 'fixture'>

export const stageElementGroups = ['structure', 'sound', 'scene'] as const

/** Ícone, grupo do catálogo e largura (m, no eixo X do elemento, usada para pôr a cópia ao lado). */
export const stageElementKinds: Record<StageElementKind, { icon: Component; group: (typeof stageElementGroups)[number]; width: number }> = {
  truss100: { icon: Fence, group: 'structure', width: 1 },
  truss150: { icon: Fence, group: 'structure', width: 1.5 },
  truss300: { icon: Fence, group: 'structure', width: 3 },
  riser: { icon: RectangleHorizontal, group: 'structure', width: 2 },
  stairs: { icon: ChartNoAxesColumnIncreasing, group: 'structure', width: 1 },
  ledPanel: { icon: Grid3x3, group: 'structure', width: 0.5 },
  projectionScreen: { icon: Presentation, group: 'structure', width: 3.2 },
  subwoofer: { icon: Speaker, group: 'sound', width: 0.6 },
  speaker: { icon: Volume2, group: 'sound', width: 0.4 },
  monitor: { icon: Volume1, group: 'sound', width: 0.55 },
  micStand: { icon: Mic, group: 'sound', width: 0.3 },
  person: { icon: PersonStanding, group: 'scene', width: 0.6 },
  drums: { icon: Drum, group: 'scene', width: 1.6 },
  keyboard: { icon: KeyboardMusic, group: 'scene', width: 1.3 },
  pulpit: { icon: Lectern, group: 'scene', width: 0.7 },
  plant: { icon: Sprout, group: 'scene', width: 0.7 },
}

const rad = (deg: number) => (deg * Math.PI) / 180

/** Gira e posiciona um objeto da cena (mesma ordem de rotação para fixtures e elementos). */
export function placeObject(o: { position: Vector3; rotation: Euler }, p: Placement) {
  o.position.set(p.x, p.y, p.z)
  o.rotation.set(rad(p.rotationX), rad(p.rotationY), rad(p.rotationZ), 'YXZ')
}

/** Lugar da cópia: ao lado, na direção da largura do elemento (girado, um truss duplicado continua a linha). */
export function besideOf(e: StageElement) {
  const step = new Vector3(stageElementKinds[e.kind].width, 0, 0).applyEuler(new Euler(rad(e.rotationX), rad(e.rotationY), rad(e.rotationZ), 'YXZ'))
  const round = (v: number) => Math.round(v * 100) / 100
  return { x: round(e.x + step.x), y: Math.max(0, round(e.y + step.y)), z: round(e.z + step.z) }
}

/**
 * Modelo simples de um elemento, feito de caixas e cilindros. A origem fica no centro da base, e a frente
 * olha para a plateia (+Z); o retorno olha para o palco.
 */
export function buildElement(kind: StageElementKind, segments: number) {
  const group = new Group()
  const materials = new Map<string, MeshStandardMaterial>()
  const add = (geometry: BufferGeometry, color: string, x: number, y: number, z: number, rotationX = 0) => {
    if (!materials.has(color)) materials.set(color, new MeshStandardMaterial({ color, roughness: 0.7 }))
    const mesh = new Mesh(geometry, materials.get(color))
    mesh.position.set(x, y, z)
    mesh.rotation.x = rotationX
    group.add(mesh)
    return mesh
  }
  const box = (w: number, h: number, d: number, color: string, x: number, y: number, z: number, rotationX = 0) => add(new BoxGeometry(w, h, d), color, x, y, z, rotationX)
  const cylinder = (r: number, h: number, color: string, x: number, y: number, z: number, rotationX = 0) =>
    add(new CylinderGeometry(r, r, h, segments), color, x, y, z, rotationX)
  const black = '#151617'
  const cone = '#050505'
  const metal = '#9aa0a6'
  const carpet = '#3a3d42'

  switch (kind) {
    case 'truss100':
    case 'truss150':
    case 'truss300': {
      // Seção quadrada de 29 cm: 4 tubos e as diagonais de cada face, em zigue-zague, numa geometria só
      const length = stageElementKinds[kind].width
      const half = 0.145 - 0.024
      const rods: BufferGeometry[] = []
      const rod = (a: Vector3, b: Vector3, r: number) => {
        const g = new CylinderGeometry(r, r, a.distanceTo(b), 6)
        g.applyQuaternion(new Quaternion().setFromUnitVectors(new Vector3(0, 1, 0), b.clone().sub(a).normalize()))
        rods.push(g.translate((a.x + b.x) / 2, (a.y + b.y) / 2, (a.z + b.z) / 2))
      }
      const corners = [[-half, -half], [-half, half], [half, half], [half, -half]] as const
      const n = Math.round(length / 0.29)
      corners.forEach(([y, z], i) => {
        const [y2, z2] = corners[(i + 1) % 4]!
        rod(new Vector3(-length / 2, y, z), new Vector3(length / 2, y, z), 0.024)
        for (let k = 0; k < n; k++) {
          const x0 = -length / 2 + (k * length) / n
          const [a, b] = k % 2 ? [[y2, z2], [y, z]] : [[y, z], [y2, z2]]
          rod(new Vector3(x0, a[0], a[1]), new Vector3(x0 + length / n, b[0], b[1]), 0.01)
        }
      })
      add(mergeGeometries(rods), '#c8ccd2', 0, 0.145, 0)
      break
    }
    case 'riser':
      box(2, 0.06, 1, carpet, 0, 0.37, 0)
      for (const [x, z] of [[-0.95, -0.45], [0.95, -0.45], [-0.95, 0.45], [0.95, 0.45]] as const) cylinder(0.025, 0.34, metal, x, 0.17, z)
      break
    case 'stairs':
      // Dois degraus de 20 cm, subindo para o fundo: chega à altura do praticável
      box(1, 0.2, 0.3, carpet, 0, 0.1, 0.15)
      box(1, 0.4, 0.3, carpet, 0, 0.2, -0.15)
      break
    case 'ledPanel':
      box(0.5, 1, 0.08, black, 0, 0.5, 0)
      box(0.48, 0.98, 0.005, '#1c2333', 0, 0.5, 0.041)
      break
    case 'projectionScreen':
      box(3.1, 2.1, 0.04, black, 0, 1.95, 0)
      box(3, 2, 0.01, '#d8dade', 0, 1.95, 0.021)
      for (const x of [-1.6, 1.6]) {
        cylinder(0.02, 3, carpet, x, 1.5, 0)
        box(0.04, 0.02, 0.8, carpet, x, 0.01, 0)
      }
      break
    case 'subwoofer':
      box(0.6, 0.6, 0.75, black, 0, 0.3, 0)
      cylinder(0.23, 0.01, cone, 0, 0.3, 0.376, Math.PI / 2)
      break
    case 'speaker':
      box(0.4, 0.65, 0.36, black, 0, 0.325, 0)
      cylinder(0.15, 0.01, cone, 0, 0.24, 0.181, Math.PI / 2)
      box(0.24, 0.1, 0.01, cone, 0, 0.53, 0.181)
      break
    case 'monitor':
      // Caixa inclinada, com a frente para o fundo do palco (−Z), olhando para cima
      box(0.55, 0.32, 0.42, black, 0, 0.25, 0, 0.5)
      box(0.5, 0.27, 0.01, cone, 0, 0.352, -0.187, 0.5)
      break
    case 'micStand':
      cylinder(0.14, 0.02, black, 0, 0.01, 0)
      cylinder(0.012, 1.45, black, 0, 0.735, 0)
      add(new CylinderGeometry(0.025, 0.018, 0.16, segments), '#3a3d42', 0, 1.5, 0.05, 0.6)
      break
    case 'person': {
      const skin = '#8a8f96'
      for (const x of [-0.1, 0.1]) cylinder(0.07, 0.85, skin, x, 0.425, 0)
      add(new CylinderGeometry(0.2, 0.16, 0.6, segments), skin, 0, 1.15, 0).scale.z = 0.6
      for (const x of [-0.26, 0.26]) cylinder(0.05, 0.6, skin, x, 1.12, 0)
      add(new SphereGeometry(0.11, segments, segments / 2), skin, 0, 1.63, 0)
      break
    }
    case 'drums': {
      const shell = '#7a1414'
      const brass = '#c9a227'
      cylinder(0.28, 0.4, shell, 0, 0.28, 0.25, Math.PI / 2)
      cylinder(0.18, 0.14, shell, -0.45, 0.62, 0.15)
      cylinder(0.14, 0.18, shell, -0.18, 0.78, 0.05)
      cylinder(0.15, 0.2, shell, 0.18, 0.78, 0.05)
      cylinder(0.22, 0.4, shell, 0.5, 0.4, 0.1)
      // Pratos nos pedestais: chimbal, ataque e condução
      for (const [x, y, z, r] of [[-0.7, 0.9, -0.05, 0.18], [-0.45, 1.3, -0.3, 0.25], [0.65, 1.2, -0.25, 0.28]] as const) {
        cylinder(r, 0.01, brass, x, y, z)
        cylinder(0.012, y, metal, x, y / 2, z)
      }
      cylinder(0.18, 0.5, black, 0, 0.25, -0.5)
      break
    }
    case 'keyboard':
      box(1.3, 0.12, 0.38, black, 0, 0.92, 0)
      box(1.2, 0.02, 0.16, '#e6e6e6', 0, 0.985, -0.08)
      for (const x of [-0.5, 0.5]) box(0.04, 0.86, 0.36, carpet, x, 0.43, 0)
      break
    case 'pulpit':
      box(0.7, 0.05, 0.55, '#4a3320', 0, 0.025, 0)
      box(0.6, 1.05, 0.45, '#6b4a2b', 0, 0.55, 0)
      box(0.7, 0.04, 0.55, '#7d5835', 0, 1.1, -0.02, -0.25)
      break
    case 'plant':
      add(new CylinderGeometry(0.2, 0.15, 0.4, segments), '#5a4636', 0, 0.2, 0)
      add(new SphereGeometry(0.32, segments, segments / 2), '#2f6b35', 0, 0.85, 0)
      add(new SphereGeometry(0.22, segments, segments / 2), '#3b7d40', 0.15, 1.15, 0.05)
      break
  }
  return { group, materials: [...materials.values()] }
}

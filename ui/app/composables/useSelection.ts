import type { Member } from '~/types/generated/Member'
import { divide, splitSteps, type Split } from '~/utils/selection'

const ALL = '*'

// Seleção de fixtures do Seletor (VC-7). Fica no aparelho, e não no servidor: o botão de alvo manda ao
// servidor a lista já resolvida (quem está no grupo na hora).
// A seleção guarda os ITENS tocados (grupo, subgrupo ou seleção salva), e não as fixtures: assim um grupo
// não "acende" só porque os subgrupos dele estão escolhidos, e a divisão vale para cada item à parte
// (as asas de cada subgrupo, e não as do conjunto).
export function useSelection() {
  const { state } = useEngine()
  const picks = useState<string[]>('selectedItems', () => [])
  // Sobe a cada toque de alvo com a seleção vazia, para o Seletor piscar
  const flash = useState('selectorFlash', () => 0)
  const project = computed(() => state.value.project)

  /** Fixtures de agora de um grupo de fixtures ou de uma seleção salva. */
  const groupIds = (id: string) => groupFixtures(project.value, id).map((f) => f.id)
  const membersIds = (members: Member[]) =>
    members.flatMap((m) => ('fixture' in m ? project.value.fixtures.filter((f) => f.id === m.fixture).map((f) => f.id) : groupIds(m.group)))
  const selectionIds = (id: string) => [...new Set(membersIds(project.value.selections.find((s) => s.id === id)?.members ?? []))]

  // Fixtures de cada item escolhido, na ordem do projeto (o que não existe mais sai sozinho)
  const itemIds = (key: string) => {
    const ids = key === ALL ? project.value.fixtures.map((f) => f.id) : project.value.selections.some((s) => s.id === key) ? selectionIds(key) : groupIds(key)
    return project.value.fixtures.map((f) => f.id).filter((i) => ids.includes(i))
  }
  const items = computed(() => picks.value.map(itemIds).filter((ids) => ids.length))

  // Divisão (P3): `mode` e `step` dizem onde ela está; cada item é dividido à parte e o resultado se soma
  const mode = useState<Split | null>('selectionSplit', () => null)
  const step = useState('selectionStep', () => 0)
  const selected = computed(() => {
    const chosen = new Set(items.value.flatMap((ids) => (mode.value ? divide(ids, mode.value, step.value % splitSteps(ids.length, mode.value)) : ids)))
    return project.value.fixtures.map((f) => f.id).filter((i) => chosen.has(i))
  })

  /** Fica "ligado" quando o item foi tocado (e não quando suas fixtures já estão na seleção por outro caminho). */
  const isOn = (key: string) => picks.value.includes(key)
  // Escolher à mão acaba a divisão
  const set = (keys: string[]) => {
    picks.value = keys
    mode.value = null
  }
  /** Tocar soma o item; se já está, tira. No modo solo, o item escolhido é o único (e tocar nele o tira). */
  const toggle = (key: string, solo = false) =>
    set(isOn(key) ? picks.value.filter((k) => k !== key) : solo ? [key] : [...picks.value, key])
  const all = () => set([ALL])
  const clear = () => set([])

  function split(m: Split, s = 0) {
    if (!items.value.length) return flash.value++
    mode.value = m
    step.value = s
  }
  /** Anterior (-1) e próxima (1): percorre as asas ou "uma por vez" (começa nesta se ainda não está), dando a volta. */
  function walk(by: 1 | -1) {
    const m = mode.value === 'wings' ? 'wings' : 'one'
    const n = Math.max(0, ...items.value.map((ids) => splitSteps(ids.length, m)))
    if (!n) return flash.value++
    split(m, mode.value === m ? (step.value + by + n) % n : by > 0 ? 0 : n - 1)
  }
  const restore = () => (mode.value = null)

  return { selected, flash, groupIds, selectionIds, isOn, toggle, all, clear, split, walk, restore, mode, divided: computed(() => !!mode.value) }
}

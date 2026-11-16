// Divisões da seleção (VC-7, P3): contas puras sobre a lista de fixtures na ordem do projeto.
export type Split = 'first' | 'second' | 'odd' | 'even' | 'wings' | 'one'

/** Quantos passos o modo tem: as asas andam de par em par (de fora para dentro) e "uma por vez", de uma em uma. */
export const splitSteps = (n: number, mode: Split) => (mode === 'wings' ? Math.ceil(n / 2) : n)

/** O pedaço de `ids` do modo; `step` só vale em `wings` (o par `step` de fora para dentro) e `one` (a fixture `step`). */
export function divide(ids: string[], mode: Split, step = 0): string[] {
  const n = ids.length
  switch (mode) {
    case 'first':
      return ids.slice(0, Math.ceil(n / 2))
    case 'second':
      return ids.slice(Math.ceil(n / 2))
    case 'odd':
      return ids.filter((_, i) => i % 2 === 0)
    case 'even':
      return ids.filter((_, i) => i % 2 === 1)
    case 'wings':
      return [...new Set([ids[step], ids[n - 1 - step]])].filter((i): i is string => !!i)
    case 'one':
      return ids[step] ? [ids[step]] : []
  }
}

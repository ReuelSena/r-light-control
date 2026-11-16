import type { InputControl } from '~/types/generated/InputControl'
import type { Op } from '~/types/generated/Op'
import type { StepAction } from '~/types/generated/StepAction'
import type { VcWidget } from '~/types/generated/VcWidget'

type Widget<T extends VcWidget['type']> = Extract<VcWidget, { type: T }>

// Operação do Virtual Console. O aparelho manda só "apertou" e "soltou" de cada elemento, e o servidor
// aplica o que ele faz (A2): assim ele solta o flash de um aparelho que cai, e o MIDI da F12 usa o
// mesmo caminho. Ficam aqui o que é do aparelho: o tap, medido onde o dedo está, e o toque (ir para o
// passo) e o toque longo (carregar o passo) num passo da cue list.
const taps = new Map<string, number>() // último toque no TAP de cada speed dial
const bpmTaps: number[] = []

export function useConsole() {
  const { state, send } = useEngine()
  const selection = useSelection()
  // Modo Preparar (VC-6), por aparelho: as operações dos elementos vão como `staged` e os elementos
  // mostram o estado preparado (do servidor, o mesmo em todos os aparelhos)
  const prepare = useState('consolePrepare', () => false)
  const view = computed(() => (prepare.value ? state.value.staged : state.value))
  const op = (o: Op) => send(prepare.value ? { op: 'staged', inner: o } : o)
  /** Algo preparado (ou a cópia do ao vivo, que Executar substitui): o que o Executar leva para a luz. */
  const hasStaged = computed(() => {
    const s = state.value.staged
    return s.full || !!s.running.length || !!s.widgetValues.length || !!s.targeted.length
  })
  /** Liga ou desliga o modo; ao ligar com o preparado vazio, o preview já vem com o que roda na luz. */
  const setPrepare = (on: boolean) => {
    prepare.value = on
    if (on && !hasStaged.value) send({ op: 'seedStaged' })
  }
  const value = (id: string) => view.value.widgetValues.find((v) => v.widget === id)?.value
  const setValue = (widget: string, value: number[]) => op({ op: 'setWidgetValue', widget, value })
  const running = (id: string | null) => !!id && view.value.running.includes(id)
  const paused = (id: string | null) => !!id && view.value.paused.includes(id)
  const chaserOf = (w: Widget<'cueList'>) => {
    const fn = state.value.project.functions.find((f) => f.id === w.chaser)
    return fn?.type === 'chaser' || fn?.type === 'sequence' ? fn : undefined
  }

  /** Botão de alvo (VC-7): a criação liga só nas fixtures da seleção do aparelho. */
  const isTarget = (w: VcWidget) => w.type === 'button' && w.target && w.action === 'function'
  /** Quanto da seleção a criação do botão cobre: tudo (`on`), parte ou nada (`off`). No Preparar, o preparado. */
  function targetState(w: Widget<'button'>) {
    const lit = new Set(view.value.targeted.filter((r) => r.function === w.function).flatMap((r) => r.fixtures))
    const mine = selection.selected.value.filter((i) => lit.has(i))
    return !mine.length ? 'off' : mine.length === selection.selected.value.length ? 'on' : 'part'
  }
  function pressTarget(w: Widget<'button'>) {
    const fixtures = selection.selected.value
    if (!w.function) return
    if (!fixtures.length) return void selection.flash.value++
    if (targetState(w) === 'on') op({ op: 'targetOff', function: w.function, fixtures })
    else op({ op: 'targetOn', function: w.function, fixtures, category: w.category })
  }

  /** O botão aceso: a criação rodando (ou pausada, no Pausar), o blackout ligado ou as fixtures apagadas. */
  function active(w: Widget<'button'>) {
    if (isTarget(w)) return targetState(w) !== 'off'
    if (w.action === 'function' || w.action === 'exclusive') return running(w.function)
    if (w.action === 'pause') return paused(w.function)
    if (w.action === 'blackout') return state.value.master.blackout
    return w.action === 'blackoutFixtures' && !!value(w.id)
  }

  /** Apertou (`down`) ou soltou uma parte do elemento. */
  function press(w: VcWidget, control: InputControl, down: boolean) {
    if (w.type === 'button' && isTarget(w)) return down ? pressTarget(w) : undefined
    // Blackout e Stop All agem sempre na luz real
    const o: Op = { op: 'pressWidget', widget: w.id, control, down }
    const real = w.type === 'button' && (w.action === 'blackout' || w.action === 'stopAll')
    send(prepare.value && !real ? { op: 'staged', inner: o } : o)
  }

  /** Passo atual da cue list, vindo do motor; -1 com o chaser parado. */
  function currentStep(w: Widget<'cueList'>) {
    if (!running(w.chaser)) return -1
    // Preparado: o servidor não publica o passo da sombra; vale o real se já roda, senão o primeiro
    if (prepare.value && !state.value.running.includes(w.chaser!)) return 0
    return state.value.chaserSteps.find((c) => c.chaser === w.chaser)?.step ?? -1
  }

  /** Toque num passo da cue list; o chaser parado passa a rodar. */
  function step(w: Widget<'cueList'>, action: StepAction) {
    if (w.chaser && chaserOf(w)?.steps.length) op({ op: 'stepChaser', id: w.chaser, action })
  }

  /** Passo carregado para o próximo Go (-1 sem nenhum). */
  const loadedStep = (w: Widget<'cueList'>) => view.value.loadedSteps.find((l) => l.chaser === w.chaser)?.step ?? -1

  /** Toque longo num passo: carrega (ou, no já carregado, descarrega) o passo do próximo Go. */
  function load(w: Widget<'cueList'>, i: number) {
    if (w.chaser) op({ op: 'loadStep', id: w.chaser, step: loadedStep(w) === i ? null : i })
  }

  /** TAP do speed dial: seguindo o BPM, é o tap do BPM master; em segundos, o intervalo entre dois
   * toques vira a duração. */
  function tap(w: Widget<'speedDial'>) {
    if (w.beats) return tapBpm()
    const now = Date.now()
    const last = taps.get(w.id)
    taps.set(w.id, now)
    if (!last || now - last > 4000) return
    const [fadeIn = 0, fadeOut = 0] = value(w.id) ?? []
    setValue(w.id, [fadeIn, fadeOut, now - last])
  }

  /** Tap do BPM: a média dos últimos toques; uma pausa de 2 s recomeça a contagem. */
  function tapBpm() {
    const now = Date.now()
    if (now - (bpmTaps.at(-1) ?? 0) > 2000) bpmTaps.length = 0
    bpmTaps.push(now)
    if (bpmTaps.length > 5) bpmTaps.shift()
    if (bpmTaps.length > 1) send({ op: 'setBpm', bpm: Math.round((60000 * (bpmTaps.length - 1)) / (now - bpmTaps[0]!)) })
  }

  /** Entrada externa (tecla; o MIDI na F12): `down` ao apertar e `false` ao soltar. */
  function input(w: VcWidget, control: InputControl, down: boolean) {
    if (w.type !== 'speedDial') press(w, control, down)
    else if (down && control === 'tap') tap(w)
  }

  /** O elemento tem algo preparado (para o destaque no modo Preparar). */
  function staged(w: VcWidget) {
    const s = state.value.staged
    const target = w.type === 'button' ? w.function : w.type === 'cueList' ? w.chaser : w.type === 'animation' ? w.matrix : null
    const on = (id: string) => s.running.includes(id) || s.paused.includes(id) || s.targeted.some((r) => r.function === id)
    return s.widgetValues.some((v) => v.widget === w.id) || (!!target && on(target))
  }

  return { isTarget, targetState, prepare, setPrepare, hasStaged, op, staged, value, setValue, running, paused, active, press, chaserOf, currentStep, step, loadedStep, load, tap, tapBpm, input }
}

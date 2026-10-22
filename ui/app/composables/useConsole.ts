import type { InputControl } from '~/types/generated/InputControl'
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
  const value = (id: string) => state.value.widgetValues.find((v) => v.widget === id)?.value
  const setValue = (widget: string, value: number[]) => send({ op: 'setWidgetValue', widget, value })
  const running = (id: string | null) => !!id && state.value.running.includes(id)
  const paused = (id: string | null) => !!id && state.value.paused.includes(id)
  const chaserOf = (w: Widget<'cueList'>) => {
    const fn = state.value.project.functions.find((f) => f.id === w.chaser)
    return fn?.type === 'chaser' || fn?.type === 'sequence' ? fn : undefined
  }

  /** O botão aceso: a criação rodando (ou pausada, no Pausar), o blackout ligado ou as fixtures apagadas. */
  function active(w: Widget<'button'>) {
    if (w.action === 'function' || w.action === 'exclusive') return running(w.function)
    if (w.action === 'pause') return paused(w.function)
    if (w.action === 'blackout') return state.value.master.blackout
    return w.action === 'blackoutFixtures' && !!value(w.id)
  }

  /** Apertou (`down`) ou soltou uma parte do elemento. */
  function press(w: VcWidget, control: InputControl, down: boolean) {
    send({ op: 'pressWidget', widget: w.id, control, down })
  }

  /** Passo atual da cue list, vindo do motor; -1 com o chaser parado. */
  function currentStep(w: Widget<'cueList'>) {
    if (!running(w.chaser)) return -1
    return state.value.chaserSteps.find((c) => c.chaser === w.chaser)?.step ?? -1
  }

  /** Toque num passo da cue list; o chaser parado passa a rodar. */
  function step(w: Widget<'cueList'>, action: StepAction) {
    if (w.chaser && chaserOf(w)?.steps.length) send({ op: 'stepChaser', id: w.chaser, action })
  }

  /** Passo carregado para o próximo Go (-1 sem nenhum). */
  const loadedStep = (w: Widget<'cueList'>) => state.value.loadedSteps.find((l) => l.chaser === w.chaser)?.step ?? -1

  /** Toque longo num passo: carrega (ou, no já carregado, descarrega) o passo do próximo Go. */
  function load(w: Widget<'cueList'>, i: number) {
    if (w.chaser) send({ op: 'loadStep', id: w.chaser, step: loadedStep(w) === i ? null : i })
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

  return { value, setValue, running, paused, active, press, chaserOf, currentStep, step, loadedStep, load, tap, tapBpm, input }
}

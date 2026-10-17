import type { InputControl } from '~/types/generated/InputControl'
import type { VcWidget } from '~/types/generated/VcWidget'

type Widget<T extends VcWidget['type']> = Extract<VcWidget, { type: T }>

// Operação do Virtual Console: o que cada elemento faz ao toque e às entradas externas
// (teclado agora; MIDI na F12). Usado pelos elementos e pelos atalhos de teclado da tela.
const taps = new Map<string, number>() // último toque no TAP de cada speed dial
const bpmTaps: number[] = []

export function useConsole() {
  const { state, send } = useEngine()
  const value = (id: string) => state.value.widgetValues.find((v) => v.widget === id)?.value
  const setValue = (widget: string, value: number[]) => send({ op: 'setWidgetValue', widget, value })
  const running = (id: string | null) => !!id && state.value.running.includes(id)
  const chaserOf = (w: Widget<'cueList'>) => {
    const fn = state.value.project.functions.find((f) => f.id === w.chaser)
    return fn?.type === 'chaser' || fn?.type === 'sequence' ? fn : undefined
  }

  function active(w: Widget<'button'>) {
    if (w.action === 'function') return running(w.function)
    if (w.action === 'blackout') return state.value.master.blackout
    return w.action === 'blackoutFixtures' && !!value(w.id)
  }

  /** Liga ou desliga o botão. Numa moldura solo, ligar uma criação para as outras da moldura. */
  function setButton(w: Widget<'button'>, on: boolean) {
    const widgets = state.value.project.console.widgets
    switch (w.action) {
      case 'function': {
        if (!w.function) return
        const frame = widgets.find((x) => x.id === w.frame)
        if (on && frame?.type === 'frame' && frame.solo) {
          for (const b of widgets) {
            if (b.type === 'button' && b.frame === w.frame && b.function && b.function !== w.function && running(b.function)) {
              send({ op: 'setFunctionRunning', id: b.function, running: false })
            }
          }
        }
        send({ op: 'setFunctionRunning', id: w.function, running: on })
        return
      }
      case 'blackout':
        send({ op: 'setBlackout', on })
        return
      case 'blackoutFixtures':
        setValue(w.id, on ? [1] : [])
        return
      case 'stopAll':
        if (on) send({ op: 'stopAllFunctions' })
    }
  }

  /** Passo atual da cue list: o escolhido à mão ou o que o tempo dita; -1 com o chaser parado. */
  function currentStep(w: Widget<'cueList'>, now: number) {
    const chaser = chaserOf(w)
    if (!chaser || !running(chaser.id)) return -1
    return value(w.id)?.[0] ?? chaserStep(chaser, now)
  }

  /** Vai para um passo (dá a volta nas pontas) e põe o chaser para rodar. */
  function goTo(w: Widget<'cueList'>, step: number) {
    const chaser = chaserOf(w)
    const n = chaser?.steps.length ?? 0
    if (!chaser || !n) return
    setValue(w.id, [(step + n) % n])
    if (!running(chaser.id)) send({ op: 'setFunctionRunning', id: chaser.id, running: true })
  }

  /** Play volta a andar no tempo; stop para o chaser. */
  function play(w: Widget<'cueList'>, on: boolean) {
    setValue(w.id, [])
    if (w.chaser) send({ op: 'setFunctionRunning', id: w.chaser, running: on })
  }

  /** TAP do speed dial: o intervalo entre dois toques vira a duração. */
  function tap(w: Widget<'speedDial'>) {
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

  /** Entrada externa (tecla ou MIDI): `down` ao apertar e `false` ao soltar. */
  function input(w: VcWidget, control: InputControl, down: boolean) {
    if (w.type === 'button') {
      if (w.action === 'stopAll' || w.mode === 'toggle') {
        if (down) setButton(w, w.action === 'stopAll' || !active(w))
      } else setButton(w, down)
    }
    if (!down) return
    if (w.type === 'cueList') {
      const step = currentStep(w, Date.now())
      if (control === 'main') play(w, step < 0)
      if (control === 'stop') play(w, false)
      if (control === 'next') goTo(w, step + 1)
      // Parada, "anterior" começa pelo último passo
      if (control === 'previous') goTo(w, step < 0 ? -1 : step - 1)
    }
    if (w.type === 'speedDial' && control === 'tap') tap(w)
  }

  return { value, setValue, active, setButton, chaserOf, currentStep, goTo, play, tap, tapBpm, input }
}

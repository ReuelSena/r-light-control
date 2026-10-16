// Arrasto relativo (E9) usado por fader, knob e XY pad: captura o ponteiro (um dedo por
// controle, mesmo que ele saia de cima), entrega o deslocamento a cada movimento e detecta toque duplo.
// Uso: <div v-on="drag">.
export function useDrag(onMove: (dx: number, dy: number) => void, onDoubleTap: () => void) {
  let pointerId: number | undefined
  let lastX = 0
  let lastY = 0
  let moved = 0
  let lastTap = 0

  return {
    pointerdown(e: PointerEvent) {
      if (pointerId !== undefined) return
      pointerId = e.pointerId
      ;(e.currentTarget as HTMLElement).setPointerCapture(e.pointerId)
      lastX = e.clientX
      lastY = e.clientY
      moved = 0
    },
    pointermove(e: PointerEvent) {
      if (e.pointerId !== pointerId) return
      const dx = e.clientX - lastX
      const dy = e.clientY - lastY
      lastX = e.clientX
      lastY = e.clientY
      moved += Math.abs(dx) + Math.abs(dy)
      onMove(dx, dy)
    },
    // Dispara ao soltar, ao cancelar o toque ou ao perder a captura: o controle nunca fica preso.
    lostpointercapture(e: PointerEvent) {
      if (e.pointerId !== pointerId) return
      pointerId = undefined
      if (moved < 4) {
        if (e.timeStamp - lastTap < 300) onDoubleTap()
        lastTap = e.timeStamp
      }
    },
  }
}

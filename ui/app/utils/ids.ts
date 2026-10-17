/**
 * UUIDv7 (A4): 48 bits do relógio em ms e o resto aleatório, então os ids saem em ordem de criação.
 * Gerador próprio porque `crypto.randomUUID` não existe no celular em http (fora de contexto seguro);
 * `crypto.getRandomValues` existe.
 */
export function uuidv7() {
  const b = crypto.getRandomValues(new Uint8Array(16))
  const ms = Date.now()
  for (let i = 0; i < 6; i++) b[i] = Math.floor(ms / 2 ** (8 * (5 - i))) % 256
  b[6] = 0x70 | (b[6]! & 0x0f)
  b[8] = 0x80 | (b[8]! & 0x3f)
  const hex = [...b].map((x) => x.toString(16).padStart(2, '0')).join('')
  return `${hex.slice(0, 8)}-${hex.slice(8, 12)}-${hex.slice(12, 16)}-${hex.slice(16, 20)}-${hex.slice(20)}`
}

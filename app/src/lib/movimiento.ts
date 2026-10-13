// Las transiciones de Svelte no respetan `prefers-reduced-motion` solas: con movimiento
// reducido, duración cero.
const reducido =
  typeof window !== "undefined" && window.matchMedia("(prefers-reduced-motion: reduce)").matches;

export function duracion(ms: number): number {
  return reducido ? 0 : ms;
}

export const EASE = (t: number) => 1 - Math.pow(1 - t, 4);

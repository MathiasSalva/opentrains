// Presentación de una lista de Arribos: el próximo destacado y el resto agrupado por Ramal.
import type { Arribo, Ramal, Situacion } from "./api";

export interface GrupoRamal {
  ramal: Ramal;
  arribos: Arribo[];
}

export function clave(arribo: Arribo): string {
  return `${arribo.numero}-${arribo.momento}`;
}

/** El próximo Arribo que va a pasar: el primero no cancelado (la lista viene ordenada). */
export function proximo(arribos: Arribo[]): Arribo | null {
  return arribos.find((a) => a.situacion !== "cancelado") ?? null;
}

/** Agrupa por Ramal conservando el orden: cada grupo aparece donde está su primer Arribo. */
export function agruparPorRamal(arribos: Arribo[]): GrupoRamal[] {
  const grupos = new Map<number, GrupoRamal>();
  for (const arribo of arribos) {
    const grupo = grupos.get(arribo.ramal.id) ?? { ramal: arribo.ramal, arribos: [] };
    grupo.arribos.push(arribo);
    grupos.set(arribo.ramal.id, grupo);
  }
  return [...grupos.values()];
}

/** Segundos que faltan para el Arribo, contra el reloj local. */
export function segundosRestantes(arribo: Arribo, ahora: number): number {
  return Math.round((new Date(arribo.momento).getTime() - ahora) / 1000);
}

/** Último minuto antes del Arribo: se muestra como "Arribando". */
export function estaArribando(arribo: Arribo, ahora: number): boolean {
  return arribo.situacion !== "cancelado" && segundosRestantes(arribo, ahora) <= 60;
}

interface Presentacion {
  etiqueta: (arribo: Arribo) => string;
}

// Un solo mapa Situación → presentación, compartido por todos los componentes.
export const SITUACIONES: Record<Situacion, Presentacion> = {
  cancelado: { etiqueta: () => "Cancelado" },
  en_anden: { etiqueta: () => "En andén" },
  demorado: {
    etiqueta: (a) => (a.demora_minutos === null ? "Demorado" : `Demorado +${a.demora_minutos} min`),
  },
  en_camino: { etiqueta: () => "En camino" },
  programado: { etiqueta: () => "Programado" },
  // Estado no catalogado: se muestra el texto del operador tal cual.
  desconocido: { etiqueta: (a) => a.estado },
};

export function etiquetaSituacion(arribo: Arribo, ahora: number): string {
  if (claseSituacion(arribo, ahora) === "arribando") return "Arribando";
  return SITUACIONES[arribo.situacion].etiqueta(arribo);
}

export type ClaseSituacion = Situacion | "arribando";

/** Clase visual: la Situación, o "arribando" en el último minuto (salvo en andén). */
export function claseSituacion(arribo: Arribo, ahora: number): ClaseSituacion {
  if (arribo.situacion !== "en_anden" && estaArribando(arribo, ahora)) return "arribando";
  return arribo.situacion;
}

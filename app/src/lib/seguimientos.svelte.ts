// Estado compartido de los Seguimientos (Servicios que el usuario sigue para recibir Avisos).
import { api, type Seguimiento } from "./api";

export const seguimientos = $state({
  lista: [] as Seguimiento[],
  /** Estación abierta: los Seguimientos nuevos se crean en ella. */
  estacion: null as { id: number; nombre: string } | null,
  /** Última anticipación elegida en cada Estación (de las Preferencias). */
  anticipaciones: {} as Record<string, number>,
});

/** La anticipación que se usó la última vez en la Estación abierta. */
export function anticipacionSugerida(): number | null {
  const estacion = seguimientos.estacion?.id;
  return estacion === undefined ? null : (seguimientos.anticipaciones[estacion] ?? null);
}

export function anticipacion(numero: number): number | null {
  const estacion = seguimientos.estacion?.id;
  const s = seguimientos.lista.find((s) => s.estacion === estacion && s.numero === numero);
  return s ? s.anticipacion_minutos : null;
}

/** Los Seguimientos terminan solos cuando el tren pasa: se recargan en cada refresco. */
export async function recargarSeguimientos() {
  try {
    seguimientos.lista = await api.seguimientos();
  } catch {
    // Sin la lista, las campanas quedan como estaban hasta el próximo refresco.
  }
}

export async function seguir(numero: number, destino: string, anticipacion_minutos: number) {
  const estacion = seguimientos.estacion;
  if (!estacion) return;
  seguimientos.lista = await api.seguir({
    estacion: estacion.id,
    estacion_nombre: estacion.nombre,
    numero,
    destino,
    anticipacion_minutos,
  });
  seguimientos.anticipaciones[estacion.id] = anticipacion_minutos;
}

export async function dejarDeSeguir(numero: number) {
  const estacion = seguimientos.estacion;
  if (!estacion) return;
  seguimientos.lista = await api.dejarDeSeguir(estacion.id, numero);
}

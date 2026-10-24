// Contrato con los comandos Tauri de `src-tauri/src/lib.rs`.
import { invoke } from "@tauri-apps/api/core";

export interface Linea { id: number; nombre: string }
export interface Ramal { id: number; nombre: string }
export interface Estacion {
  id: number;
  nombre: string;
  orden: number;
  latitud: number;
  longitud: number;
  /** Ramales que pasan por la Estación; vacío en catálogos guardados antes de saberlo. */
  ramales?: number[];
}

export interface Coordenadas { latitud: number; longitud: number }

export interface Favorito {
  estacion: number;
  estacion_nombre: string;
  ramal: Ramal | null;
  /** Ausentes en Favoritos guardados antes de que existieran las distancias. */
  coordenadas?: Coordenadas | null;
  /** Ramales que pasan por la Estación, para sus Noticias. */
  ramales?: number[];
}

export interface Preferencias {
  favoritos: Favorito[];
  ultimo_abierto: Favorito | null;
  /** Última anticipación elegida al seguir un Servicio, por id de Estación. */
  anticipaciones?: Record<string, number>;
  /** El usuario cerró el aviso de Ubicación desactivada. */
  aviso_ubicacion_oculto?: boolean;
}

/** Noticia del operador para la Estación abierta. */
export interface Noticia {
  id: number;
  texto: string;
  /** Se muestra entera y no se puede descartar. */
  destacada: boolean;
}

export interface EstacionCercana { estacion: Estacion; distancia: string }

export type Situacion =
  | "cancelado"
  | "en_anden"
  | "demorado"
  | "en_camino"
  | "programado"
  | "desconocido";

export interface Arribo {
  numero: number;
  ramal: Ramal;
  destino: string;
  /** Texto del Estado del Servicio tal como lo informa el operador. */
  estado: string;
  situacion: Situacion;
  formacion: string | null;
  anden: string | null;
  /** Si hubo un Cambio de andén, el andén que tenía antes. */
  anden_anterior: string | null;
  /** Noticia del Servicio. */
  noticia: string | null;
  programada: string;
  demora_minutos: number | null;
  momento: string;
}

export interface Instalaciones {
  direccion: string | null;
  boleteria: string | null;
  ascensor: boolean;
  rampa: boolean;
  banos: boolean;
  wifi: boolean;
  terminal_sube: boolean;
  conexiones: string | null;
  colectivos: string | null;
}

export interface Seguimiento {
  estacion: number;
  estacion_nombre: string;
  numero: number;
  destino: string;
  anticipacion_minutos: number;
}

export type ErrorApp =
  | { tipo: "fuente"; detalle: { tipo: "sin_conexion"; detalle: string } | { tipo: "fuente_cambiada" } | { tipo: "respuesta_invalida"; detalle: string } }
  | { tipo: "almacenamiento"; detalle: string };

export function describirError(error: unknown): string {
  const e = error as ErrorApp;
  if (e?.tipo === "fuente") {
    switch (e.detalle.tipo) {
      case "fuente_cambiada":
        return "La fuente de datos cambió; la app necesita actualizarse.";
      case "sin_conexion":
        return "Sin conexión con la fuente de datos.";
      case "respuesta_invalida":
        return "La fuente de datos respondió algo inesperado.";
    }
  }
  if (e?.tipo === "almacenamiento") return `No se pudieron guardar los datos locales: ${e.detalle}`;
  return String(error);
}

export function esFuenteCambiada(error: unknown): boolean {
  const e = error as ErrorApp;
  return e?.tipo === "fuente" && e.detalle.tipo === "fuente_cambiada";
}

export const api = {
  lineas: () => invoke<Linea[]>("lineas"),
  ramales: (linea: number) => invoke<Ramal[]>("ramales", { linea }),
  estaciones: (ramal: number) => invoke<Estacion[]>("estaciones", { ramal }),
  arribos: (favorito: Favorito) => invoke<Arribo[]>("arribos", { favorito }),
  /** Como `arribos`, pero no cambia el tren que muestra la bandeja. */
  arribosTablero: (favorito: Favorito) => invoke<Arribo[]>("arribos_tablero", { favorito }),
  /** Abre el tablero en esa Estación, o lo cambia a ella si ya está abierto. */
  abrirTablero: (favorito: Favorito) => invoke<void>("abrir_tablero", { favorito }),
  tablero: () => invoke<Favorito | null>("tablero"),
  pantallaCompletaTablero: (activa: boolean) => invoke<void>("pantalla_completa_tablero", { activa }),
  noticias: (favorito: Favorito) => invoke<Noticia[]>("noticias", { favorito }),
  descartarNoticia: (id: number) => invoke<Preferencias>("descartar_noticia", { id }),
  instalaciones: (estacion: number) => invoke<Instalaciones | null>("instalaciones", { estacion }),
  preferencias: () => invoke<Preferencias>("preferencias"),
  agregarFavorito: (favorito: Favorito) => invoke<Preferencias>("agregar_favorito", { favorito }),
  quitarFavorito: (favorito: Favorito) => invoke<Preferencias>("quitar_favorito", { favorito }),
  abrir: (favorito: Favorito) => invoke<Preferencias>("abrir", { favorito }),
  seguimientos: () => invoke<Seguimiento[]>("seguimientos"),
  seguir: (pedido: Seguimiento) => invoke<Seguimiento[]>("seguir", { pedido }),
  dejarDeSeguir: (estacion: number, numero: number) =>
    invoke<Seguimiento[]>("dejar_de_seguir", { estacion, numero }),
  /** `false` si no hay Ubicación conocida (desactivada, denegada o sin señal). */
  ubicar: () => invoke<boolean>("ubicar"),
  /** Textos como "a 850 m", en el orden de `destinos`; `null` sin Ubicación. */
  distancias: (destinos: Coordenadas[]) => invoke<string[] | null>("distancias", { destinos }),
  cercanas: () => invoke<EstacionCercana[] | null>("cercanas"),
  ocultarAvisoUbicacion: () => invoke<Preferencias>("ocultar_aviso_ubicacion"),
  abrirAjustesUbicacion: () => invoke<void>("abrir_ajustes_ubicacion"),
};

export function coordenadas(e: Estacion): Coordenadas {
  return { latitud: e.latitud, longitud: e.longitud };
}

export function mismoFavorito(a: Favorito, b: Favorito): boolean {
  return a.estacion === b.estacion && (a.ramal?.id ?? null) === (b.ramal?.id ?? null);
}

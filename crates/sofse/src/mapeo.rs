//! Respuestas JSON de SOFSE → dominio.
//!
//! Los tipos `Api*` reflejan la forma de la API y no salen de este módulo. Son
//! permisivos a propósito: la API es no oficial y los campos opcionales cambian según
//! el estado del Servicio.

use chrono::{DateTime, Duration, FixedOffset, NaiveDateTime, Utc};
use opentrains_core::dominio::{
    AlcanceNoticia, Arribo, Estacion, EstacionId, EstadoServicio, Instalaciones, Linea, LineaId,
    Noticia, Ramal, RamalId, Servicio,
};
use opentrains_core::puertos::SourceError;
use serde::de::DeserializeOwned;
use serde::Deserialize;

fn parsear<T: DeserializeOwned>(json: &str) -> Result<T, SourceError> {
    serde_json::from_str(json).map_err(|e| SourceError::RespuestaInvalida(e.to_string()))
}

#[derive(Deserialize)]
struct ApiIdNombre {
    id: u32,
    nombre: String,
}

pub fn lineas(json: &str) -> Result<Vec<Linea>, SourceError> {
    let gerencias: Vec<ApiIdNombre> = parsear(json)?;
    Ok(gerencias
        .into_iter()
        .map(|g| Linea {
            id: LineaId(g.id),
            nombre: g.nombre,
        })
        .collect())
}

pub fn ramales(json: &str) -> Result<Vec<Ramal>, SourceError> {
    let ramales: Vec<ApiIdNombre> = parsear(json)?;
    Ok(ramales
        .into_iter()
        .map(|r| Ramal {
            id: RamalId(r.id),
            nombre: r.nombre,
        })
        .collect())
}

#[derive(Deserialize)]
struct ApiEstacion {
    nombre: String,
    id_estacion: String,
    orden: String,
    latitud: String,
    longitud: String,
    /// Lista de ids, o esa misma lista escrita como string JSON (estaciones cercanas).
    #[serde(default)]
    incluida_en_ramales: serde_json::Value,
}

/// Estaciones de un Ramal, en el orden del recorrido.
pub fn estaciones(json: &str) -> Result<Vec<Estacion>, SourceError> {
    let mut estaciones = estaciones_cercanas(json)?;
    estaciones.sort_by_key(|e| e.orden);
    Ok(estaciones)
}

/// Estaciones cercanas a un punto, en el orden de la API (de la más cercana a la más lejana).
pub fn estaciones_cercanas(json: &str) -> Result<Vec<Estacion>, SourceError> {
    let api: Vec<ApiEstacion> = parsear(json)?;
    api.into_iter()
        .map(|e| {
            Ok(Estacion {
                id: EstacionId(numero(&e.id_estacion)?),
                orden: numero(&e.orden)?,
                latitud: decimal(&e.latitud)?,
                longitud: decimal(&e.longitud)?,
                ramales: ids_de_ramales(&e.incluida_en_ramales),
                nombre: e.nombre,
            })
        })
        .collect()
}

fn ids_de_ramales(valor: &serde_json::Value) -> Vec<RamalId> {
    let lista: Vec<u32> = match valor {
        serde_json::Value::String(texto) => serde_json::from_str(texto).unwrap_or_default(),
        otro => serde_json::from_value(otro.clone()).unwrap_or_default(),
    };
    lista.into_iter().map(RamalId).collect()
}

fn numero(texto: &str) -> Result<u32, SourceError> {
    texto
        .trim()
        .parse()
        .map_err(|_| SourceError::RespuestaInvalida(format!("número inválido: {texto:?}")))
}

fn decimal(texto: &str) -> Result<f64, SourceError> {
    texto
        .trim()
        .parse()
        .map_err(|_| SourceError::RespuestaInvalida(format!("coordenada inválida: {texto:?}")))
}

#[derive(Deserialize)]
struct ApiArribos {
    results: Vec<ApiResultado>,
}

#[derive(Deserialize)]
struct ApiResultado {
    arribo: ApiArribo,
    servicio: ApiServicio,
}

#[derive(Deserialize)]
struct ApiArribo {
    anden: Option<ApiAnden>,
    #[serde(default)]
    llegada: Option<ApiHorario>,
    #[serde(default)]
    salida: Option<ApiHorario>,
    segundos: Option<i64>,
}

#[derive(Deserialize)]
struct ApiAnden {
    nombre: String,
}

#[derive(Deserialize)]
struct ApiHorario {
    programada: Option<DateTime<Utc>>,
    estimada: Option<DateTime<Utc>>,
}

#[derive(Deserialize)]
struct ApiServicio {
    numero: u32,
    ramal: ApiIdNombre,
    cancelacion: Option<serde_json::Value>,
    equipo: Option<ApiEquipo>,
    #[serde(default)]
    oculto: bool,
    leyenda: Option<String>,
    desde: Option<ApiDesde>,
    hasta: Option<ApiHasta>,
}

#[derive(Deserialize)]
struct ApiEquipo {
    nombre: String,
}

#[derive(Deserialize)]
struct ApiDesde {
    estado: Option<ApiIdNombre>,
}

#[derive(Deserialize)]
struct ApiHasta {
    estacion: Option<ApiNombre>,
}

#[derive(Deserialize)]
struct ApiNombre {
    nombre: Option<String>,
}

/// `recibido` es el momento (reloj local) en que llegó la respuesta: la cuenta regresiva
/// `segundos` se ancla ahí y no al `timestamp` del servidor, para que un desfase entre
/// relojes no la corra.
pub fn arribos(json: &str, recibido: DateTime<Utc>) -> Result<Vec<Arribo>, SourceError> {
    let api: ApiArribos = parsear(json)?;
    api.results
        .into_iter()
        .map(|r| arribo(r, recibido))
        .collect()
}

fn arribo(r: ApiResultado, recibido: DateTime<Utc>) -> Result<Arribo, SourceError> {
    // En la cabecera de origen no hay llegada: el horario relevante es la salida.
    let horario = [r.arribo.llegada, r.arribo.salida]
        .into_iter()
        .flatten()
        .find(|h| h.programada.is_some())
        .ok_or_else(|| {
            SourceError::RespuestaInvalida(format!("servicio {} sin horario", r.servicio.numero))
        })?;
    let programada = horario.programada.expect("filtrado arriba");
    let momento = match r.arribo.segundos {
        Some(segundos) => recibido + Duration::seconds(segundos),
        None => horario.estimada.unwrap_or(programada),
    };
    let s = r.servicio;
    Ok(Arribo {
        servicio: Servicio {
            numero: s.numero,
            ramal: Ramal {
                id: RamalId(s.ramal.id),
                nombre: s.ramal.nombre,
            },
            destino: s
                .hasta
                .and_then(|h| h.estacion)
                .and_then(|e| e.nombre)
                .unwrap_or_default(),
            estado: s
                .desde
                .and_then(|d| d.estado)
                .map(|e| EstadoServicio {
                    id: e.id,
                    nombre: e.nombre,
                })
                .unwrap_or(EstadoServicio {
                    id: 0,
                    nombre: "Sin informar".into(),
                }),
            formacion: s.equipo.map(|e| e.nombre),
            cancelado: s.cancelacion.is_some_and(|c| !c.is_null()),
            oculto: s.oculto,
            noticia: texto(s.leyenda),
        },
        anden: r.arribo.anden.map(|a| a.nombre),
        anden_anterior: None,
        programada,
        estimada: horario.estimada,
        momento,
    })
}

#[derive(Deserialize)]
struct ApiInstalaciones {
    direccion: Option<String>,
    boleteria: Option<String>,
    ascensor: Option<String>,
    rampa: Option<String>,
    banos: Option<String>,
    wifi: Option<String>,
    terminal_sube: Option<String>,
    conexiones: Option<String>,
    colectivos: Option<String>,
}

pub fn instalaciones(json: &str) -> Result<Option<Instalaciones>, SourceError> {
    let api: Vec<ApiInstalaciones> = parsear(json)?;
    Ok(api.into_iter().next().map(|i| Instalaciones {
        direccion: texto(i.direccion),
        boleteria: texto(i.boleteria),
        ascensor: si(&i.ascensor),
        rampa: si(&i.rampa),
        banos: si(&i.banos),
        wifi: si(&i.wifi),
        terminal_sube: si(&i.terminal_sube),
        conexiones: texto(i.conexiones),
        colectivos: texto(i.colectivos),
    }))
}

fn si(valor: &Option<String>) -> bool {
    valor
        .as_deref()
        .is_some_and(|v| v.trim().eq_ignore_ascii_case("SI"))
}

fn texto(valor: Option<String>) -> Option<String> {
    valor.map(|v| v.trim().to_owned()).filter(|v| !v.is_empty())
}

#[derive(Deserialize)]
struct ApiGerencia {
    id: u32,
    #[serde(default)]
    alerta: serde_json::Value,
}

#[derive(Deserialize)]
struct ApiRamal {
    id: u32,
    id_gerencia: u32,
    #[serde(default)]
    alerta: serde_json::Value,
}

#[derive(Deserialize)]
struct ApiAlerta {
    id: u64,
    linea_id: Option<u32>,
    ramal_id: Option<u32>,
    estacion_id: Option<u32>,
    contenido: String,
    habilitado: Option<i64>,
    vigencia_desde: Option<String>,
    vigencia_hasta: Option<String>,
    criticidad_orden: Option<u32>,
}

/// Noticias de toda la red: las alertas de cada gerencia (`gerencias`) y de cada Ramal
/// (`ramales`, una respuesta por Línea). Una alerta de toda la Línea se expresa como sus
/// Ramales. Las alertas que no se entienden se omiten: no deben tapar a las demás.
pub fn noticias(gerencias: &str, ramales: &[String]) -> Result<Vec<Noticia>, SourceError> {
    let gerencias: Vec<ApiGerencia> = parsear(gerencias)?;
    let mut todos_los_ramales: Vec<ApiRamal> = vec![];
    for json in ramales {
        todos_los_ramales.extend(parsear::<Vec<ApiRamal>>(json)?);
    }
    let alertas = gerencias
        .iter()
        .flat_map(|g| alertas(&g.alerta).into_iter().map(|a| (g.id, a)))
        .chain(
            todos_los_ramales
                .iter()
                .flat_map(|r| alertas(&r.alerta).into_iter().map(|a| (r.id_gerencia, a))),
        );
    let mut noticias: Vec<Noticia> = vec![];
    for (gerencia, a) in alertas {
        if noticias.iter().any(|n| n.id == a.id) {
            continue;
        }
        let alcance = match (a.estacion_id, a.ramal_id) {
            (Some(estacion), _) => AlcanceNoticia::Estacion(EstacionId(estacion)),
            (None, Some(ramal)) => AlcanceNoticia::Ramal(RamalId(ramal)),
            (None, None) => {
                let linea = a.linea_id.unwrap_or(gerencia);
                AlcanceNoticia::Linea(
                    todos_los_ramales
                        .iter()
                        .filter(|r| r.id_gerencia == linea)
                        .map(|r| RamalId(r.id))
                        .collect(),
                )
            }
        };
        noticias.push(Noticia {
            id: a.id,
            alcance,
            texto: a.contenido.trim().to_owned(),
            criticidad: a.criticidad_orden,
            habilitada: a.habilitado != Some(0),
            desde: a.vigencia_desde.as_deref().and_then(hora_argentina),
            hasta: a.vigencia_hasta.as_deref().and_then(hora_argentina),
        });
    }
    Ok(noticias)
}

/// Una alerta sola, una lista o nada.
fn alertas(valor: &serde_json::Value) -> Vec<ApiAlerta> {
    let lista = match valor {
        serde_json::Value::Array(lista) => lista.clone(),
        serde_json::Value::Object(_) => vec![valor.clone()],
        _ => vec![],
    };
    lista
        .into_iter()
        .filter_map(|a| serde_json::from_value(a).ok())
        .collect()
}

/// "2026-10-13 00:01:00" en hora argentina (UTC-3).
fn hora_argentina(texto: &str) -> Option<DateTime<Utc>> {
    let argentina = FixedOffset::west_opt(3 * 3600).expect("offset válido");
    NaiveDateTime::parse_from_str(texto.trim(), "%Y-%m-%d %H:%M:%S")
        .ok()?
        .and_local_timezone(argentina)
        .single()
        .map(|h| h.with_timezone(&Utc))
}

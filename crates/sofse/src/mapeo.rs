//! Respuestas JSON de SOFSE → dominio.
//!
//! Los tipos `Api*` reflejan la forma de la API y no salen de este módulo. Son
//! permisivos a propósito: la API es no oficial y los campos opcionales cambian según
//! el estado del Servicio.

use chrono::{DateTime, Duration, Utc};
use opentrains_core::dominio::{
    Arribo, Estacion, EstacionId, EstadoServicio, Instalaciones, Linea, LineaId, Ramal, RamalId,
    Servicio,
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
}

pub fn estaciones(json: &str) -> Result<Vec<Estacion>, SourceError> {
    let api: Vec<ApiEstacion> = parsear(json)?;
    let mut estaciones = api
        .into_iter()
        .map(|e| {
            Ok(Estacion {
                id: EstacionId(numero(&e.id_estacion)?),
                orden: numero(&e.orden)?,
                latitud: decimal(&e.latitud)?,
                longitud: decimal(&e.longitud)?,
                nombre: e.nombre,
            })
        })
        .collect::<Result<Vec<_>, SourceError>>()?;
    estaciones.sort_by_key(|e| e.orden);
    Ok(estaciones)
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
    timestamp: i64,
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

pub fn arribos(json: &str) -> Result<Vec<Arribo>, SourceError> {
    let api: ApiArribos = parsear(json)?;
    let consultado = DateTime::from_timestamp(api.timestamp, 0)
        .ok_or_else(|| SourceError::RespuestaInvalida("timestamp inválido".into()))?;
    api.results
        .into_iter()
        .map(|r| arribo(r, consultado))
        .collect()
}

fn arribo(r: ApiResultado, consultado: DateTime<Utc>) -> Result<Arribo, SourceError> {
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
        Some(segundos) => consultado + Duration::seconds(segundos),
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
        },
        anden: r.arribo.anden.map(|a| a.nombre),
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

//! Tipos del dominio. Ver `CONTEXT.md` para el glosario.

use std::collections::BTreeMap;
use std::fmt;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LineaId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RamalId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct EstacionId(pub u32);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Linea {
    pub id: LineaId,
    pub nombre: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Ramal {
    pub id: RamalId,
    pub nombre: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Estacion {
    pub id: EstacionId,
    pub nombre: String,
    /// Posición de la Estación dentro del Ramal consultado.
    pub orden: u32,
    pub latitud: f64,
    pub longitud: f64,
}

impl Estacion {
    pub fn coordenadas(&self) -> Coordenadas {
        Coordenadas {
            latitud: self.latitud,
            longitud: self.longitud,
        }
    }
}

/// Un punto en el mapa, en grados decimales (WGS84).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Coordenadas {
    pub latitud: f64,
    pub longitud: f64,
}

impl Coordenadas {
    /// Distancia en línea recta (haversine): no es lo que se camina, es una referencia.
    pub fn distancia(&self, otra: &Coordenadas) -> Distancia {
        const RADIO_TIERRA_METROS: f64 = 6_371_000.0;
        let (lat1, lat2) = (self.latitud.to_radians(), otra.latitud.to_radians());
        let dlat = lat2 - lat1;
        let dlon = (otra.longitud - self.longitud).to_radians();
        let a = (dlat / 2.0).sin().powi(2) + lat1.cos() * lat2.cos() * (dlon / 2.0).sin().powi(2);
        Distancia {
            metros: 2.0 * RADIO_TIERRA_METROS * a.sqrt().asin(),
        }
    }
}

/// Distancia desde la Ubicación; se muestra como "a 850 m", "a 3,2 km" o "a 27 km".
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Distancia {
    pub metros: f64,
}

impl fmt::Display for Distancia {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let metros = (self.metros / 10.0).round() * 10.0;
        if metros < 1_000.0 {
            return write!(f, "a {metros:.0} m");
        }
        let decimas = (self.metros / 100.0).round() / 10.0;
        if decimas < 10.0 {
            write!(f, "a {}", format!("{decimas:.1} km").replace('.', ","))
        } else {
            write!(f, "a {:.0} km", self.metros / 1_000.0)
        }
    }
}

/// Estado del Servicio tal como lo informa el operador.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EstadoServicio {
    pub id: u32,
    pub nombre: String,
}

/// Situación de un Arribo para el pasajero, derivada del Estado del Servicio, la
/// demora y la cancelación.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Situacion {
    Cancelado,
    EnAnden,
    Demorado,
    EnCamino,
    Programado,
    /// Estado aún no catalogado.
    Desconocido,
}

impl EstadoServicio {
    pub const PROGRAMADO: u32 = 10;
    pub const EN_ANDEN: u32 = 25;
    pub const PARTIO: u32 = 35;
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Servicio {
    pub numero: u32,
    pub ramal: Ramal,
    pub destino: String,
    pub estado: EstadoServicio,
    pub formacion: Option<String>,
    pub cancelado: bool,
    pub oculto: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Arribo {
    pub servicio: Servicio,
    pub anden: Option<String>,
    pub programada: DateTime<Utc>,
    pub estimada: Option<DateTime<Utc>>,
    /// Momento en que el Servicio llega (o sale) de la Estación, según el operador.
    pub momento: DateTime<Utc>,
}

impl Arribo {
    /// Demora en minutos enteros; `None` si no hay horario estimado o es menor a un minuto.
    pub fn demora_minutos(&self) -> Option<i64> {
        let minutos = (self.estimada? - self.programada).num_minutes();
        (minutos >= 1).then_some(minutos)
    }

    /// Prioridad: cancelado > en andén > demorado > estado informado.
    pub fn situacion(&self) -> Situacion {
        if self.servicio.cancelado {
            return Situacion::Cancelado;
        }
        if self.servicio.estado.id == EstadoServicio::EN_ANDEN {
            return Situacion::EnAnden;
        }
        if self.demora_minutos().is_some() {
            return Situacion::Demorado;
        }
        match self.servicio.estado.id {
            EstadoServicio::PARTIO => Situacion::EnCamino,
            EstadoServicio::PROGRAMADO => Situacion::Programado,
            _ => Situacion::Desconocido,
        }
    }
}

/// Un Servicio que el usuario sigue en una Estación para recibir Avisos.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Seguimiento {
    pub estacion: EstacionId,
    pub estacion_nombre: String,
    pub numero: u32,
    pub destino: String,
    /// Cuántos minutos antes del Arribo llega el recordatorio.
    pub anticipacion_minutos: i64,
    #[serde(default)]
    pub recordatorio_enviado: bool,
    #[serde(default)]
    pub demora_avisada: Option<i64>,
    #[serde(default)]
    pub cancelacion_avisada: bool,
    /// Último Arribo visto de este Servicio en la Estación.
    #[serde(default)]
    pub ultimo_arribo: Option<Arribo>,
}

/// Algo que vale la pena avisarle al usuario sobre un Servicio que sigue.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "tipo", rename_all = "snake_case")]
pub enum Aviso {
    /// Faltan `minutos` para que el Servicio pase por la Estación.
    Recordatorio {
        minutos: i64,
    },
    Demora {
        minutos: i64,
    },
    Cancelado,
}

impl Seguimiento {
    pub const DEMORA_MINIMA: i64 = 3;
    pub const SALTO_DE_DEMORA: i64 = 3;

    pub fn nuevo(
        estacion: EstacionId,
        estacion_nombre: impl Into<String>,
        numero: u32,
        destino: impl Into<String>,
        anticipacion_minutos: i64,
    ) -> Self {
        Self {
            estacion,
            estacion_nombre: estacion_nombre.into(),
            numero,
            destino: destino.into(),
            anticipacion_minutos,
            recordatorio_enviado: false,
            demora_avisada: None,
            cancelacion_avisada: false,
            ultimo_arribo: None,
        }
    }

    pub fn es_de(&self, estacion: EstacionId, numero: u32) -> bool {
        self.estacion == estacion && self.numero == numero
    }

    /// Avisos que corresponden según el estado actual del Arribo; recuerda lo ya avisado.
    pub fn evaluar(&mut self, arribo: &Arribo, ahora: DateTime<Utc>) -> Vec<Aviso> {
        let mut avisos = vec![];
        // Un Servicio cancelado ya no va a pasar: no tiene sentido recordarlo ni su demora.
        if arribo.servicio.cancelado {
            if !self.cancelacion_avisada {
                self.cancelacion_avisada = true;
                avisos.push(Aviso::Cancelado);
            }
            return avisos;
        }
        if let Some(demora) = arribo.demora_minutos() {
            let crecio = self
                .demora_avisada
                .is_none_or(|avisada| demora >= avisada + Self::SALTO_DE_DEMORA);
            if demora >= Self::DEMORA_MINIMA && crecio {
                self.demora_avisada = Some(demora);
                avisos.push(Aviso::Demora { minutos: demora });
            }
        }
        let faltan = arribo.momento - ahora;
        if !self.recordatorio_enviado
            && faltan > chrono::Duration::zero()
            && faltan <= chrono::Duration::minutes(self.anticipacion_minutos)
        {
            self.recordatorio_enviado = true;
            avisos.push(Aviso::Recordatorio {
                minutos: faltan.num_minutes(),
            });
        }
        avisos
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Instalaciones {
    pub direccion: Option<String>,
    pub boleteria: Option<String>,
    pub ascensor: bool,
    pub rampa: bool,
    pub banos: bool,
    pub wifi: bool,
    pub terminal_sube: bool,
    pub conexiones: Option<String>,
    pub colectivos: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Favorito {
    pub estacion: EstacionId,
    pub estacion_nombre: String,
    pub ramal: Option<Ramal>,
    /// Ausentes en Favoritos guardados antes de que existieran las distancias.
    #[serde(default)]
    pub coordenadas: Option<Coordenadas>,
}

impl Favorito {
    pub fn mismo_que(&self, otro: &Favorito) -> bool {
        self.estacion == otro.estacion
            && self.ramal.as_ref().map(|r| r.id) == otro.ramal.as_ref().map(|r| r.id)
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Preferencias {
    pub favoritos: Vec<Favorito>,
    pub ultimo_abierto: Option<Favorito>,
    /// Última anticipación elegida al seguir un Servicio, por Estación.
    #[serde(default)]
    pub anticipaciones: BTreeMap<EstacionId, u32>,
}

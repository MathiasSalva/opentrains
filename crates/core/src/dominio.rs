//! Tipos del dominio. Ver `CONTEXT.md` para el glosario.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LineaId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RamalId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
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

/// Estado del Servicio tal como lo informa el operador.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EstadoServicio {
    pub id: u32,
    pub nombre: String,
}

/// Cómo debe presentarse un Arribo según su Estado y si fue cancelado.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Tono {
    Normal,
    Cancelado,
    /// Estado aún no catalogado.
    Neutro,
}

impl EstadoServicio {
    pub const PROGRAMADO: u32 = 10;
    pub const EN_ANDEN: u32 = 25;
    pub const PARTIO: u32 = 35;

    fn es_conocido(&self) -> bool {
        matches!(self.id, Self::PROGRAMADO | Self::EN_ANDEN | Self::PARTIO)
    }
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

    pub fn tono(&self) -> Tono {
        if self.servicio.cancelado {
            Tono::Cancelado
        } else if self.servicio.estado.es_conocido() {
            Tono::Normal
        } else {
            Tono::Neutro
        }
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
}

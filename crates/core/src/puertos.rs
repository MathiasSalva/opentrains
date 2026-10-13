//! Puertos: lo que `core` necesita del mundo exterior.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::dominio::{
    Arribo, Estacion, EstacionId, Instalaciones, Linea, LineaId, Preferencias, Ramal, RamalId,
};

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error, Serialize)]
#[serde(tag = "tipo", content = "detalle", rename_all = "snake_case")]
pub enum SourceError {
    #[error("no se pudo contactar la fuente de datos: {0}")]
    SinConexion(String),
    /// La fuente rechaza nuestras credenciales de forma persistente.
    #[error("la fuente de datos cambió; la app necesita actualizarse")]
    FuenteCambiada,
    #[error("respuesta inesperada de la fuente de datos: {0}")]
    RespuestaInvalida(String),
}

#[derive(Debug, thiserror::Error)]
#[error("error de almacenamiento: {0}")]
pub struct StorageError(pub String);

/// Catálogo de la red: Líneas, Ramales y Estaciones.
#[allow(async_fn_in_trait)]
pub trait CatalogoSource {
    async fn lineas(&self) -> Result<Vec<Linea>, SourceError>;
    async fn ramales(&self, linea: LineaId) -> Result<Vec<Ramal>, SourceError>;
    async fn estaciones(&self, ramal: RamalId) -> Result<Vec<Estacion>, SourceError>;
}

#[allow(async_fn_in_trait)]
pub trait ArribosSource {
    async fn arribos(
        &self,
        estacion: EstacionId,
        ramal: Option<RamalId>,
    ) -> Result<Vec<Arribo>, SourceError>;
}

#[allow(async_fn_in_trait)]
pub trait InstalacionesSource {
    async fn instalaciones(
        &self,
        estacion: EstacionId,
    ) -> Result<Option<Instalaciones>, SourceError>;
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "tipo", content = "id", rename_all = "snake_case")]
pub enum ClaveCatalogo {
    Lineas,
    Ramales(LineaId),
    Estaciones(RamalId),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "tipo", content = "datos", rename_all = "snake_case")]
pub enum DatosCatalogo {
    Lineas(Vec<Linea>),
    Ramales(Vec<Ramal>),
    Estaciones(Vec<Estacion>),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EntradaCatalogo {
    pub guardada: DateTime<Utc>,
    pub datos: DatosCatalogo,
}

pub trait CatalogoCache {
    fn leer(&self, clave: &ClaveCatalogo) -> Result<Option<EntradaCatalogo>, StorageError>;
    fn guardar(&self, clave: &ClaveCatalogo, entrada: &EntradaCatalogo)
        -> Result<(), StorageError>;
}

pub trait PreferenciasRepo {
    fn cargar(&self) -> Result<Preferencias, StorageError>;
    fn guardar(&self, preferencias: &Preferencias) -> Result<(), StorageError>;
}

pub trait Clock {
    fn ahora(&self) -> DateTime<Utc>;
}

pub struct SystemClock;

impl Clock for SystemClock {
    fn ahora(&self) -> DateTime<Utc> {
        Utc::now()
    }
}

// Los adaptadores se comparten entre casos de uso a través de `Arc`.

impl<T: CatalogoSource> CatalogoSource for std::sync::Arc<T> {
    async fn lineas(&self) -> Result<Vec<Linea>, SourceError> {
        (**self).lineas().await
    }
    async fn ramales(&self, linea: LineaId) -> Result<Vec<Ramal>, SourceError> {
        (**self).ramales(linea).await
    }
    async fn estaciones(&self, ramal: RamalId) -> Result<Vec<Estacion>, SourceError> {
        (**self).estaciones(ramal).await
    }
}

impl<T: ArribosSource> ArribosSource for std::sync::Arc<T> {
    async fn arribos(
        &self,
        estacion: EstacionId,
        ramal: Option<RamalId>,
    ) -> Result<Vec<Arribo>, SourceError> {
        (**self).arribos(estacion, ramal).await
    }
}

impl<T: CatalogoCache> CatalogoCache for std::sync::Arc<T> {
    fn leer(&self, clave: &ClaveCatalogo) -> Result<Option<EntradaCatalogo>, StorageError> {
        (**self).leer(clave)
    }
    fn guardar(
        &self,
        clave: &ClaveCatalogo,
        entrada: &EntradaCatalogo,
    ) -> Result<(), StorageError> {
        (**self).guardar(clave, entrada)
    }
}

impl<T: PreferenciasRepo> PreferenciasRepo for std::sync::Arc<T> {
    fn cargar(&self) -> Result<Preferencias, StorageError> {
        (**self).cargar()
    }
    fn guardar(&self, preferencias: &Preferencias) -> Result<(), StorageError> {
        (**self).guardar(preferencias)
    }
}

impl<T: InstalacionesSource> InstalacionesSource for std::sync::Arc<T> {
    async fn instalaciones(
        &self,
        estacion: EstacionId,
    ) -> Result<Option<Instalaciones>, SourceError> {
        (**self).instalaciones(estacion).await
    }
}

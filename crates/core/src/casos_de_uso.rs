//! Casos de uso: la API pública de `core` que consumen los adaptadores de entrada.

use std::future::Future;

use chrono::Duration;

use crate::dominio::{Arribo, Estacion, Favorito, Linea, LineaId, Preferencias, Ramal, RamalId};
use crate::puertos::{
    ArribosSource, CatalogoCache, CatalogoSource, ClaveCatalogo, Clock, DatosCatalogo,
    EntradaCatalogo, PreferenciasRepo, SourceError, StorageError,
};

/// Arribos de un Favorito, listos para mostrar.
pub struct ConsultaArribos<A> {
    source: A,
}

impl<A: ArribosSource> ConsultaArribos<A> {
    pub fn new(source: A) -> Self {
        Self { source }
    }

    /// Sin Servicios ocultos, ordenados por el momento en que llegan.
    pub async fn arribos(&self, favorito: &Favorito) -> Result<Vec<Arribo>, SourceError> {
        let ramal = favorito.ramal.as_ref().map(|r| r.id);
        let mut arribos: Vec<Arribo> = self
            .source
            .arribos(favorito.estacion, ramal)
            .await?
            .into_iter()
            .filter(|a| !a.servicio.oculto)
            .collect();
        arribos.sort_by_key(|a| a.momento);
        Ok(arribos)
    }
}

/// Catálogo de la red con caché en disco.
pub struct Catalogo<S, C, R> {
    source: S,
    cache: C,
    reloj: R,
}

impl<S: CatalogoSource, C: CatalogoCache, R: Clock> Catalogo<S, C, R> {
    pub const VIGENCIA_DIAS: i64 = 7;

    pub fn new(source: S, cache: C, reloj: R) -> Self {
        Self {
            source,
            cache,
            reloj,
        }
    }

    pub async fn lineas(&self) -> Result<Vec<Linea>, SourceError> {
        self.obtener(
            ClaveCatalogo::Lineas,
            || self.source.lineas(),
            DatosCatalogo::Lineas,
            |d| match d {
                DatosCatalogo::Lineas(v) => Some(v),
                _ => None,
            },
        )
        .await
    }

    pub async fn ramales(&self, linea: LineaId) -> Result<Vec<Ramal>, SourceError> {
        self.obtener(
            ClaveCatalogo::Ramales(linea),
            || self.source.ramales(linea),
            DatosCatalogo::Ramales,
            |d| match d {
                DatosCatalogo::Ramales(v) => Some(v),
                _ => None,
            },
        )
        .await
    }

    pub async fn estaciones(&self, ramal: RamalId) -> Result<Vec<Estacion>, SourceError> {
        self.obtener(
            ClaveCatalogo::Estaciones(ramal),
            || self.source.estaciones(ramal),
            DatosCatalogo::Estaciones,
            |d| match d {
                DatosCatalogo::Estaciones(v) => Some(v),
                _ => None,
            },
        )
        .await
    }

    /// Cache vigente → fuente → cache vencida, en ese orden. Los errores de la cache
    /// se tratan como ausencia: la cache es una optimización, no una fuente de verdad.
    async fn obtener<T: Clone, F: Future<Output = Result<Vec<T>, SourceError>>>(
        &self,
        clave: ClaveCatalogo,
        consultar: impl FnOnce() -> F,
        envolver: fn(Vec<T>) -> DatosCatalogo,
        extraer: fn(DatosCatalogo) -> Option<Vec<T>>,
    ) -> Result<Vec<T>, SourceError> {
        let ahora = self.reloj.ahora();
        let en_cache = self.cache.leer(&clave).ok().flatten();
        if let Some(entrada) = &en_cache {
            if ahora - entrada.guardada < Duration::days(Self::VIGENCIA_DIAS) {
                if let Some(datos) = extraer(entrada.datos.clone()) {
                    return Ok(datos);
                }
            }
        }
        match consultar().await {
            Ok(datos) => {
                let entrada = EntradaCatalogo {
                    guardada: ahora,
                    datos: envolver(datos.clone()),
                };
                let _ = self.cache.guardar(&clave, &entrada);
                Ok(datos)
            }
            Err(error) => en_cache.and_then(|e| extraer(e.datos)).ok_or(error),
        }
    }
}

/// Favoritos del usuario y último Favorito abierto.
pub struct Favoritos<P> {
    repo: P,
}

impl<P: PreferenciasRepo> Favoritos<P> {
    pub fn new(repo: P) -> Self {
        Self { repo }
    }

    pub fn preferencias(&self) -> Result<Preferencias, StorageError> {
        self.repo.cargar()
    }

    pub fn agregar(&self, favorito: Favorito) -> Result<Preferencias, StorageError> {
        self.modificar(|p| {
            if !p.favoritos.iter().any(|f| f.mismo_que(&favorito)) {
                p.favoritos.push(favorito);
            }
        })
    }

    pub fn quitar(&self, favorito: &Favorito) -> Result<Preferencias, StorageError> {
        self.modificar(|p| p.favoritos.retain(|f| !f.mismo_que(favorito)))
    }

    /// Recuerda la Estación (y Ramal) abierta, sea o no un Favorito guardado.
    pub fn abrir(&self, favorito: Favorito) -> Result<Preferencias, StorageError> {
        self.modificar(|p| p.ultimo_abierto = Some(favorito))
    }

    fn modificar(
        &self,
        cambio: impl FnOnce(&mut Preferencias),
    ) -> Result<Preferencias, StorageError> {
        let mut preferencias = self.repo.cargar()?;
        cambio(&mut preferencias);
        self.repo.guardar(&preferencias)?;
        Ok(preferencias)
    }
}

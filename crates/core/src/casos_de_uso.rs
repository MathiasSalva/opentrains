//! Casos de uso: la API pública de `core` que consumen los adaptadores de entrada.

use std::collections::BTreeSet;
use std::future::Future;
use std::sync::Mutex;

use chrono::Duration;

use serde::Serialize;

use crate::dominio::{
    Arribo, Coordenadas, Distancia, Estacion, EstacionId, Favorito, Linea, LineaId, Preferencias,
    Ramal, RamalId, Seguimiento,
};
use crate::puertos::{
    ArribosSource, CatalogoCache, CatalogoSource, ClaveCatalogo, Clock, DatosCatalogo,
    EntradaCatalogo, EstacionesCercanasSource, Notificador, PreferenciasRepo, SourceError,
    StorageError, UbicacionSource,
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

    /// Recuerda la Estación (y Ramal) abierta, sea o no un Favorito guardado. Si trae
    /// coordenadas, se las completa a los Favoritos de esa Estación que no las tenían.
    pub fn abrir(&self, favorito: Favorito) -> Result<Preferencias, StorageError> {
        self.modificar(|p| {
            if let Some(coordenadas) = favorito.coordenadas {
                p.favoritos
                    .iter_mut()
                    .filter(|f| f.estacion == favorito.estacion && f.coordenadas.is_none())
                    .for_each(|f| f.coordenadas = Some(coordenadas));
            }
            p.ultimo_abierto = Some(favorito);
        })
    }

    /// La anticipación elegida al seguir un Servicio pasa a ser la sugerida en esa Estación.
    pub fn recordar_anticipacion(
        &self,
        estacion: EstacionId,
        minutos: u32,
    ) -> Result<Preferencias, StorageError> {
        self.modificar(|p| {
            p.anticipaciones.insert(estacion, minutos);
        })
    }

    pub fn ocultar_aviso_ubicacion(&self) -> Result<Preferencias, StorageError> {
        self.modificar(|p| p.aviso_ubicacion_oculto = true)
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

/// Una de las Estaciones cercanas, con su distancia a la Ubicación.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct EstacionCercana {
    pub estacion: Estacion,
    pub distancia: Distancia,
}

/// Ubicación del usuario y distancias a las Estaciones. La Ubicación vive en memoria:
/// se conserva la última conocida y nunca se guarda en disco.
pub struct Cercania<U, E> {
    ubicacion: U,
    source: E,
    ultima: Mutex<Option<Coordenadas>>,
    /// Estaciones cercanas y desde dónde se pidieron.
    cercanas: Mutex<Option<(Coordenadas, Vec<Estacion>)>>,
}

impl<U: UbicacionSource, E: EstacionesCercanasSource> Cercania<U, E> {
    pub const CUANTAS: usize = 3;
    /// Movimiento mínimo para volver a pedir las Estaciones cercanas: menos es ruido
    /// de la ubicación por Wi-Fi.
    pub const MOVIMIENTO_METROS: f64 = 300.0;

    pub fn new(ubicacion: U, source: E) -> Self {
        Self {
            ubicacion,
            source,
            ultima: Mutex::default(),
            cercanas: Mutex::default(),
        }
    }

    /// Consulta la Ubicación; si no hay una nueva, queda la última conocida.
    pub async fn actualizar(&self) -> Option<Coordenadas> {
        let nueva = self.ubicacion.ubicacion().await;
        let mut ultima = self.ultima.lock().unwrap();
        if nueva.is_some() {
            *ultima = nueva;
        }
        *ultima
    }

    pub fn distancias(&self, destinos: &[Coordenadas]) -> Option<Vec<Distancia>> {
        let desde = (*self.ultima.lock().unwrap())?;
        Some(destinos.iter().map(|d| desde.distancia(d)).collect())
    }

    /// Las Estaciones más próximas a la Ubicación, de cualquier Línea.
    pub async fn cercanas(&self) -> Result<Option<Vec<EstacionCercana>>, SourceError> {
        let Some(desde) = *self.ultima.lock().unwrap() else {
            return Ok(None);
        };
        let vigentes = self
            .cercanas
            .lock()
            .unwrap()
            .clone()
            .filter(|(pedidas, _)| pedidas.distancia(&desde).metros < Self::MOVIMIENTO_METROS);
        let estaciones = match vigentes {
            Some((_, estaciones)) => estaciones,
            None => {
                let estaciones = self.source.estaciones_cercanas(desde).await?;
                *self.cercanas.lock().unwrap() = Some((desde, estaciones.clone()));
                estaciones
            }
        };
        let mut vistas = BTreeSet::new();
        let mut cercanas: Vec<EstacionCercana> = estaciones
            .into_iter()
            .filter(|e| vistas.insert(e.id))
            .map(|estacion| EstacionCercana {
                distancia: desde.distancia(&estacion.coordenadas()),
                estacion,
            })
            .collect();
        cercanas.sort_by(|a, b| a.distancia.metros.total_cmp(&b.distancia.metros));
        cercanas.truncate(Self::CUANTAS);
        Ok(Some(cercanas))
    }
}

/// Servicios que el usuario sigue y los Avisos que generan.
pub struct Seguimientos<A, N, C> {
    source: A,
    notificador: N,
    reloj: C,
    activos: Mutex<Vec<Seguimiento>>,
}

impl<A: ArribosSource, N: Notificador, C: Clock> Seguimientos<A, N, C> {
    pub fn new(source: A, notificador: N, reloj: C) -> Self {
        Self {
            source,
            notificador,
            reloj,
            activos: Mutex::default(),
        }
    }

    /// Seguir de nuevo el mismo Servicio en la misma Estación reemplaza el Seguimiento.
    pub fn seguir(&self, seguimiento: Seguimiento) {
        let mut activos = self.activos.lock().unwrap();
        activos.retain(|s| !s.es_de(seguimiento.estacion, seguimiento.numero));
        activos.push(seguimiento);
    }

    pub fn dejar_de_seguir(&self, estacion: EstacionId, numero: u32) {
        self.activos
            .lock()
            .unwrap()
            .retain(|s| !s.es_de(estacion, numero));
    }

    pub fn activos(&self) -> Vec<Seguimiento> {
        self.activos.lock().unwrap().clone()
    }

    /// Consulta los Arribos de cada Estación con Seguimientos, notifica los Avisos y
    /// termina los Seguimientos de Servicios que ya pasaron.
    /// Si falla la consulta de una Estación, sus Seguimientos siguen intactos y se
    /// devuelve el último error, después de revisar las demás.
    pub async fn revisar(&self) -> Result<(), SourceError> {
        let estaciones: BTreeSet<u32> = self.activos().iter().map(|s| s.estacion.0).collect();
        let mut ultimo_error = None;
        for estacion in estaciones.into_iter().map(EstacionId) {
            let arribos = match self.source.arribos(estacion, None).await {
                Ok(arribos) => arribos,
                Err(error) => {
                    ultimo_error = Some(error);
                    continue;
                }
            };
            let ahora = self.reloj.ahora();
            let mut a_notificar = vec![];
            self.activos.lock().unwrap().retain_mut(|seguimiento| {
                if seguimiento.estacion != estacion {
                    return true;
                }
                // Si el Servicio ya no figura en la Estación, pasó: el Seguimiento termina.
                let Some(arribo) = arribos
                    .iter()
                    .find(|a| a.servicio.numero == seguimiento.numero)
                else {
                    return false;
                };
                seguimiento.ultimo_arribo = Some(arribo.clone());
                for aviso in seguimiento.evaluar(arribo, ahora) {
                    a_notificar.push((seguimiento.clone(), aviso));
                }
                true
            });
            for (seguimiento, aviso) in &a_notificar {
                self.notificador.notificar(seguimiento, aviso);
            }
        }
        ultimo_error.map_or(Ok(()), Err)
    }
}

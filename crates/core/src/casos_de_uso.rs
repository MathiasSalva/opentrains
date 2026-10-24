//! Casos de uso: la API pública de `core` que consumen los adaptadores de entrada.

use std::collections::{BTreeSet, HashMap};
use std::future::Future;
use std::sync::Mutex;

use chrono::{DateTime, Duration, Utc};

use serde::Serialize;

use crate::dominio::{
    Arribo, Coordenadas, Distancia, Estacion, EstacionId, Favorito, Linea, LineaId, Noticia,
    Preferencias, Ramal, RamalId, Seguimiento, Tablero, VentanaTablero,
};
use crate::puertos::{
    ArribosSource, CatalogoCache, CatalogoSource, ClaveCatalogo, Clock, DatosCatalogo,
    EntradaCatalogo, EstacionesCercanasSource, NoticiasSource, Notificador, PreferenciasRepo,
    SourceError, StorageError, UbicacionSource,
};

/// Último andén conocido de un Servicio y, si hubo un Cambio de andén, el anterior.
type AndenVisto = (String, Option<String>);

/// Arribos de un Favorito, listos para mostrar.
pub struct ConsultaArribos<A> {
    source: A,
    /// Por Servicio en cada Estación; vive en memoria mientras el Arribo siga en la lista.
    andenes: Mutex<HashMap<(EstacionId, u32), AndenVisto>>,
}

impl<A: ArribosSource> ConsultaArribos<A> {
    pub fn new(source: A) -> Self {
        Self {
            source,
            andenes: Mutex::default(),
        }
    }

    /// Sin Servicios ocultos, ordenados por el momento en que llegan, y con el Cambio de
    /// andén marcado.
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
        self.marcar_cambios_de_anden(favorito.estacion, &mut arribos);
        Ok(arribos)
    }

    fn marcar_cambios_de_anden(&self, estacion: EstacionId, arribos: &mut [Arribo]) {
        let mut andenes = self.andenes.lock().unwrap();
        andenes.retain(|(e, numero), _| {
            *e != estacion || arribos.iter().any(|a| a.servicio.numero == *numero)
        });
        for arribo in arribos.iter_mut() {
            let clave = (estacion, arribo.servicio.numero);
            let (conocido, anterior) = andenes.get(&clave).cloned().unzip();
            arribo.marcar_cambio_de_anden(conocido.as_deref(), anterior.flatten());
            // Perder el andén no es un cambio: se recuerda el último conocido.
            if let Some(actual) = arribo.anden.clone() {
                andenes.insert(clave, (actual, arribo.anden_anterior.clone()));
            }
        }
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
    /// coordenadas o Ramales, se los completa a los Favoritos de esa Estación que no los
    /// tenían.
    pub fn abrir(&self, favorito: Favorito) -> Result<Preferencias, StorageError> {
        self.modificar(|p| {
            if !favorito.ramales.is_empty() {
                p.favoritos
                    .iter_mut()
                    .filter(|f| f.estacion == favorito.estacion && f.ramales.is_empty())
                    .for_each(|f| f.ramales = favorito.ramales.clone());
            }
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

    /// Abre el tablero en esa Estación (o lo cambia a ella); la ventana queda donde estaba.
    pub fn abrir_tablero(&self, favorito: Favorito) -> Result<Preferencias, StorageError> {
        self.modificar(|p| {
            let anterior = p.tablero.take();
            p.tablero = Some(Tablero {
                favorito,
                abierto: true,
                ventana: anterior.as_ref().and_then(|t| t.ventana.clone()),
                pantalla_completa: anterior.is_some_and(|t| t.pantalla_completa),
            });
        })
    }

    /// Lo cerró el usuario: no se reabre al iniciar, pero recuerda dónde estaba.
    pub fn cerrar_tablero(&self) -> Result<Preferencias, StorageError> {
        self.modificar(|p| {
            if let Some(tablero) = &mut p.tablero {
                tablero.abierto = false;
            }
        })
    }

    pub fn recordar_ventana_tablero(
        &self,
        ventana: Option<VentanaTablero>,
        pantalla_completa: bool,
    ) -> Result<Preferencias, StorageError> {
        self.modificar(|p| {
            if let Some(tablero) = &mut p.tablero {
                if ventana.is_some() {
                    tablero.ventana = ventana;
                }
                tablero.pantalla_completa = pantalla_completa;
            }
        })
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
                let mut arribo = arribo.clone();
                let anterior = seguimiento
                    .ultimo_arribo
                    .as_ref()
                    .and_then(|visto| visto.anden_anterior.clone());
                arribo.marcar_cambio_de_anden(seguimiento.anden_conocido.as_deref(), anterior);
                // Perder el andén no es un cambio: se recuerda el último conocido.
                if arribo.anden.is_some() {
                    seguimiento.anden_conocido = arribo.anden.clone();
                }
                seguimiento.ultimo_arribo = Some(arribo.clone());
                for aviso in seguimiento.evaluar(&arribo, ahora) {
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

/// Noticias del operador que corresponden a la Estación abierta. Se piden para toda la
/// red y se guardan en memoria: cambian cada horas, no cada refresco de Arribos.
pub struct Noticias<S, P, C> {
    source: S,
    repo: P,
    reloj: C,
    /// Cuándo se intentó pedirlas por última vez y las últimas que llegaron.
    ultimas: Mutex<Option<(DateTime<Utc>, Vec<Noticia>)>>,
    /// Las ya avisadas, por si falla el disco: así no se repiten en cada vuelta.
    avisadas: Mutex<BTreeSet<u64>>,
}

impl<S: NoticiasSource, P: PreferenciasRepo, C: Clock> Noticias<S, P, C> {
    pub fn new(source: S, repo: P, reloj: C) -> Self {
        Self {
            source,
            repo,
            reloj,
            ultimas: Mutex::default(),
            avisadas: Mutex::default(),
        }
    }

    pub const REFRESCO_MINUTOS: i64 = 5;

    /// Vigentes y de esa Estación, sus Ramales o su Línea, de la más grave a la menos.
    /// Si la fuente falla quedan las últimas que llegaron; si nunca llegó ninguna, no hay.
    pub async fn de_estacion(&self, estacion: EstacionId, ramales: &[RamalId]) -> Vec<Noticia> {
        let ahora = self.reloj.ahora();
        let todas = self.todas(ahora).await;
        let descartadas = self
            .repo
            .cargar()
            .map(|p| p.noticias_descartadas)
            .unwrap_or_default();
        let mut vistas: Vec<Noticia> = todas
            .into_iter()
            .filter(|n| n.aplica_a(estacion, ramales) && n.vigente(ahora))
            .filter(|n| n.destacada() || !descartadas.contains(&n.id))
            .collect();
        vistas.sort_by_key(|n| n.criticidad.unwrap_or(u32::MAX));
        vistas
    }

    /// Avisa una vez cada Noticia destacada y vigente que corresponda a un Favorito o a
    /// un Servicio seguido (su Estación o su Ramal); el lugar es el primero que coincida.
    pub async fn avisar(&self, seguimientos: &[Seguimiento], notificador: &impl Notificador) {
        let ahora = self.reloj.ahora();
        let todas = self.todas(ahora).await;
        // Se carga después de consultar la fuente, para no pisar lo que cambió mientras.
        let Ok(mut preferencias) = self.repo.cargar() else {
            return;
        };
        let lugares: Vec<(&str, EstacionId, Vec<RamalId>)> = preferencias
            .favoritos
            .iter()
            .map(|f| {
                (
                    f.estacion_nombre.as_str(),
                    f.estacion,
                    f.ramales_de_noticias(),
                )
            })
            .chain(seguimientos.iter().map(|s| {
                let ramal = s.ultimo_arribo.as_ref().map(|a| a.servicio.ramal.id);
                (
                    s.estacion_nombre.as_str(),
                    s.estacion,
                    ramal.into_iter().collect(),
                )
            }))
            .collect();
        let mut avisadas = self.avisadas.lock().unwrap();
        let mut nuevas = vec![];
        for noticia in todas.iter().filter(|n| n.destacada() && n.vigente(ahora)) {
            if avisadas.contains(&noticia.id)
                || preferencias.noticias_avisadas.contains(&noticia.id)
            {
                continue;
            }
            let lugar = lugares
                .iter()
                .find(|(_, estacion, ramales)| noticia.aplica_a(*estacion, ramales));
            if let Some((lugar, _, _)) = lugar {
                notificador.notificar_noticia(lugar, noticia);
                avisadas.insert(noticia.id);
                nuevas.push(noticia.id);
            }
        }
        drop(avisadas);
        if !nuevas.is_empty() {
            preferencias.noticias_avisadas.extend(nuevas);
            let _ = self.repo.guardar(&preferencias);
        }
    }

    async fn todas(&self, ahora: DateTime<Utc>) -> Vec<Noticia> {
        let guardadas = self.ultimas.lock().unwrap().clone();
        if let Some((consultada, noticias)) = &guardadas {
            if ahora - *consultada < Duration::minutes(Self::REFRESCO_MINUTOS) {
                return noticias.clone();
            }
        }
        let noticias = match self.source.noticias().await {
            Ok(noticias) => {
                self.olvidar_ausentes(&noticias);
                noticias
            }
            Err(_) => guardadas.map(|(_, n)| n).unwrap_or_default(),
        };
        *self.ultimas.lock().unwrap() = Some((ahora, noticias.clone()));
        noticias
    }

    /// Las Noticias destacadas se siguen mostrando aunque se hayan descartado.
    pub fn descartar(&self, id: u64) -> Result<Preferencias, StorageError> {
        let mut preferencias = self.repo.cargar()?;
        preferencias.noticias_descartadas.insert(id);
        self.repo.guardar(&preferencias)?;
        Ok(preferencias)
    }

    /// Así las listas de descartadas y avisadas no crecen para siempre.
    fn olvidar_ausentes(&self, publicadas: &[Noticia]) {
        let Ok(mut preferencias) = self.repo.cargar() else {
            return;
        };
        let publicada = |id: &u64| publicadas.iter().any(|n| n.id == *id);
        let antes = preferencias.clone();
        preferencias.noticias_descartadas.retain(publicada);
        preferencias.noticias_avisadas.retain(publicada);
        if preferencias != antes {
            let _ = self.repo.guardar(&preferencias);
        }
    }
}

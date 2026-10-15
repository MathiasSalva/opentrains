use chrono::{DateTime, Utc};
use opentrains_core::casos_de_uso::Catalogo;
use opentrains_core::dominio::{Estacion, EstacionId, Linea, LineaId, Ramal, RamalId};
use opentrains_core::puertos::{
    CatalogoCache, CatalogoSource, ClaveCatalogo, Clock, DatosCatalogo, EntradaCatalogo,
    SourceError, StorageError,
};
use std::cell::{Cell, RefCell};
use std::collections::HashMap;

fn t(s: &str) -> DateTime<Utc> {
    s.parse().unwrap()
}

fn linea(id: u32, nombre: &str) -> Linea {
    Linea {
        id: LineaId(id),
        nombre: nombre.into(),
    }
}

#[derive(Default)]
struct FakeSource {
    lineas: RefCell<Option<Result<Vec<Linea>, SourceError>>>,
    llamadas: Cell<u32>,
}

impl CatalogoSource for &FakeSource {
    async fn lineas(&self) -> Result<Vec<Linea>, SourceError> {
        self.llamadas.set(self.llamadas.get() + 1);
        self.lineas
            .borrow()
            .clone()
            .expect("la fuente no debería consultarse")
    }
    async fn ramales(&self, _: LineaId) -> Result<Vec<Ramal>, SourceError> {
        self.llamadas.set(self.llamadas.get() + 1);
        Ok(vec![Ramal {
            id: RamalId(131),
            nombre: "Retiro-Junín".into(),
        }])
    }
    async fn estaciones(&self, _: RamalId) -> Result<Vec<Estacion>, SourceError> {
        self.llamadas.set(self.llamadas.get() + 1);
        Ok(vec![Estacion {
            id: EstacionId(463),
            nombre: "Retiro (LSM)".into(),
            orden: 1,
            latitud: -34.58,
            longitud: -58.37,
            ramales: vec![RamalId(131)],
        }])
    }
}

#[derive(Default)]
struct FakeCache(RefCell<HashMap<ClaveCatalogo, EntradaCatalogo>>);

impl CatalogoCache for &FakeCache {
    fn leer(&self, clave: &ClaveCatalogo) -> Result<Option<EntradaCatalogo>, StorageError> {
        Ok(self.0.borrow().get(clave).cloned())
    }
    fn guardar(
        &self,
        clave: &ClaveCatalogo,
        entrada: &EntradaCatalogo,
    ) -> Result<(), StorageError> {
        self.0.borrow_mut().insert(clave.clone(), entrada.clone());
        Ok(())
    }
}

struct RelojFijo(DateTime<Utc>);

impl Clock for RelojFijo {
    fn ahora(&self) -> DateTime<Utc> {
        self.0
    }
}

fn cache_con_lineas(guardada: &str, lineas: Vec<Linea>) -> FakeCache {
    let cache = FakeCache::default();
    cache.0.borrow_mut().insert(
        ClaveCatalogo::Lineas,
        EntradaCatalogo {
            guardada: t(guardada),
            datos: DatosCatalogo::Lineas(lineas),
        },
    );
    cache
}

#[tokio::test]
async fn con_cache_vigente_no_se_consulta_la_fuente() {
    let source = FakeSource::default();
    let cache = cache_con_lineas("2026-10-10T00:00:00Z", vec![linea(31, "San Martín")]);
    let catalogo = Catalogo::new(&source, &cache, RelojFijo(t("2026-10-13T00:00:00Z")));

    let lineas = catalogo.lineas().await.unwrap();

    assert_eq!(lineas, vec![linea(31, "San Martín")]);
    assert_eq!(source.llamadas.get(), 0);
}

#[tokio::test]
async fn con_cache_vencida_se_consulta_la_fuente_y_se_actualiza_la_cache() {
    let source = FakeSource::default();
    *source.lineas.borrow_mut() = Some(Ok(vec![linea(1, "Sarmiento"), linea(31, "San Martín")]));
    let cache = cache_con_lineas("2026-10-01T00:00:00Z", vec![linea(31, "San Martín")]);
    let ahora = t("2026-10-13T00:00:00Z");
    let catalogo = Catalogo::new(&source, &cache, RelojFijo(ahora));

    let lineas = catalogo.lineas().await.unwrap();

    assert_eq!(lineas, vec![linea(1, "Sarmiento"), linea(31, "San Martín")]);
    assert_eq!(
        cache.0.borrow().get(&ClaveCatalogo::Lineas),
        Some(&EntradaCatalogo {
            guardada: ahora,
            datos: DatosCatalogo::Lineas(lineas)
        })
    );
}

#[tokio::test]
async fn si_la_fuente_falla_se_usa_la_cache_vencida() {
    let source = FakeSource::default();
    *source.lineas.borrow_mut() = Some(Err(SourceError::SinConexion("timeout".into())));
    let cache = cache_con_lineas("2026-09-01T00:00:00Z", vec![linea(31, "San Martín")]);
    let catalogo = Catalogo::new(&source, &cache, RelojFijo(t("2026-10-13T00:00:00Z")));

    assert_eq!(catalogo.lineas().await, Ok(vec![linea(31, "San Martín")]));
}

#[tokio::test]
async fn si_la_fuente_falla_y_no_hay_cache_se_propaga_el_error() {
    let source = FakeSource::default();
    *source.lineas.borrow_mut() = Some(Err(SourceError::SinConexion("timeout".into())));
    let cache = FakeCache::default();
    let catalogo = Catalogo::new(&source, &cache, RelojFijo(t("2026-10-13T00:00:00Z")));

    assert_eq!(
        catalogo.lineas().await,
        Err(SourceError::SinConexion("timeout".into()))
    );
}

#[tokio::test]
async fn ramales_y_estaciones_se_cachean_por_su_clave() {
    let source = FakeSource::default();
    let cache = FakeCache::default();
    let catalogo = Catalogo::new(&source, &cache, RelojFijo(t("2026-10-13T00:00:00Z")));

    let ramales = catalogo.ramales(LineaId(31)).await.unwrap();
    let estaciones = catalogo.estaciones(RamalId(131)).await.unwrap();
    catalogo.ramales(LineaId(31)).await.unwrap();
    catalogo.estaciones(RamalId(131)).await.unwrap();

    assert_eq!(ramales[0].id, RamalId(131));
    assert_eq!(estaciones[0].id, EstacionId(463));
    assert_eq!(source.llamadas.get(), 2);
}

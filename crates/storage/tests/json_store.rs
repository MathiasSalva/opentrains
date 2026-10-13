use chrono::{DateTime, Utc};
use opentrains_core::dominio::{
    EstacionId, Favorito, Linea, LineaId, Preferencias, Ramal, RamalId,
};
use opentrains_core::puertos::{
    CatalogoCache, ClaveCatalogo, DatosCatalogo, EntradaCatalogo, PreferenciasRepo,
};
use opentrains_storage::JsonStore;

fn t(s: &str) -> DateTime<Utc> {
    s.parse().unwrap()
}

fn preferencias() -> Preferencias {
    let retiro = Favorito {
        estacion: EstacionId(463),
        estacion_nombre: "Retiro (LSM)".into(),
        ramal: Some(Ramal {
            id: RamalId(131),
            nombre: "Retiro - Junín".into(),
        }),
    };
    Preferencias {
        favoritos: vec![retiro.clone()],
        ultimo_abierto: Some(retiro),
    }
}

#[test]
fn sin_archivo_las_preferencias_estan_vacias() {
    let dir = tempfile::tempdir().unwrap();

    assert_eq!(
        JsonStore::new(dir.path()).cargar().unwrap(),
        Preferencias::default()
    );
}

#[test]
fn las_preferencias_sobreviven_a_un_reinicio() {
    let dir = tempfile::tempdir().unwrap();
    let directorio = dir.path().join("OpenTrains");

    PreferenciasRepo::guardar(&JsonStore::new(&directorio), &preferencias()).unwrap();

    assert_eq!(
        JsonStore::new(&directorio).cargar().unwrap(),
        preferencias()
    );
}

#[test]
fn unas_preferencias_corruptas_son_un_error_y_no_se_pisan_en_silencio() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("preferencias.json"), "{ no es json").unwrap();

    assert!(JsonStore::new(dir.path()).cargar().is_err());
}

#[test]
fn la_cache_del_catalogo_se_guarda_por_clave() {
    let dir = tempfile::tempdir().unwrap();
    let store = JsonStore::new(dir.path());
    let lineas = EntradaCatalogo {
        guardada: t("2026-10-13T00:00:00Z"),
        datos: DatosCatalogo::Lineas(vec![Linea {
            id: LineaId(31),
            nombre: "San Martín".into(),
        }]),
    };
    let ramales = EntradaCatalogo {
        guardada: t("2026-10-12T00:00:00Z"),
        datos: DatosCatalogo::Ramales(vec![Ramal {
            id: RamalId(65),
            nombre: "Retiro-Cabred".into(),
        }]),
    };

    CatalogoCache::guardar(&store, &ClaveCatalogo::Lineas, &lineas).unwrap();
    CatalogoCache::guardar(&store, &ClaveCatalogo::Ramales(LineaId(31)), &ramales).unwrap();

    let reabierto = JsonStore::new(dir.path());
    assert_eq!(
        reabierto.leer(&ClaveCatalogo::Lineas).unwrap(),
        Some(lineas)
    );
    assert_eq!(
        reabierto
            .leer(&ClaveCatalogo::Ramales(LineaId(31)))
            .unwrap(),
        Some(ramales)
    );
    assert_eq!(
        reabierto.leer(&ClaveCatalogo::Ramales(LineaId(5))).unwrap(),
        None
    );
}

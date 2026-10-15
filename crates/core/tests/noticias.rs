use chrono::{DateTime, Utc};
use opentrains_core::casos_de_uso::Noticias;
use opentrains_core::dominio::{AlcanceNoticia, EstacionId, Noticia, Preferencias, RamalId};
use opentrains_core::puertos::{
    Clock, NoticiasSource, PreferenciasRepo, SourceError, StorageError,
};
use std::cell::{Cell, RefCell};

fn t(s: &str) -> DateTime<Utc> {
    s.parse().unwrap()
}

fn noticia(id: u64, alcance: AlcanceNoticia) -> Noticia {
    Noticia {
        id,
        alcance,
        texto: format!("Noticia {id}"),
        criticidad: Some(4),
        habilitada: true,
        desde: None,
        hasta: None,
    }
}

const RETIRO: EstacionId = EstacionId(463);
const JUNIN: RamalId = RamalId(131);
const CABRED: RamalId = RamalId(65);

struct FakeNoticias {
    respuesta: RefCell<Result<Vec<Noticia>, SourceError>>,
    llamadas: Cell<u32>,
}

impl FakeNoticias {
    fn con(noticias: Vec<Noticia>) -> Self {
        Self {
            respuesta: RefCell::new(Ok(noticias)),
            llamadas: Cell::new(0),
        }
    }
}

impl NoticiasSource for &FakeNoticias {
    async fn noticias(&self) -> Result<Vec<Noticia>, SourceError> {
        self.llamadas.set(self.llamadas.get() + 1);
        self.respuesta.borrow().clone()
    }
}

#[derive(Default)]
struct FakeRepo(RefCell<Preferencias>);

impl PreferenciasRepo for &FakeRepo {
    fn cargar(&self) -> Result<Preferencias, StorageError> {
        Ok(self.0.borrow().clone())
    }
    fn guardar(&self, preferencias: &Preferencias) -> Result<(), StorageError> {
        *self.0.borrow_mut() = preferencias.clone();
        Ok(())
    }
}

struct Reloj(Cell<DateTime<Utc>>);

impl Clock for &Reloj {
    fn ahora(&self) -> DateTime<Utc> {
        self.0.get()
    }
}

fn reloj() -> Reloj {
    Reloj(Cell::new(t("2026-10-15T12:00:00Z")))
}

fn ids(noticias: &[Noticia]) -> Vec<u64> {
    noticias.iter().map(|n| n.id).collect()
}

#[tokio::test]
async fn solo_se_ven_las_noticias_de_la_estacion_sus_ramales_o_su_linea() {
    let fuente = FakeNoticias::con(vec![
        noticia(1, AlcanceNoticia::Linea(vec![JUNIN, CABRED, RamalId(75)])),
        noticia(2, AlcanceNoticia::Linea(vec![RamalId(1), RamalId(111)])),
        noticia(3, AlcanceNoticia::Ramal(CABRED)),
        noticia(4, AlcanceNoticia::Ramal(RamalId(75))),
        noticia(5, AlcanceNoticia::Estacion(RETIRO)),
        noticia(6, AlcanceNoticia::Estacion(EstacionId(297))),
    ]);
    let (repo, reloj) = (FakeRepo::default(), reloj());
    let noticias = Noticias::new(&fuente, &repo, &reloj);

    let vistas = noticias.de_estacion(RETIRO, &[JUNIN, CABRED]).await;

    assert_eq!(ids(&vistas), vec![1, 3, 5]);
}

#[tokio::test]
async fn las_noticias_deshabilitadas_futuras_o_vencidas_no_se_ven() {
    let en_retiro = || noticia(0, AlcanceNoticia::Estacion(RETIRO));
    let fuente = FakeNoticias::con(vec![
        Noticia {
            id: 1,
            ..en_retiro()
        },
        Noticia {
            id: 2,
            habilitada: false,
            ..en_retiro()
        },
        Noticia {
            id: 3,
            desde: Some(t("2026-10-15T13:00:00Z")),
            ..en_retiro()
        },
        Noticia {
            id: 4,
            hasta: Some(t("2026-10-15T11:00:00Z")),
            ..en_retiro()
        },
        Noticia {
            id: 5,
            desde: Some(t("2026-10-15T11:00:00Z")),
            hasta: Some(t("2026-10-15T13:00:00Z")),
            ..en_retiro()
        },
    ]);
    let (repo, reloj) = (FakeRepo::default(), reloj());
    let noticias = Noticias::new(&fuente, &repo, &reloj);

    let vistas = noticias.de_estacion(RETIRO, &[]).await;

    assert_eq!(ids(&vistas), vec![1, 5]);
}

#[tokio::test]
async fn las_mas_graves_van_primero() {
    let en_retiro = |id, criticidad| Noticia {
        criticidad,
        ..noticia(id, AlcanceNoticia::Estacion(RETIRO))
    };
    let fuente = FakeNoticias::con(vec![
        en_retiro(1, Some(4)),
        en_retiro(2, None),
        en_retiro(3, Some(1)),
        en_retiro(4, Some(2)),
    ]);
    let (repo, reloj) = (FakeRepo::default(), reloj());
    let noticias = Noticias::new(&fuente, &repo, &reloj);

    let vistas = noticias.de_estacion(RETIRO, &[]).await;

    assert_eq!(ids(&vistas), vec![3, 4, 1, 2]);
    assert_eq!(
        vistas.iter().map(Noticia::destacada).collect::<Vec<_>>(),
        vec![true, true, false, false]
    );
}

#[tokio::test]
async fn la_fuente_se_consulta_a_lo_sumo_cada_cinco_minutos() {
    let fuente = FakeNoticias::con(vec![noticia(1, AlcanceNoticia::Estacion(RETIRO))]);
    let (repo, reloj) = (FakeRepo::default(), reloj());
    let noticias = Noticias::new(&fuente, &repo, &reloj);

    noticias.de_estacion(RETIRO, &[]).await;
    reloj.0.set(t("2026-10-15T12:04:59Z"));
    let vistas = noticias.de_estacion(EstacionId(297), &[]).await;
    assert_eq!(fuente.llamadas.get(), 1);
    assert!(
        vistas.is_empty(),
        "el filtro se aplica también a lo guardado"
    );

    reloj.0.set(t("2026-10-15T12:05:00Z"));
    noticias.de_estacion(RETIRO, &[]).await;
    assert_eq!(fuente.llamadas.get(), 2);
}

#[tokio::test]
async fn si_la_fuente_falla_quedan_las_ultimas_noticias() {
    let fuente = FakeNoticias::con(vec![noticia(1, AlcanceNoticia::Estacion(RETIRO))]);
    let (repo, reloj) = (FakeRepo::default(), reloj());
    let noticias = Noticias::new(&fuente, &repo, &reloj);
    noticias.de_estacion(RETIRO, &[]).await;

    *fuente.respuesta.borrow_mut() = Err(SourceError::SinConexion("sin red".into()));
    reloj.0.set(t("2026-10-15T12:10:00Z"));

    assert_eq!(ids(&noticias.de_estacion(RETIRO, &[]).await), vec![1]);
}

#[tokio::test]
async fn si_la_fuente_nunca_respondio_no_hay_noticias_ni_error() {
    let fuente = FakeNoticias::con(vec![]);
    *fuente.respuesta.borrow_mut() = Err(SourceError::SinConexion("sin red".into()));
    let (repo, reloj) = (FakeRepo::default(), reloj());
    let noticias = Noticias::new(&fuente, &repo, &reloj);

    assert!(noticias.de_estacion(RETIRO, &[]).await.is_empty());
}

#[tokio::test]
async fn una_noticia_descartada_no_se_vuelve_a_ver_ni_despues_de_reiniciar() {
    let fuente = FakeNoticias::con(vec![
        noticia(1, AlcanceNoticia::Estacion(RETIRO)),
        noticia(2, AlcanceNoticia::Estacion(RETIRO)),
    ]);
    let (repo, reloj) = (FakeRepo::default(), reloj());
    let noticias = Noticias::new(&fuente, &repo, &reloj);
    noticias.de_estacion(RETIRO, &[]).await;

    noticias.descartar(1).unwrap();

    assert_eq!(ids(&noticias.de_estacion(RETIRO, &[]).await), vec![2]);
    let reiniciada = Noticias::new(&fuente, &repo, &reloj);
    assert_eq!(ids(&reiniciada.de_estacion(RETIRO, &[]).await), vec![2]);
}

#[tokio::test]
async fn las_noticias_destacadas_no_se_pueden_descartar() {
    let grave = Noticia {
        criticidad: Some(1),
        ..noticia(1, AlcanceNoticia::Estacion(RETIRO))
    };
    let fuente = FakeNoticias::con(vec![grave]);
    let (repo, reloj) = (FakeRepo::default(), reloj());
    let noticias = Noticias::new(&fuente, &repo, &reloj);

    noticias.descartar(1).unwrap();

    assert_eq!(ids(&noticias.de_estacion(RETIRO, &[]).await), vec![1]);
}

#[tokio::test]
async fn se_olvidan_las_descartadas_que_el_operador_ya_no_publica() {
    let fuente = FakeNoticias::con(vec![noticia(1, AlcanceNoticia::Estacion(RETIRO))]);
    let repo = FakeRepo::default();
    repo.0.borrow_mut().noticias_descartadas = [1, 2].into();
    let reloj = reloj();
    let noticias = Noticias::new(&fuente, &repo, &reloj);

    noticias.de_estacion(RETIRO, &[]).await;

    assert_eq!(repo.0.borrow().noticias_descartadas, [1].into());
}

#[tokio::test]
async fn si_la_fuente_falla_no_se_olvida_ninguna_descartada() {
    let fuente = FakeNoticias::con(vec![]);
    *fuente.respuesta.borrow_mut() = Err(SourceError::SinConexion("sin red".into()));
    let repo = FakeRepo::default();
    repo.0.borrow_mut().noticias_descartadas = [1, 2].into();
    let reloj = reloj();
    let noticias = Noticias::new(&fuente, &repo, &reloj);

    noticias.de_estacion(RETIRO, &[]).await;

    assert_eq!(repo.0.borrow().noticias_descartadas, [1, 2].into());
}

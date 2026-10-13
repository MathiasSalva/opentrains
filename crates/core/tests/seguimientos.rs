use chrono::{DateTime, Duration, Utc};
use opentrains_core::casos_de_uso::Seguimientos;
use opentrains_core::dominio::{
    Arribo, Aviso, EstacionId, EstadoServicio, Ramal, RamalId, Seguimiento, Servicio,
};
use opentrains_core::puertos::{ArribosSource, Clock, Notificador, SourceError};
use std::cell::{Cell, RefCell};
use std::collections::HashMap;

fn t(s: &str) -> DateTime<Utc> {
    s.parse().unwrap()
}

const RETIRO: EstacionId = EstacionId(463);
const PALERMO: EstacionId = EstacionId(297);

fn arribo(numero: u32, momento: DateTime<Utc>) -> Arribo {
    Arribo {
        servicio: Servicio {
            numero,
            ramal: Ramal {
                id: RamalId(65),
                nombre: "Retiro-Cabred".into(),
            },
            destino: "Pilar".into(),
            estado: EstadoServicio {
                id: 10,
                nombre: "Programado".into(),
            },
            formacion: None,
            cancelado: false,
            oculto: false,
        },
        anden: None,
        programada: momento,
        estimada: None,
        momento,
    }
}

fn demorado(numero: u32, programada: DateTime<Utc>, minutos: i64) -> Arribo {
    let estimada = programada + Duration::minutes(minutos);
    Arribo {
        estimada: Some(estimada),
        momento: estimada,
        ..arribo(numero, programada)
    }
}

/// Fuente con los Arribos de cada Estación; cuenta las consultas.
#[derive(Default)]
struct FakeArribos {
    por_estacion: RefCell<HashMap<EstacionId, Vec<Arribo>>>,
    consultas: RefCell<Vec<EstacionId>>,
}

impl FakeArribos {
    fn poner(&self, estacion: EstacionId, arribos: Vec<Arribo>) {
        self.por_estacion.borrow_mut().insert(estacion, arribos);
    }
}

impl ArribosSource for &FakeArribos {
    async fn arribos(
        &self,
        estacion: EstacionId,
        _ramal: Option<RamalId>,
    ) -> Result<Vec<Arribo>, SourceError> {
        self.consultas.borrow_mut().push(estacion);
        Ok(self
            .por_estacion
            .borrow()
            .get(&estacion)
            .cloned()
            .unwrap_or_default())
    }
}

#[derive(Default)]
struct FakeNotificador(RefCell<Vec<(u32, Aviso)>>);

impl Notificador for &FakeNotificador {
    fn notificar(&self, seguimiento: &Seguimiento, aviso: &Aviso) {
        self.0
            .borrow_mut()
            .push((seguimiento.numero, aviso.clone()));
    }
}

struct RelojManual(Cell<DateTime<Utc>>);

impl Clock for &RelojManual {
    fn ahora(&self) -> DateTime<Utc> {
        self.0.get()
    }
}

struct Escenario {
    fuente: FakeArribos,
    avisos: FakeNotificador,
    reloj: RelojManual,
}

impl Escenario {
    fn new(ahora: &str) -> Self {
        Self {
            fuente: FakeArribos::default(),
            avisos: FakeNotificador::default(),
            reloj: RelojManual(Cell::new(t(ahora))),
        }
    }

    fn seguimientos(&self) -> Seguimientos<&FakeArribos, &FakeNotificador, &RelojManual> {
        Seguimientos::new(&self.fuente, &self.avisos, &self.reloj)
    }

    fn avisos(&self) -> Vec<(u32, Aviso)> {
        self.avisos.0.borrow_mut().drain(..).collect()
    }
}

fn seguir_3907(anticipacion: i64) -> Seguimiento {
    Seguimiento::nuevo(RETIRO, "Retiro (LSM)", 3907, "Pilar", anticipacion)
}

#[tokio::test]
async fn el_recordatorio_llega_una_vez_al_entrar_en_la_anticipacion_elegida() {
    let e = Escenario::new("2026-10-13T10:00:00Z");
    let s = e.seguimientos();
    s.seguir(seguir_3907(10));
    let sale = t("2026-10-13T10:15:00Z");

    e.fuente.poner(RETIRO, vec![arribo(3907, sale)]);
    s.revisar().await.unwrap();
    assert_eq!(e.avisos(), vec![], "a 15 min todavía no");

    e.reloj.0.set(t("2026-10-13T10:05:30Z"));
    s.revisar().await.unwrap();
    assert_eq!(e.avisos(), vec![(3907, Aviso::Recordatorio { minutos: 9 })]);

    e.reloj.0.set(t("2026-10-13T10:06:30Z"));
    s.revisar().await.unwrap();
    assert_eq!(e.avisos(), vec![], "el recordatorio no se repite");
}

#[tokio::test]
async fn la_demora_se_avisa_desde_3_minutos_y_cada_vez_que_crece_3_mas() {
    let e = Escenario::new("2026-10-13T09:00:00Z");
    let s = e.seguimientos();
    s.seguir(seguir_3907(5));
    let programada = t("2026-10-13T10:00:00Z");

    let mut avisos = vec![];
    for minutos in [2, 3, 4, 5, 6, 8, 9] {
        e.fuente
            .poner(RETIRO, vec![demorado(3907, programada, minutos)]);
        s.revisar().await.unwrap();
        avisos.extend(e.avisos());
    }

    assert_eq!(
        avisos,
        vec![
            (3907, Aviso::Demora { minutos: 3 }),
            (3907, Aviso::Demora { minutos: 6 }),
            (3907, Aviso::Demora { minutos: 9 }),
        ]
    );
}

#[tokio::test]
async fn una_cancelacion_se_avisa_una_vez_y_reemplaza_al_recordatorio() {
    let e = Escenario::new("2026-10-13T10:00:00Z");
    let s = e.seguimientos();
    s.seguir(seguir_3907(10));
    let mut cancelado = arribo(3907, t("2026-10-13T10:05:00Z"));
    cancelado.servicio.cancelado = true;
    e.fuente.poner(RETIRO, vec![cancelado]);

    s.revisar().await.unwrap();
    s.revisar().await.unwrap();

    assert_eq!(e.avisos(), vec![(3907, Aviso::Cancelado)]);
}

#[tokio::test]
async fn el_seguimiento_termina_cuando_el_servicio_ya_no_figura_en_la_estacion() {
    let e = Escenario::new("2026-10-13T10:00:00Z");
    let s = e.seguimientos();
    s.seguir(seguir_3907(10));
    e.fuente
        .poner(RETIRO, vec![arribo(3907, t("2026-10-13T10:30:00Z"))]);
    s.revisar().await.unwrap();
    assert_eq!(s.activos().len(), 1);

    e.fuente
        .poner(RETIRO, vec![arribo(3909, t("2026-10-13T10:45:00Z"))]);
    s.revisar().await.unwrap();

    assert!(s.activos().is_empty());
    assert_eq!(e.avisos(), vec![]);
}

#[tokio::test]
async fn cada_estacion_se_consulta_una_sola_vez_por_revision() {
    let e = Escenario::new("2026-10-13T10:00:00Z");
    let s = e.seguimientos();
    s.seguir(seguir_3907(10));
    s.seguir(Seguimiento::nuevo(
        RETIRO,
        "Retiro (LSM)",
        3909,
        "Pilar",
        10,
    ));
    s.seguir(Seguimiento::nuevo(PALERMO, "Palermo", 3907, "Pilar", 5));

    s.revisar().await.unwrap();

    let mut consultas = e.fuente.consultas.borrow().clone();
    consultas.sort_by_key(|e| e.0);
    assert_eq!(consultas, vec![PALERMO, RETIRO]);
}

#[tokio::test]
async fn sin_seguimientos_no_se_consulta_nada() {
    let e = Escenario::new("2026-10-13T10:00:00Z");

    e.seguimientos().revisar().await.unwrap();

    assert!(e.fuente.consultas.borrow().is_empty());
}

#[test]
fn seguir_de_nuevo_reemplaza_y_dejar_de_seguir_lo_quita() {
    let e = Escenario::new("2026-10-13T10:00:00Z");
    let s = e.seguimientos();
    s.seguir(seguir_3907(10));
    s.seguir(seguir_3907(5));
    assert_eq!(s.activos(), vec![seguir_3907(5)]);

    s.dejar_de_seguir(RETIRO, 3907);
    assert!(s.activos().is_empty());
}

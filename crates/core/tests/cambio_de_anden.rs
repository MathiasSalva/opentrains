use chrono::{DateTime, Utc};
use opentrains_core::casos_de_uso::ConsultaArribos;
use opentrains_core::dominio::{
    Arribo, EstacionId, EstadoServicio, Favorito, Ramal, RamalId, Servicio,
};
use opentrains_core::puertos::{ArribosSource, SourceError};
use std::cell::RefCell;

fn t(s: &str) -> DateTime<Utc> {
    s.parse().unwrap()
}

fn arribo(numero: u32, anden: Option<&str>) -> Arribo {
    Arribo {
        servicio: Servicio {
            numero,
            ramal: Ramal {
                id: RamalId(131),
                nombre: "Retiro-Junín".into(),
            },
            destino: "Pilar".into(),
            estado: EstadoServicio {
                id: 10,
                nombre: "Programado".into(),
            },
            formacion: None,
            cancelado: false,
            oculto: false,
            noticia: None,
        },
        anden: anden.map(Into::into),
        anden_anterior: None,
        programada: t("2026-10-15T12:10:00Z"),
        estimada: None,
        momento: t("2026-10-15T12:10:00Z"),
    }
}

/// Devuelve, en cada consulta, la siguiente respuesta preparada.
struct FakeArribos(RefCell<Vec<Vec<Arribo>>>);

impl ArribosSource for &FakeArribos {
    async fn arribos(&self, _: EstacionId, _: Option<RamalId>) -> Result<Vec<Arribo>, SourceError> {
        Ok(self.0.borrow_mut().remove(0))
    }
}

fn retiro() -> Favorito {
    Favorito {
        estacion: EstacionId(463),
        estacion_nombre: "Retiro (LSM)".into(),
        ramal: None,
        coordenadas: None,
        ramales: vec![],
    }
}

/// El andén anterior de cada Arribo en cada consulta sucesiva.
async fn anteriores(respuestas: Vec<Vec<Arribo>>) -> Vec<Vec<Option<String>>> {
    let n = respuestas.len();
    let fuente = FakeArribos(RefCell::new(respuestas));
    let consulta = ConsultaArribos::new(&fuente);
    let mut vistos = vec![];
    for _ in 0..n {
        let arribos = consulta.arribos(&retiro()).await.unwrap();
        vistos.push(arribos.into_iter().map(|a| a.anden_anterior).collect());
    }
    vistos
}

fn s(anden: &str) -> Option<String> {
    Some(anden.into())
}

#[tokio::test]
async fn pasar_de_un_anden_a_otro_es_un_cambio_de_anden() {
    let vistos = anteriores(vec![
        vec![arribo(3907, Some("2"))],
        vec![arribo(3907, Some("4"))],
    ])
    .await;

    assert_eq!(vistos, vec![vec![None], vec![s("2")]]);
}

#[tokio::test]
async fn que_se_asigne_o_se_pierda_el_anden_no_es_un_cambio() {
    let vistos = anteriores(vec![
        vec![arribo(3907, None)],
        vec![arribo(3907, Some("3"))],
        vec![arribo(3907, None)],
        vec![arribo(3907, Some("3"))],
    ])
    .await;

    assert_eq!(vistos, vec![vec![None], vec![None], vec![None], vec![None]]);
}

#[tokio::test]
async fn el_cambio_se_recuerda_hasta_que_el_arribo_sale_de_la_lista() {
    let vistos = anteriores(vec![
        vec![arribo(3907, Some("2")), arribo(3911, Some("1"))],
        vec![arribo(3907, Some("4")), arribo(3911, Some("1"))],
        vec![arribo(3907, Some("4")), arribo(3911, Some("1"))],
        vec![arribo(3911, Some("1"))],
        vec![arribo(3907, Some("5")), arribo(3911, Some("1"))],
    ])
    .await;

    assert_eq!(
        vistos,
        vec![
            vec![None, None],
            vec![s("2"), None],
            vec![s("2"), None],
            vec![None],
            vec![None, None],
        ]
    );
}

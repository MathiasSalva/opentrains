use chrono::{DateTime, Utc};
use opentrains_core::casos_de_uso::ConsultaArribos;
use opentrains_core::dominio::{
    Arribo, EstacionId, EstadoServicio, Favorito, Ramal, RamalId, Servicio, Situacion,
};
use opentrains_core::puertos::{ArribosSource, SourceError};
use std::cell::RefCell;

fn t(s: &str) -> DateTime<Utc> {
    s.parse().unwrap()
}

fn arribo(numero: u32, momento: &str) -> Arribo {
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
            formacion: Some("B953".into()),
            cancelado: false,
            oculto: false,
            noticia: None,
        },
        anden: Some("3".into()),
        anden_anterior: None,
        programada: t(momento),
        estimada: None,
        momento: t(momento),
    }
}

struct FakeArribos {
    respuesta: Result<Vec<Arribo>, SourceError>,
    pedidos: RefCell<Vec<(EstacionId, Option<RamalId>)>>,
}

impl ArribosSource for &FakeArribos {
    async fn arribos(
        &self,
        estacion: EstacionId,
        ramal: Option<RamalId>,
    ) -> Result<Vec<Arribo>, SourceError> {
        self.pedidos.borrow_mut().push((estacion, ramal));
        self.respuesta.clone()
    }
}

fn retiro_a_pilar() -> Favorito {
    Favorito {
        estacion: EstacionId(463),
        estacion_nombre: "Retiro (LSM)".into(),
        ramal: Some(Ramal {
            id: RamalId(131),
            nombre: "Retiro-Junín".into(),
        }),
        coordenadas: None,
        ramales: vec![],
    }
}

#[tokio::test]
async fn los_arribos_de_un_favorito_se_ordenan_por_momento_y_omiten_servicios_ocultos() {
    let mut oculto = arribo(3999, "2026-10-13T10:05:00Z");
    oculto.servicio.oculto = true;
    let fake = FakeArribos {
        respuesta: Ok(vec![
            arribo(3911, "2026-10-13T10:30:00Z"),
            oculto,
            arribo(3907, "2026-10-13T10:10:00Z"),
        ]),
        pedidos: RefCell::default(),
    };

    let arribos = ConsultaArribos::new(&fake)
        .arribos(&retiro_a_pilar())
        .await
        .unwrap();

    let numeros: Vec<u32> = arribos.iter().map(|a| a.servicio.numero).collect();
    assert_eq!(numeros, vec![3907, 3911]);
    assert_eq!(
        *fake.pedidos.borrow(),
        vec![(EstacionId(463), Some(RamalId(131)))]
    );
}

#[tokio::test]
async fn un_error_de_la_fuente_se_propaga() {
    let fake = FakeArribos {
        respuesta: Err(SourceError::FuenteCambiada),
        pedidos: RefCell::default(),
    };

    let resultado = ConsultaArribos::new(&fake).arribos(&retiro_a_pilar()).await;

    assert_eq!(resultado, Err(SourceError::FuenteCambiada));
}

#[test]
fn la_demora_son_los_minutos_entre_estimada_y_programada() {
    let mut a = arribo(3907, "2026-10-13T10:10:00Z");
    a.estimada = Some(t("2026-10-13T10:14:30Z"));
    assert_eq!(a.demora_minutos(), Some(4));
}

#[test]
fn sin_horario_estimado_o_con_menos_de_un_minuto_no_hay_demora() {
    let mut a = arribo(3907, "2026-10-13T10:10:00Z");
    assert_eq!(a.demora_minutos(), None);
    a.estimada = Some(t("2026-10-13T10:10:59Z"));
    assert_eq!(a.demora_minutos(), None);
    a.estimada = Some(t("2026-10-13T10:08:00Z"));
    assert_eq!(a.demora_minutos(), None);
}

fn con_estado(id: u32, nombre: &str) -> Arribo {
    let mut a = arribo(1, "2026-10-13T10:10:00Z");
    a.servicio.estado = EstadoServicio {
        id,
        nombre: nombre.into(),
    };
    a
}

#[test]
fn la_situacion_refleja_el_estado_del_servicio() {
    assert_eq!(
        con_estado(10, "Programado").situacion(),
        Situacion::Programado
    );
    assert_eq!(con_estado(25, "En Andén").situacion(), Situacion::EnAnden);
    assert_eq!(con_estado(35, "Partió").situacion(), Situacion::EnCamino);
    assert_eq!(
        con_estado(77, "Confirmado").situacion(),
        Situacion::Desconocido
    );
}

#[test]
fn una_demora_pesa_mas_que_estar_en_camino_o_programado() {
    for (id, nombre) in [(10, "Programado"), (35, "Partió"), (77, "Confirmado")] {
        let mut a = con_estado(id, nombre);
        a.estimada = Some(t("2026-10-13T10:14:00Z"));
        assert_eq!(a.situacion(), Situacion::Demorado, "estado {nombre}");
    }
}

#[test]
fn estar_en_anden_pesa_mas_que_la_demora() {
    let mut a = con_estado(25, "En Andén");
    a.estimada = Some(t("2026-10-13T10:14:00Z"));
    assert_eq!(a.situacion(), Situacion::EnAnden);
}

#[test]
fn la_cancelacion_pesa_mas_que_todo() {
    let mut a = con_estado(25, "En Andén");
    a.estimada = Some(t("2026-10-13T10:14:00Z"));
    a.servicio.cancelado = true;
    assert_eq!(a.situacion(), Situacion::Cancelado);
}

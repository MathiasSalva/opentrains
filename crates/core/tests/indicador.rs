use chrono::{DateTime, Duration, Utc};
use opentrains_core::dominio::{
    Arribo, EstacionId, EstadoServicio, Ramal, RamalId, Seguimiento, Servicio, Situacion,
};
use opentrains_core::indicador::{indicador, EstacionAbierta};

fn t(s: &str) -> DateTime<Utc> {
    s.parse().unwrap()
}

const AHORA: &str = "2026-10-13T10:00:00Z";

fn arribo(numero: u32, destino: &str, en_segundos: i64) -> Arribo {
    let momento = t(AHORA) + Duration::seconds(en_segundos);
    Arribo {
        servicio: Servicio {
            numero,
            ramal: Ramal {
                id: RamalId(65),
                nombre: "Retiro-Cabred".into(),
            },
            destino: destino.into(),
            estado: EstadoServicio {
                id: 10,
                nombre: "Programado".into(),
            },
            formacion: None,
            cancelado: false,
            oculto: false,
            noticia: None,
        },
        anden: Some("3".into()),
        anden_anterior: None,
        programada: momento,
        estimada: None,
        momento,
    }
}

fn seguido(estacion: &str, a: Arribo) -> Seguimiento {
    let mut s = Seguimiento::nuevo(
        EstacionId(463),
        estacion,
        a.servicio.numero,
        a.servicio.destino.clone(),
        10,
    );
    s.ultimo_arribo = Some(a);
    s
}

fn abierta(arribos: &[Arribo]) -> Option<EstacionAbierta<'_>> {
    Some(EstacionAbierta {
        nombre: "Constitución",
        arribos,
    })
}

#[test]
fn sin_seguimientos_muestra_el_proximo_no_cancelado_de_la_estacion_abierta() {
    let mut cancelado = arribo(1, "Ezeiza", 60);
    cancelado.servicio.cancelado = true;
    let arribos = [cancelado, arribo(2, "Glew", 270), arribo(3, "A. Korn", 900)];

    let i = indicador(&[], abierta(&arribos), t(AHORA)).unwrap();

    assert_eq!(i.estacion_nombre, "Constitución");
    assert_eq!(i.arribo.servicio.numero, 2);
    assert_eq!(i.minutos, 4);
    assert!(!i.arribando);
    assert!(!i.seguido);
}

#[test]
fn un_servicio_seguido_tiene_prioridad_sobre_la_estacion_abierta() {
    let arribos = [arribo(2, "Glew", 60)];
    let seguimientos = [
        seguido("Retiro (LSM)", arribo(3911, "Pilar", 1500)),
        seguido("Retiro (LSM)", arribo(3907, "Pilar", 600)),
    ];

    let i = indicador(&seguimientos, abierta(&arribos), t(AHORA)).unwrap();

    assert_eq!(i.estacion_nombre, "Retiro (LSM)");
    assert_eq!(i.arribo.servicio.numero, 3907, "el seguido que pasa antes");
    assert_eq!(i.minutos, 10);
    assert!(i.seguido);
}

#[test]
fn un_servicio_seguido_cancelado_se_sigue_mostrando() {
    let mut a = arribo(3907, "Pilar", 600);
    a.servicio.cancelado = true;

    let i = indicador(&[seguido("Retiro (LSM)", a)], None, t(AHORA)).unwrap();

    assert_eq!(i.situacion, Situacion::Cancelado);
    assert!(i.seguido);
}

#[test]
fn en_el_ultimo_minuto_esta_arribando_y_los_minutos_no_bajan_de_cero() {
    let arribos = [arribo(2, "Glew", 45)];
    let i = indicador(&[], abierta(&arribos), t(AHORA)).unwrap();
    assert!(i.arribando);
    assert_eq!(i.minutos, 0);

    let seguimientos = [seguido("Retiro (LSM)", arribo(3907, "Pilar", -30))];
    let i = indicador(&seguimientos, None, t(AHORA)).unwrap();
    assert!(i.arribando);
    assert_eq!(i.minutos, 0);
}

#[test]
fn de_la_estacion_abierta_se_saltean_los_que_ya_pasaron() {
    // El último refresco puede tener hasta 45 s: un tren que pasó hace más de un minuto
    // ya no es "el próximo".
    let arribos = [arribo(1, "Ezeiza", -90), arribo(2, "Glew", 300)];

    let i = indicador(&[], abierta(&arribos), t(AHORA)).unwrap();

    assert_eq!(i.arribo.servicio.numero, 2);
}

#[test]
fn sin_seguidos_ni_arribos_no_hay_indicador() {
    let mut cancelado = arribo(1, "Ezeiza", 60);
    cancelado.servicio.cancelado = true;
    let sin_ver = Seguimiento::nuevo(EstacionId(463), "Retiro (LSM)", 3907, "Pilar", 10);

    assert_eq!(indicador(&[], None, t(AHORA)), None);
    assert_eq!(indicador(&[sin_ver], abierta(&[cancelado]), t(AHORA)), None);
}

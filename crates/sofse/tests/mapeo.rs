use chrono::{DateTime, NaiveDate, Utc};
use opentrains_core::dominio::{
    Arribo, Estacion, EstacionId, EstadoServicio, Instalaciones, Linea, LineaId, Ramal, RamalId,
    Servicio,
};
use opentrains_sofse::{credencial, mapeo};

fn fixture(nombre: &str) -> String {
    std::fs::read_to_string(format!(
        "{}/tests/fixtures/{nombre}",
        env!("CARGO_MANIFEST_DIR")
    ))
    .unwrap()
}

fn t(s: &str) -> DateTime<Utc> {
    s.parse().unwrap()
}

#[test]
fn las_gerencias_son_lineas() {
    let lineas = mapeo::lineas(&fixture("gerencias.json")).unwrap();

    assert_eq!(
        lineas,
        vec![
            Linea {
                id: LineaId(1),
                nombre: "Sarmiento".into()
            },
            Linea {
                id: LineaId(31),
                nombre: "San Martín".into()
            },
            Linea {
                id: LineaId(501),
                nombre: "Regionales".into()
            },
        ]
    );
}

#[test]
fn los_ramales_se_mapean_por_id_y_nombre() {
    let ramales = mapeo::ramales(&fixture("ramales.json")).unwrap();

    assert_eq!(
        ramales,
        vec![
            Ramal {
                id: RamalId(131),
                nombre: "Retiro - Junín".into()
            },
            Ramal {
                id: RamalId(65),
                nombre: "Retiro - Cabred".into()
            },
        ]
    );
}

#[test]
fn las_estaciones_se_ordenan_por_orden_y_convierten_sus_campos_de_texto() {
    let estaciones = mapeo::estaciones(&fixture("estaciones.json")).unwrap();

    assert_eq!(
        estaciones,
        vec![
            Estacion {
                id: EstacionId(463),
                nombre: "Retiro (LSM)".into(),
                orden: 1,
                latitud: -34.5888579,
                longitud: -58.3741843,
            },
            Estacion {
                id: EstacionId(297),
                nombre: "Palermo".into(),
                orden: 2,
                latitud: -34.5781,
                longitud: -58.4278,
            },
        ]
    );
}

#[test]
fn los_arribos_toman_horario_destino_estado_y_momento_de_la_respuesta() {
    let arribos = mapeo::arribos(&fixture("arribos.json")).unwrap();

    assert_eq!(
        arribos,
        vec![
            // En cabecera no hay llegada: se usa la salida.
            Arribo {
                servicio: Servicio {
                    numero: 3907,
                    ramal: Ramal {
                        id: RamalId(65),
                        nombre: "Retiro-Cabred".into()
                    },
                    destino: "Pilar".into(),
                    estado: EstadoServicio {
                        id: 25,
                        nombre: "En Andén".into()
                    },
                    formacion: Some("B953".into()),
                    cancelado: false,
                    oculto: false,
                },
                anden: Some("3".into()),
                programada: t("2026-10-13T00:48:00Z"),
                estimada: None,
                momento: t("2026-10-13T00:48:00Z"),
            },
            Arribo {
                servicio: Servicio {
                    numero: 3904,
                    ramal: Ramal {
                        id: RamalId(65),
                        nombre: "Retiro-Cabred".into()
                    },
                    destino: "Retiro (LSM)".into(),
                    estado: EstadoServicio {
                        id: 35,
                        nombre: "Partió".into()
                    },
                    formacion: Some("B965".into()),
                    cancelado: true,
                    oculto: false,
                },
                anden: Some("2".into()),
                programada: t("2026-10-13T01:04:00Z"),
                estimada: Some(t("2026-10-13T01:08:30Z")),
                momento: t("2026-10-13T01:08:30Z"),
            },
            Arribo {
                servicio: Servicio {
                    numero: 9999,
                    ramal: Ramal {
                        id: RamalId(131),
                        nombre: "Retiro-Junín".into()
                    },
                    destino: "Junín".into(),
                    estado: EstadoServicio {
                        id: 10,
                        nombre: "Programado".into()
                    },
                    formacion: None,
                    cancelado: false,
                    oculto: true,
                },
                anden: None,
                programada: t("2026-10-13T01:20:00Z"),
                estimada: None,
                momento: t("2026-10-13T01:08:00Z"),
            },
        ]
    );
}

#[test]
fn las_instalaciones_convierten_si_no_en_booleanos() {
    let instalaciones = mapeo::instalaciones(&fixture("instalaciones.json")).unwrap();

    assert_eq!(
        instalaciones,
        Some(Instalaciones {
            direccion: Some("Av. Dr. José María Ramos Mejía 1500, CABA.".into()),
            boleteria: Some("Lunes a viernes 24h, sábados y domingos de 5 a 21 h".into()),
            ascensor: false,
            rampa: true,
            banos: true,
            wifi: true,
            terminal_sube: true,
            conexiones: Some("Subte C - Subte E - Tren Mitre".into()),
            colectivos: Some("5 - 6 - 7 - 9".into()),
        })
    );
}

#[test]
fn una_estacion_sin_relevamiento_no_tiene_instalaciones() {
    assert_eq!(mapeo::instalaciones("[]").unwrap(), None);
}

#[test]
fn un_json_inesperado_es_respuesta_invalida() {
    assert!(matches!(
        mapeo::lineas("{\"code\":500}"),
        Err(opentrains_core::puertos::SourceError::RespuestaInvalida(_))
    ));
}

#[test]
fn la_credencial_diaria_deriva_de_la_fecha_argentina() {
    let (usuario, clave) = credencial::para_fecha(NaiveDate::from_ymd_opt(2026, 10, 13).unwrap());

    // Valores de referencia calculados con el algoritmo de docs/reverse-engineering §2.
    assert_eq!(usuario, "MjAyNjEwMTNzb2ZzZQ==");
    assert_eq!(clave, "v%23Q1VwJ0I4VVNxJFWk5kVFVjNZpmSjQ3I4xGcSBFVwMyZ");
}

#[test]
fn la_fecha_de_la_credencial_es_la_de_argentina_y_no_la_utc() {
    // 02:00 UTC del 14 son las 23:00 del 13 en Argentina.
    assert_eq!(
        credencial::fecha_argentina(t("2026-10-14T02:00:00Z")),
        NaiveDate::from_ymd_opt(2026, 10, 13).unwrap()
    );
}

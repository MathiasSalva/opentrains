use chrono::{DateTime, NaiveDate, Utc};
use opentrains_core::dominio::{
    AlcanceNoticia, Arribo, Estacion, EstacionId, EstadoServicio, Instalaciones, Linea, LineaId,
    Noticia, Ramal, RamalId, Servicio,
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
                ramales: vec![RamalId(31), RamalId(65), RamalId(131)],
            },
            Estacion {
                id: EstacionId(297),
                nombre: "Palermo".into(),
                orden: 2,
                latitud: -34.5781,
                longitud: -58.4278,
                ramales: vec![RamalId(31), RamalId(65), RamalId(131)],
            },
        ]
    );
}

#[test]
fn los_arribos_toman_horario_destino_estado_y_momento_de_la_respuesta() {
    // El momento se ancla a cuándo recibimos la respuesta (reloj local), no al
    // `timestamp` del servidor (00:48:00): así un desfase de relojes no corre la
    // cuenta regresiva.
    let recibido = t("2026-10-13T00:50:00Z");
    let arribos = mapeo::arribos(&fixture("arribos.json"), recibido).unwrap();

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
                    noticia: None,
                },
                anden: Some("3".into()),
                anden_anterior: None,
                programada: t("2026-10-13T00:48:00Z"),
                estimada: None,
                momento: t("2026-10-13T00:50:00Z"),
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
                    noticia: Some("Circula con 4 coches".into()),
                },
                anden: Some("2".into()),
                anden_anterior: None,
                programada: t("2026-10-13T01:04:00Z"),
                estimada: Some(t("2026-10-13T01:08:30Z")),
                momento: t("2026-10-13T01:10:30Z"),
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
                    noticia: None,
                },
                anden: None,
                anden_anterior: None,
                programada: t("2026-10-13T01:20:00Z"),
                estimada: None,
                momento: t("2026-10-13T01:10:00Z"),
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

#[test]
fn las_estaciones_cercanas_conservan_el_orden_de_la_api() {
    // Mismo esquema que las de un Ramal, pero `incluida_en_ramales` viene como string
    // JSON y `orden` es el de su propio Ramal: no sirve para ordenar.
    let estaciones = mapeo::estaciones_cercanas(&fixture("estaciones_cercanas.json")).unwrap();

    let ids: Vec<u32> = estaciones.iter().map(|e| e.id.0).collect();
    assert_eq!(ids, vec![332, 463, 293, 93]);
    assert_eq!(
        estaciones[1],
        Estacion {
            id: EstacionId(463),
            nombre: "Retiro (LSM)".into(),
            orden: 1,
            latitud: -34.5888579,
            longitud: -58.3741843,
            ramales: vec![RamalId(31), RamalId(65), RamalId(131)],
        }
    );
}

#[test]
fn las_alertas_de_lineas_y_ramales_son_noticias() {
    let ramales_roca = r#"[
        {"id": 17, "nombre": "Constitución-Alejandro Korn", "id_gerencia": 11, "alerta": null},
        {"id": 19, "nombre": "Constitución-Ezeiza", "id_gerencia": 11, "alerta": null}
    ]"#;
    // La forma de la alerta de un Ramal no se vio con datos: se asume la misma que la de
    // una Línea, y puede venir sola o en lista. Una alerta repetida se toma una vez.
    let ramales_san_martin = r#"[
        {"id": 131, "nombre": "Retiro-Junín", "id_gerencia": 31,
         "alerta": {"id": 7003, "linea_id": 31, "ramal_id": 131, "estacion_id": null,
                    "contenido": "Demoras por arrollamiento.", "habilitado": 1,
                    "vigencia_desde": "2026-10-15 09:12:00", "vigencia_hasta": null, "criticidad_orden": 1}},
        {"id": 65, "nombre": "Retiro-Cabred", "id_gerencia": 31,
         "alerta": [{"id": 7001, "linea_id": 31, "ramal_id": 65, "contenido": "Servicio limitado a Hurlingham por obras.",
                     "habilitado": 1, "criticidad_orden": 2}]}
    ]"#;

    let noticias = mapeo::noticias(
        &fixture("gerencias_con_alertas.json"),
        &[ramales_roca.into(), ramales_san_martin.into()],
    )
    .unwrap();

    assert_eq!(
        noticias,
        vec![
            Noticia {
                id: 98433,
                alcance: AlcanceNoticia::Linea(vec![RamalId(17), RamalId(19)]),
                texto: "Las personas con CUD pueden viajar en tren sin costo con SUBE.".into(),
                criticidad: Some(4),
                habilitada: true,
                // La vigencia viene en hora argentina.
                desde: Some(t("2026-10-13T03:01:00Z")),
                hasta: None,
            },
            Noticia {
                id: 7001,
                alcance: AlcanceNoticia::Ramal(RamalId(65)),
                texto: "Servicio limitado a Hurlingham por obras.".into(),
                criticidad: Some(2),
                habilitada: true,
                desde: Some(t("2026-10-15T08:00:00Z")),
                hasta: Some(t("2026-10-16T02:30:00Z")),
            },
            Noticia {
                id: 7002,
                alcance: AlcanceNoticia::Estacion(EstacionId(463)),
                texto: "Ascensor del andén 3 fuera de servicio.".into(),
                criticidad: None,
                habilitada: false,
                desde: None,
                hasta: None,
            },
            Noticia {
                id: 7003,
                alcance: AlcanceNoticia::Ramal(RamalId(131)),
                texto: "Demoras por arrollamiento.".into(),
                criticidad: Some(1),
                habilitada: true,
                desde: Some(t("2026-10-15T12:12:00Z")),
                hasta: None,
            },
        ]
    );
}

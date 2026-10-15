use opentrains_core::casos_de_uso::Cercania;
use opentrains_core::dominio::{Coordenadas, Distancia, Estacion, EstacionId};
use opentrains_core::puertos::{EstacionesCercanasSource, SourceError, UbicacionSource};
use std::cell::{Cell, RefCell};

const RETIRO_LSM: Coordenadas = Coordenadas {
    latitud: -34.5888579,
    longitud: -58.3741843,
};
const PALERMO: Coordenadas = Coordenadas {
    latitud: -34.5776,
    longitud: -58.4265,
};

fn estacion(id: u32, nombre: &str, en: Coordenadas) -> Estacion {
    Estacion {
        id: EstacionId(id),
        nombre: nombre.into(),
        orden: 1,
        latitud: en.latitud,
        longitud: en.longitud,
        ramales: vec![],
    }
}

fn texto(metros: f64) -> String {
    Distancia { metros }.to_string()
}

#[test]
fn la_distancia_es_en_linea_recta() {
    // Retiro (LSM) → Palermo: ~5 km según cualquier mapa.
    let d = RETIRO_LSM.distancia(&PALERMO);
    assert!((d.metros - 4_987.0).abs() < 50.0, "{}", d.metros);
    assert_eq!(RETIRO_LSM.distancia(&RETIRO_LSM).metros, 0.0);
}

#[test]
fn la_distancia_se_redondea_segun_su_magnitud() {
    assert_eq!(texto(0.0), "a 0 m");
    assert_eq!(texto(853.0), "a 850 m");
    assert_eq!(
        texto(996.0),
        "a 1,0 km",
        "si el redondeo llega a 1000 m, pasa a km"
    );
    assert_eq!(texto(3_249.0), "a 3,2 km");
    assert_eq!(
        texto(9_970.0),
        "a 10 km",
        "si el redondeo llega a 10 km, sin decimal"
    );
    assert_eq!(texto(27_400.0), "a 27 km");
}

#[derive(Default)]
struct FakeUbicacion(RefCell<Option<Coordenadas>>);

impl UbicacionSource for &FakeUbicacion {
    async fn ubicacion(&self) -> Option<Coordenadas> {
        *self.0.borrow()
    }
}

#[derive(Default)]
struct FakeCercanas {
    estaciones: Vec<Estacion>,
    llamadas: Cell<u32>,
}

impl EstacionesCercanasSource for &FakeCercanas {
    async fn estaciones_cercanas(&self, _: Coordenadas) -> Result<Vec<Estacion>, SourceError> {
        self.llamadas.set(self.llamadas.get() + 1);
        Ok(self.estaciones.clone())
    }
}

fn en(c: Coordenadas) -> FakeUbicacion {
    FakeUbicacion(RefCell::new(Some(c)))
}

#[tokio::test]
async fn sin_ubicacion_no_hay_distancias_ni_cercanas() {
    let ubicacion = FakeUbicacion::default();
    let fuente = FakeCercanas::default();
    let cercania = Cercania::new(&ubicacion, &fuente);

    assert_eq!(cercania.actualizar().await, None);
    assert_eq!(cercania.distancias(&[PALERMO]), None);
    assert_eq!(cercania.cercanas().await.unwrap(), None);
    assert_eq!(fuente.llamadas.get(), 0);
}

#[tokio::test]
async fn se_conserva_la_ultima_ubicacion_conocida() {
    let ubicacion = en(RETIRO_LSM);
    let fuente = FakeCercanas::default();
    let cercania = Cercania::new(&ubicacion, &fuente);
    cercania.actualizar().await;

    *ubicacion.0.borrow_mut() = None;

    assert_eq!(cercania.actualizar().await, Some(RETIRO_LSM));
    let distancias = cercania.distancias(&[PALERMO, RETIRO_LSM]).unwrap();
    assert_eq!(distancias[1].metros, 0.0);
}

#[tokio::test]
async fn las_cercanas_son_las_tres_mas_proximas_sin_repetir() {
    let ubicacion = en(RETIRO_LSM);
    let lejos = Coordenadas {
        latitud: -34.70,
        longitud: -58.50,
    };
    let fuente = FakeCercanas {
        // La API ordena por cercanía, pero no siempre bien: se reordena por distancia.
        estaciones: vec![
            estacion(332, "Retiro (LGM)", RETIRO_LSM),
            estacion(93, "Lejana", lejos),
            estacion(297, "Palermo", PALERMO),
            estacion(332, "Retiro (LGM)", RETIRO_LSM),
            estacion(463, "Retiro (LSM)", RETIRO_LSM),
        ],
        ..Default::default()
    };
    let cercania = Cercania::new(&ubicacion, &fuente);
    cercania.actualizar().await;

    let cercanas = cercania.cercanas().await.unwrap().unwrap();

    let ids: Vec<u32> = cercanas.iter().map(|c| c.estacion.id.0).collect();
    assert_eq!(ids, vec![332, 463, 297]);
    assert!((cercanas[2].distancia.metros - 4_987.0).abs() < 50.0);
}

#[tokio::test]
async fn las_cercanas_se_vuelven_a_pedir_solo_si_te_moviste() {
    let ubicacion = en(RETIRO_LSM);
    let fuente = FakeCercanas {
        estaciones: vec![estacion(297, "Palermo", PALERMO)],
        ..Default::default()
    };
    let cercania = Cercania::new(&ubicacion, &fuente);

    cercania.actualizar().await;
    cercania.cercanas().await.unwrap();
    // Unos metros de ruido del Wi-Fi no justifican otra consulta.
    *ubicacion.0.borrow_mut() = Some(Coordenadas {
        latitud: RETIRO_LSM.latitud + 0.0005,
        ..RETIRO_LSM
    });
    cercania.actualizar().await;
    let cercanas = cercania.cercanas().await.unwrap().unwrap();
    assert_eq!(fuente.llamadas.get(), 1);
    assert!(
        cercanas[0].distancia.metros < 4_987.0,
        "la distancia sí se recalcula con la ubicación nueva"
    );

    *ubicacion.0.borrow_mut() = Some(PALERMO);
    cercania.actualizar().await;
    cercania.cercanas().await.unwrap();
    assert_eq!(fuente.llamadas.get(), 2);
}

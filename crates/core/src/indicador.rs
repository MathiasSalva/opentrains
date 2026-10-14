//! Qué tren se muestra de un vistazo (en Windows, en el ícono de la bandeja) y en qué estado.

use chrono::{DateTime, Duration, Utc};
use serde::Serialize;

use crate::dominio::{Arribo, Seguimiento, Situacion};

/// El Arribo que se muestra de un vistazo, con su cuenta regresiva ya calculada.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Indicador {
    pub estacion_nombre: String,
    pub arribo: Arribo,
    /// Minutos enteros que faltan; 0 en el último minuto o si ya debería haber pasado.
    pub minutos: i64,
    pub arribando: bool,
    pub situacion: Situacion,
    /// Si es un Servicio que el usuario sigue.
    pub seguido: bool,
}

/// Arribos de la Estación abierta tal como se vieron en el último refresco.
pub struct EstacionAbierta<'a> {
    pub nombre: &'a str,
    pub arribos: &'a [Arribo],
}

/// Prioridad: el Servicio seguido que pasa antes; si no hay, el próximo Arribo no cancelado
/// de la Estación abierta.
pub fn indicador(
    seguimientos: &[Seguimiento],
    abierta: Option<EstacionAbierta<'_>>,
    ahora: DateTime<Utc>,
) -> Option<Indicador> {
    let seguido = seguimientos
        .iter()
        .filter_map(|s| Some((s.estacion_nombre.as_str(), s.ultimo_arribo.as_ref()?)))
        .min_by_key(|(_, a)| a.momento);
    if let Some((nombre, arribo)) = seguido {
        return Some(armar(nombre, arribo, true, ahora));
    }
    let abierta = abierta?;
    // El último refresco puede tener hasta un ciclo de antigüedad: lo que pasó hace más de
    // un minuto ya no es "el próximo".
    let limite = ahora - Duration::seconds(60);
    abierta
        .arribos
        .iter()
        .find(|a| !a.servicio.cancelado && a.momento >= limite)
        .map(|a| armar(abierta.nombre, a, false, ahora))
}

fn armar(nombre: &str, arribo: &Arribo, seguido: bool, ahora: DateTime<Utc>) -> Indicador {
    let segundos = (arribo.momento - ahora).num_seconds();
    Indicador {
        estacion_nombre: nombre.to_owned(),
        minutos: segundos.max(0) / 60,
        arribando: segundos <= 60,
        situacion: arribo.situacion(),
        seguido,
        arribo: arribo.clone(),
    }
}

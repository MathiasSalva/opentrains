//! Smoke test manual contra la API real: `cargo run -p opentrains-sofse --example smoke`.

use opentrains_core::dominio::{EstacionId, LineaId, RamalId};
use opentrains_core::puertos::{ArribosSource, CatalogoSource, InstalacionesSource};
use opentrains_sofse::SofseClient;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let cliente = SofseClient::default();
    let lineas = cliente.lineas().await.expect("lineas");
    println!("líneas: {:?}", lineas.iter().map(|l| (l.id.0, &l.nombre)).collect::<Vec<_>>());
    let ramales = cliente.ramales(LineaId(31)).await.expect("ramales");
    println!("ramales San Martín: {:?}", ramales.iter().map(|r| (r.id.0, &r.nombre)).collect::<Vec<_>>());
    let estaciones = cliente.estaciones(RamalId(131)).await.expect("estaciones");
    println!("estaciones ramal 131: {} (primera: {:?})", estaciones.len(), estaciones.first().map(|e| &e.nombre));
    let arribos = cliente.arribos(EstacionId(463), None).await.expect("arribos");
    println!("arribos Retiro LSM: {}", arribos.len());
    for a in arribos.iter().take(5) {
        println!(
            "  #{} a {} | {} ({}) | andén {:?} | prog {} est {:?} momento {} | cancelado {} oculto {}",
            a.servicio.numero, a.servicio.destino, a.servicio.estado.nombre, a.servicio.estado.id,
            a.anden, a.programada, a.estimada, a.momento, a.servicio.cancelado, a.servicio.oculto
        );
    }
    println!("instalaciones 463: {:?}", cliente.instalaciones(EstacionId(463)).await.expect("instalaciones"));
}

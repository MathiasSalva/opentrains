use opentrains_core::casos_de_uso::Favoritos;
use opentrains_core::dominio::{EstacionId, Favorito, Preferencias, Ramal, RamalId};
use opentrains_core::puertos::{PreferenciasRepo, StorageError};
use std::cell::RefCell;

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

fn favorito(estacion: u32, ramal: Option<u32>) -> Favorito {
    Favorito {
        estacion: EstacionId(estacion),
        estacion_nombre: format!("Estación {estacion}"),
        ramal: ramal.map(|id| Ramal {
            id: RamalId(id),
            nombre: format!("Ramal {id}"),
        }),
    }
}

#[test]
fn agregar_un_favorito_lo_persiste_sin_duplicarlo() {
    let repo = FakeRepo::default();
    let favoritos = Favoritos::new(&repo);

    favoritos.agregar(favorito(463, Some(131))).unwrap();
    favoritos.agregar(favorito(463, None)).unwrap();
    let preferencias = favoritos.agregar(favorito(463, Some(131))).unwrap();

    assert_eq!(
        preferencias.favoritos,
        vec![favorito(463, Some(131)), favorito(463, None)]
    );
    assert_eq!(repo.0.borrow().favoritos, preferencias.favoritos);
}

#[test]
fn quitar_un_favorito_distingue_el_ramal() {
    let repo = FakeRepo::default();
    let favoritos = Favoritos::new(&repo);
    favoritos.agregar(favorito(463, Some(131))).unwrap();
    favoritos.agregar(favorito(463, None)).unwrap();

    let preferencias = favoritos.quitar(&favorito(463, None)).unwrap();

    assert_eq!(preferencias.favoritos, vec![favorito(463, Some(131))]);
}

#[test]
fn abrir_un_favorito_lo_recuerda_como_ultimo_abierto() {
    let repo = FakeRepo::default();
    let favoritos = Favoritos::new(&repo);
    favoritos.agregar(favorito(463, Some(131))).unwrap();

    favoritos.abrir(favorito(293, None)).unwrap();

    assert_eq!(
        favoritos.preferencias().unwrap().ultimo_abierto,
        Some(favorito(293, None))
    );
}

#[test]
fn quitar_el_ultimo_abierto_no_lo_olvida() {
    // Abrir una Estación no la convierte en Favorito: el último abierto es independiente.
    let repo = FakeRepo::default();
    let favoritos = Favoritos::new(&repo);
    favoritos.agregar(favorito(463, None)).unwrap();
    favoritos.abrir(favorito(463, None)).unwrap();

    let preferencias = favoritos.quitar(&favorito(463, None)).unwrap();

    assert!(preferencias.favoritos.is_empty());
    assert_eq!(preferencias.ultimo_abierto, Some(favorito(463, None)));
}

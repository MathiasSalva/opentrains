//! Adaptador de salida: preferencias y caché del catálogo como archivos JSON.

use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use opentrains_core::dominio::Preferencias;
use opentrains_core::puertos::{
    CatalogoCache, ClaveCatalogo, EntradaCatalogo, PreferenciasRepo, StorageError,
};
use serde::de::DeserializeOwned;
use serde::Serialize;

pub struct JsonStore {
    directorio: PathBuf,
}

impl JsonStore {
    pub fn new(directorio: impl Into<PathBuf>) -> Self {
        Self {
            directorio: directorio.into(),
        }
    }

    fn archivo_catalogo(&self, clave: &ClaveCatalogo) -> PathBuf {
        let nombre = match clave {
            ClaveCatalogo::Lineas => "lineas.json".to_owned(),
            ClaveCatalogo::Ramales(linea) => format!("ramales-{}.json", linea.0),
            ClaveCatalogo::Estaciones(ramal) => format!("estaciones-{}.json", ramal.0),
        };
        self.directorio.join("catalogo").join(nombre)
    }
}

impl PreferenciasRepo for JsonStore {
    fn cargar(&self) -> Result<Preferencias, StorageError> {
        Ok(leer(&self.directorio.join("preferencias.json"))?.unwrap_or_default())
    }

    fn guardar(&self, preferencias: &Preferencias) -> Result<(), StorageError> {
        escribir(&self.directorio.join("preferencias.json"), preferencias)
    }
}

impl CatalogoCache for JsonStore {
    fn leer(&self, clave: &ClaveCatalogo) -> Result<Option<EntradaCatalogo>, StorageError> {
        leer(&self.archivo_catalogo(clave))
    }

    fn guardar(
        &self,
        clave: &ClaveCatalogo,
        entrada: &EntradaCatalogo,
    ) -> Result<(), StorageError> {
        escribir(&self.archivo_catalogo(clave), entrada)
    }
}

fn leer<T: DeserializeOwned>(archivo: &Path) -> Result<Option<T>, StorageError> {
    match fs::read_to_string(archivo) {
        Ok(json) => serde_json::from_str(&json)
            .map(Some)
            .map_err(|e| StorageError(format!("{}: {e}", archivo.display()))),
        Err(e) if e.kind() == ErrorKind::NotFound => Ok(None),
        Err(e) => Err(StorageError(format!("{}: {e}", archivo.display()))),
    }
}

/// Escribe en un archivo temporal y lo renombra, para no dejar JSON a medias.
fn escribir<T: Serialize>(archivo: &Path, valor: &T) -> Result<(), StorageError> {
    let error = |e: &dyn std::fmt::Display| StorageError(format!("{}: {e}", archivo.display()));
    if let Some(padre) = archivo.parent() {
        fs::create_dir_all(padre).map_err(|e| error(&e))?;
    }
    let json = serde_json::to_string_pretty(valor).map_err(|e| error(&e))?;
    let temporal = archivo.with_extension("json.tmp");
    fs::write(&temporal, json).map_err(|e| error(&e))?;
    fs::rename(&temporal, archivo).map_err(|e| error(&e))
}

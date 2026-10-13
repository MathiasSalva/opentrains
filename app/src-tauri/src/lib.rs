//! Adaptador de entrada y punto de composición: conecta los casos de uso de `core`
//! con los adaptadores de SOFSE y de almacenamiento, y los expone como comandos Tauri.

use std::path::PathBuf;
use std::sync::Arc;

use chrono::{DateTime, Utc};
use opentrains_core::casos_de_uso::{Catalogo, ConsultaArribos, Favoritos};
use opentrains_core::dominio::{
    Arribo, Estacion, EstacionId, Favorito, Instalaciones, Linea, LineaId, Preferencias, Ramal,
    RamalId, Tono,
};
use opentrains_core::puertos::{InstalacionesSource, RelojSistema, SourceError, StorageError};
use opentrains_sofse::SofseClient;
use opentrains_storage::JsonStore;
use serde::Serialize;
use tauri::{Manager, State};

struct Estado {
    catalogo: Catalogo<Arc<SofseClient>, Arc<JsonStore>, RelojSistema>,
    arribos: ConsultaArribos<Arc<SofseClient>>,
    instalaciones: Arc<SofseClient>,
    favoritos: Favoritos<Arc<JsonStore>>,
}

/// Errores tal como los ve el frontend.
#[derive(Serialize)]
#[serde(tag = "tipo", content = "detalle", rename_all = "snake_case")]
enum ErrorApp {
    Fuente(SourceError),
    Almacenamiento(String),
}

impl From<SourceError> for ErrorApp {
    fn from(error: SourceError) -> Self {
        ErrorApp::Fuente(error)
    }
}

impl From<StorageError> for ErrorApp {
    fn from(error: StorageError) -> Self {
        ErrorApp::Almacenamiento(error.0)
    }
}

/// Un Arribo con lo derivado ya calculado para la vista.
#[derive(Serialize)]
struct ArriboVista {
    numero: u32,
    ramal: String,
    destino: String,
    estado: String,
    tono: Tono,
    cancelado: bool,
    formacion: Option<String>,
    anden: Option<String>,
    programada: DateTime<Utc>,
    demora_minutos: Option<i64>,
    momento: DateTime<Utc>,
}

impl From<Arribo> for ArriboVista {
    fn from(a: Arribo) -> Self {
        Self {
            demora_minutos: a.demora_minutos(),
            tono: a.tono(),
            numero: a.servicio.numero,
            ramal: a.servicio.ramal.nombre,
            destino: a.servicio.destino,
            estado: a.servicio.estado.nombre,
            cancelado: a.servicio.cancelado,
            formacion: a.servicio.formacion,
            anden: a.anden,
            programada: a.programada,
            momento: a.momento,
        }
    }
}

#[tauri::command]
async fn lineas(estado: State<'_, Estado>) -> Result<Vec<Linea>, ErrorApp> {
    Ok(estado.catalogo.lineas().await?)
}

#[tauri::command]
async fn ramales(estado: State<'_, Estado>, linea: u32) -> Result<Vec<Ramal>, ErrorApp> {
    Ok(estado.catalogo.ramales(LineaId(linea)).await?)
}

#[tauri::command]
async fn estaciones(estado: State<'_, Estado>, ramal: u32) -> Result<Vec<Estacion>, ErrorApp> {
    Ok(estado.catalogo.estaciones(RamalId(ramal)).await?)
}

#[tauri::command]
async fn arribos(
    estado: State<'_, Estado>,
    favorito: Favorito,
) -> Result<Vec<ArriboVista>, ErrorApp> {
    let arribos = estado.arribos.arribos(&favorito).await?;
    Ok(arribos.into_iter().map(ArriboVista::from).collect())
}

#[tauri::command]
async fn instalaciones(
    estado: State<'_, Estado>,
    estacion: u32,
) -> Result<Option<Instalaciones>, ErrorApp> {
    Ok(estado
        .instalaciones
        .instalaciones(EstacionId(estacion))
        .await?)
}

#[tauri::command]
fn preferencias(estado: State<'_, Estado>) -> Result<Preferencias, ErrorApp> {
    Ok(estado.favoritos.preferencias()?)
}

#[tauri::command]
fn agregar_favorito(
    estado: State<'_, Estado>,
    favorito: Favorito,
) -> Result<Preferencias, ErrorApp> {
    Ok(estado.favoritos.agregar(favorito)?)
}

#[tauri::command]
fn quitar_favorito(
    estado: State<'_, Estado>,
    favorito: Favorito,
) -> Result<Preferencias, ErrorApp> {
    Ok(estado.favoritos.quitar(&favorito)?)
}

#[tauri::command]
fn abrir(estado: State<'_, Estado>, favorito: Favorito) -> Result<Preferencias, ErrorApp> {
    Ok(estado.favoritos.abrir(favorito)?)
}

/// `%APPDATA%\OpenTrains` en Windows; el directorio de datos de la app en otras plataformas.
fn directorio_datos(app: &tauri::App) -> PathBuf {
    #[cfg(windows)]
    if let Some(appdata) = std::env::var_os("APPDATA") {
        return PathBuf::from(appdata).join("OpenTrains");
    }
    app.path()
        .app_data_dir()
        .expect("directorio de datos de la app")
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let sofse = Arc::new(SofseClient::default());
            let store = Arc::new(JsonStore::new(directorio_datos(app)));
            app.manage(Estado {
                catalogo: Catalogo::new(sofse.clone(), store.clone(), RelojSistema),
                arribos: ConsultaArribos::new(sofse.clone()),
                instalaciones: sofse,
                favoritos: Favoritos::new(store),
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            lineas,
            ramales,
            estaciones,
            arribos,
            instalaciones,
            preferencias,
            agregar_favorito,
            quitar_favorito,
            abrir
        ])
        .run(tauri::generate_context!())
        .expect("error al iniciar OpenTrains");
}

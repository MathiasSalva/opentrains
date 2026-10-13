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
use opentrains_core::puertos::{InstalacionesSource, SourceError, StorageError, SystemClock};
use opentrains_sofse::SofseClient;
use opentrains_storage::JsonStore;
use serde::Serialize;
use tauri::{Manager, State};

struct AppState {
    catalogo: Catalogo<Arc<SofseClient>, Arc<JsonStore>, SystemClock>,
    arribos: ConsultaArribos<Arc<SofseClient>>,
    instalaciones: Arc<SofseClient>, // Sin lógica de dominio: el caso de uso sería un pasamanos.
    favoritos: Favoritos<Arc<JsonStore>>,
}

/// Errores tal como los ve el frontend.
#[derive(Serialize)]
#[serde(tag = "tipo", content = "detalle", rename_all = "snake_case")]
enum AppError {
    Fuente(SourceError),
    Almacenamiento(String),
}

impl From<SourceError> for AppError {
    fn from(error: SourceError) -> Self {
        AppError::Fuente(error)
    }
}

impl From<StorageError> for AppError {
    fn from(error: StorageError) -> Self {
        AppError::Almacenamiento(error.0)
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
async fn lineas(app: State<'_, AppState>) -> Result<Vec<Linea>, AppError> {
    Ok(app.catalogo.lineas().await?)
}

#[tauri::command]
async fn ramales(app: State<'_, AppState>, linea: LineaId) -> Result<Vec<Ramal>, AppError> {
    Ok(app.catalogo.ramales(linea).await?)
}

#[tauri::command]
async fn estaciones(app: State<'_, AppState>, ramal: RamalId) -> Result<Vec<Estacion>, AppError> {
    Ok(app.catalogo.estaciones(ramal).await?)
}

#[tauri::command]
async fn arribos(
    app: State<'_, AppState>,
    favorito: Favorito,
) -> Result<Vec<ArriboVista>, AppError> {
    let arribos = app.arribos.arribos(&favorito).await?;
    Ok(arribos.into_iter().map(ArriboVista::from).collect())
}

#[tauri::command]
async fn instalaciones(
    app: State<'_, AppState>,
    estacion: EstacionId,
) -> Result<Option<Instalaciones>, AppError> {
    Ok(app.instalaciones.instalaciones(estacion).await?)
}

#[tauri::command]
fn preferencias(app: State<'_, AppState>) -> Result<Preferencias, AppError> {
    Ok(app.favoritos.preferencias()?)
}

#[tauri::command]
fn agregar_favorito(
    app: State<'_, AppState>,
    favorito: Favorito,
) -> Result<Preferencias, AppError> {
    Ok(app.favoritos.agregar(favorito)?)
}

#[tauri::command]
fn quitar_favorito(app: State<'_, AppState>, favorito: Favorito) -> Result<Preferencias, AppError> {
    Ok(app.favoritos.quitar(&favorito)?)
}

#[tauri::command]
fn abrir(app: State<'_, AppState>, favorito: Favorito) -> Result<Preferencias, AppError> {
    Ok(app.favoritos.abrir(favorito)?)
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
            app.manage(AppState {
                catalogo: Catalogo::new(sofse.clone(), store.clone(), SystemClock),
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

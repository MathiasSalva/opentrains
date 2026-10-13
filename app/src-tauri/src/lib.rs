//! Adaptador de entrada y punto de composición: conecta los casos de uso de `core`
//! con los adaptadores de SOFSE y de almacenamiento, y los expone como comandos Tauri.

#[cfg(desktop)]
mod bandeja;

use std::path::PathBuf;
use std::sync::Arc;

use chrono::{DateTime, Utc};
use opentrains_core::casos_de_uso::{Catalogo, ConsultaArribos, Favoritos};
use opentrains_core::dominio::{
    Arribo, Estacion, EstacionId, Favorito, Instalaciones, Linea, LineaId, Preferencias, Ramal,
    RamalId, Situacion,
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
    ramal: Ramal,
    destino: String,
    /// Texto del Estado del Servicio tal como lo informa el operador.
    estado: String,
    situacion: Situacion,
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
            situacion: a.situacion(),
            numero: a.servicio.numero,
            ramal: a.servicio.ramal,
            destino: a.servicio.destino,
            estado: a.servicio.estado.nombre,
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

/// Texto del ícono de la bandeja con el próximo tren; lo arma el frontend en cada refresco.
#[tauri::command]
fn actualizar_bandeja(app: tauri::AppHandle, texto: String) {
    #[cfg(desktop)]
    bandeja::tooltip(&app, &texto);
    #[cfg(not(desktop))]
    let _ = (app, texto);
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
    let builder = tauri::Builder::default();
    #[cfg(desktop)]
    let builder = builder
        // Abrir la app de nuevo (por ejemplo desde el menú Inicio) muestra el panel de la
        // instancia que ya corre en la bandeja.
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            bandeja::mostrar(app)
        }))
        .plugin(
            tauri_plugin_autostart::Builder::new()
                .args([bandeja::ARG_OCULTA])
                .build(),
        )
        .manage(bandeja::EstadoPanel::default());
    builder
        .setup(|app| {
            let sofse = Arc::new(SofseClient::default());
            let store = Arc::new(JsonStore::new(directorio_datos(app)));
            app.manage(AppState {
                catalogo: Catalogo::new(sofse.clone(), store.clone(), SystemClock),
                arribos: ConsultaArribos::new(sofse.clone()),
                instalaciones: sofse,
                favoritos: Favoritos::new(store),
            });
            #[cfg(desktop)]
            {
                bandeja::crear(app.handle())?;
                if let Some(ventana) = app.get_webview_window("main") {
                    bandeja::comportamiento_panel(&ventana);
                }
                // Al iniciar con Windows arranca oculta; abierta a mano, muestra el panel.
                if !std::env::args().any(|a| a == bandeja::ARG_OCULTA) {
                    bandeja::mostrar(app.handle());
                }
            }
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
            abrir,
            actualizar_bandeja
        ])
        .run(tauri::generate_context!())
        .expect("error al iniciar OpenTrains");
}

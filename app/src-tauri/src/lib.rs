//! Adaptador de entrada y punto de composición: conecta los casos de uso de `core`
//! con los adaptadores de SOFSE y de almacenamiento, y los expone como comandos Tauri.

mod avisos;
#[cfg(desktop)]
mod bandeja;
#[cfg(desktop)]
mod insignia;
mod ubicacion;

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use chrono::{DateTime, Utc};
use opentrains_core::casos_de_uso::{
    Catalogo, Cercania, ConsultaArribos, Favoritos, Noticias, Seguimientos,
};
use opentrains_core::dominio::{
    Arribo, Coordenadas, Estacion, EstacionId, Favorito, Instalaciones, Linea, LineaId, Noticia,
    Preferencias, Ramal, RamalId, Seguimiento, Situacion,
};
use opentrains_core::puertos::{
    EstacionesCercanasSource, InstalacionesSource, SourceError, StorageError, SystemClock,
};
use opentrains_sofse::SofseClient;
use opentrains_storage::JsonStore;
use serde::{Deserialize, Serialize};
use tauri::{Manager, State};

struct AppState {
    catalogo: Catalogo<Arc<SofseClient>, Arc<JsonStore>, SystemClock>,
    arribos: ConsultaArribos<Arc<SofseClient>>,
    /// Instalaciones y Ramales de una Estación: sin lógica de dominio, el caso de uso
    /// sería un pasamanos.
    sofse: Arc<SofseClient>,
    favoritos: Favoritos<Arc<JsonStore>>,
    noticias: Arc<avisos::NoticiasApp>,
    cercania: Cercania<ubicacion::UbicacionSistema, Arc<SofseClient>>,
    seguimientos: Arc<avisos::SeguimientosApp<tauri::Wry>>,
    /// Nombre y Arribos de la Estación abierta según el último refresco, para la bandeja.
    estacion_abierta: Mutex<Option<(String, Vec<Arribo>)>>,
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
    /// Si hubo un Cambio de andén, el andén que tenía antes.
    anden_anterior: Option<String>,
    /// Noticia del Servicio.
    noticia: Option<String>,
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
            anden_anterior: a.anden_anterior,
            noticia: a.servicio.noticia,
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
    handle: tauri::AppHandle,
    app: State<'_, AppState>,
    favorito: Favorito,
) -> Result<Vec<ArriboVista>, AppError> {
    let arribos = app.arribos.arribos(&favorito).await?;
    *app.estacion_abierta.lock().unwrap() = Some((favorito.estacion_nombre, arribos.clone()));
    refrescar_bandeja(&handle);
    Ok(arribos.into_iter().map(ArriboVista::from).collect())
}

#[derive(Serialize)]
struct NoticiaVista {
    id: u64,
    texto: String,
    /// Se muestra entera y no se puede descartar.
    destacada: bool,
}

impl From<Noticia> for NoticiaVista {
    fn from(n: Noticia) -> Self {
        Self {
            destacada: n.destacada(),
            id: n.id,
            texto: n.texto,
        }
    }
}

/// Noticias de la Estación abierta: si el Favorito está acotado a un Ramal, las de ese
/// Ramal; si no, las de todos los que pasan por la Estación.
#[tauri::command]
async fn noticias(
    app: State<'_, AppState>,
    favorito: Favorito,
) -> Result<Vec<NoticiaVista>, AppError> {
    let noticias = app
        .noticias
        .de_estacion(favorito.estacion, &favorito.ramales_de_noticias())
        .await;
    Ok(noticias.into_iter().map(NoticiaVista::from).collect())
}

#[tauri::command]
fn descartar_noticia(app: State<'_, AppState>, id: u64) -> Result<Preferencias, AppError> {
    Ok(app.noticias.descartar(id)?)
}

#[tauri::command]
async fn instalaciones(
    app: State<'_, AppState>,
    estacion: EstacionId,
) -> Result<Option<Instalaciones>, AppError> {
    Ok(app.sofse.instalaciones(estacion).await?)
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
async fn abrir(app: State<'_, AppState>, mut favorito: Favorito) -> Result<Preferencias, AppError> {
    // Favoritos de antes de las distancias o de las Noticias: las coordenadas y los
    // Ramales de la Estación salen del Ramal (en caché).
    let incompleto = favorito.coordenadas.is_none() || favorito.ramales.is_empty();
    if let (true, Some(ramal)) = (incompleto, &favorito.ramal) {
        if let Ok(estaciones) = app.catalogo.estaciones(ramal.id).await {
            if let Some(estacion) = estaciones.iter().find(|e| e.id == favorito.estacion) {
                favorito.coordenadas.get_or_insert(estacion.coordenadas());
                if favorito.ramales.is_empty() {
                    favorito.ramales = estacion.ramales.clone();
                }
            }
        }
    }
    // Sin Ramal para buscarla en el catálogo: la Estación aparece entre las cercanas a
    // sus propias coordenadas, y esa respuesta trae sus Ramales.
    if let (true, Some(coordenadas)) = (favorito.ramales.is_empty(), favorito.coordenadas) {
        if let Ok(cercanas) = app.sofse.estaciones_cercanas(coordenadas).await {
            if let Some(estacion) = cercanas.iter().find(|e| e.id == favorito.estacion) {
                favorito.ramales = estacion.ramales.clone();
            }
        }
    }
    Ok(app.favoritos.abrir(favorito)?)
}

/// Pedido de Seguimiento desde la UI: el Servicio de un Arribo en la Estación abierta.
#[derive(Deserialize)]
struct PedidoSeguimiento {
    estacion: EstacionId,
    estacion_nombre: String,
    numero: u32,
    destino: String,
    anticipacion_minutos: i64,
}

#[tauri::command]
fn seguir(
    handle: tauri::AppHandle,
    app: State<'_, AppState>,
    pedido: PedidoSeguimiento,
) -> Vec<Seguimiento> {
    // Recordarla es una comodidad: si falla el disco, el Seguimiento sigue igual.
    if let Ok(minutos) = u32::try_from(pedido.anticipacion_minutos) {
        let _ = app
            .favoritos
            .recordar_anticipacion(pedido.estacion, minutos);
    }
    app.seguimientos.seguir(Seguimiento::nuevo(
        pedido.estacion,
        pedido.estacion_nombre,
        pedido.numero,
        pedido.destino,
        pedido.anticipacion_minutos,
    ));
    // Revisión inmediata: si el tren ya está dentro de la anticipación, el Aviso llega ya.
    let seguimientos = app.seguimientos.clone();
    tauri::async_runtime::spawn(async move {
        // Android 13+ pide el permiso de notificaciones en tiempo de ejecución; se pide
        // con el primer Seguimiento, que es cuando hace falta.
        #[cfg(mobile)]
        {
            let handle = handle.clone();
            let _ =
                tauri::async_runtime::spawn_blocking(move || avisos::pedir_permiso(&handle)).await;
        }
        let _ = seguimientos.revisar().await;
        refrescar_bandeja(&handle);
    });
    app.seguimientos.activos()
}

#[tauri::command]
fn dejar_de_seguir(
    handle: tauri::AppHandle,
    app: State<'_, AppState>,
    estacion: EstacionId,
    numero: u32,
) -> Vec<Seguimiento> {
    app.seguimientos.dejar_de_seguir(estacion, numero);
    refrescar_bandeja(&handle);
    app.seguimientos.activos()
}

#[tauri::command]
fn seguimientos(app: State<'_, AppState>) -> Vec<Seguimiento> {
    app.seguimientos.activos()
}

#[tauri::command]
fn ocultar_aviso_ubicacion(app: State<'_, AppState>) -> Result<Preferencias, AppError> {
    Ok(app.favoritos.ocultar_aviso_ubicacion()?)
}

/// Abre la configuración de ubicación del sistema (en Windows, Privacidad > Ubicación).
#[tauri::command]
fn abrir_ajustes_ubicacion() {
    #[cfg(windows)]
    let _ = std::process::Command::new("explorer.exe")
        .arg("ms-settings:privacy-location")
        .spawn();
}

/// Consulta la Ubicación; `false` si no hay ninguna conocida.
#[tauri::command]
async fn ubicar(app: State<'_, AppState>) -> Result<bool, AppError> {
    Ok(app.cercania.actualizar().await.is_some())
}

/// Distancias ya formateadas ("a 850 m") desde la última Ubicación, en el mismo orden.
#[tauri::command]
fn distancias(app: State<'_, AppState>, destinos: Vec<Coordenadas>) -> Option<Vec<String>> {
    app.cercania
        .distancias(&destinos)
        .map(|ds| ds.iter().map(ToString::to_string).collect())
}

#[derive(Serialize)]
struct EstacionCercanaVista {
    estacion: Estacion,
    distancia: String,
}

#[tauri::command]
async fn cercanas(app: State<'_, AppState>) -> Result<Option<Vec<EstacionCercanaVista>>, AppError> {
    Ok(app.cercania.cercanas().await?.map(|cercanas| {
        cercanas
            .into_iter()
            .map(|c| EstacionCercanaVista {
                distancia: c.distancia.to_string(),
                estacion: c.estacion,
            })
            .collect()
    }))
}

/// Recalcula el tren que muestra la bandeja (insignia y tooltip) sin consultar la API.
fn refrescar_bandeja(handle: &tauri::AppHandle) {
    #[cfg(desktop)]
    {
        let app = handle.state::<AppState>();
        let seguimientos = app.seguimientos.activos();
        let abierta = app.estacion_abierta.lock().unwrap().clone();
        let indicador = opentrains_core::indicador::indicador(
            &seguimientos,
            abierta.as_ref().map(
                |(nombre, arribos)| opentrains_core::indicador::EstacionAbierta { nombre, arribos },
            ),
            Utc::now(),
        );
        bandeja::mostrar_indicador(handle, indicador.as_ref());
    }
    #[cfg(not(desktop))]
    let _ = handle;
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
    #[cfg(mobile)]
    let builder = builder.plugin(tauri_plugin_geolocation::init());
    builder
        .plugin(tauri_plugin_notification::init())
        .setup(|app| {
            let sofse = Arc::new(SofseClient::default());
            let store = Arc::new(JsonStore::new(directorio_datos(app)));
            let seguimientos = Arc::new(Seguimientos::new(
                sofse.clone(),
                avisos::NotificadorSistema(app.handle().clone()),
                SystemClock,
            ));
            let noticias = Arc::new(Noticias::new(sofse.clone(), store.clone(), SystemClock));
            let handle = app.handle().clone();
            // Cada vuelta también recalcula la cuenta regresiva de la bandeja. En Android el
            // loop solo corre con la app abierta: en segundo plano el sistema lo duerme.
            avisos::vigilar(
                seguimientos.clone(),
                noticias.clone(),
                avisos::NotificadorSistema(app.handle().clone()),
                move || refrescar_bandeja(&handle),
            );
            app.manage(AppState {
                catalogo: Catalogo::new(sofse.clone(), store.clone(), SystemClock),
                arribos: ConsultaArribos::new(sofse.clone()),
                cercania: Cercania::new(
                    ubicacion::UbicacionSistema::new(app.handle()),
                    sofse.clone(),
                ),
                noticias,
                sofse,
                favoritos: Favoritos::new(store),
                seguimientos,
                estacion_abierta: Mutex::default(),
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
            noticias,
            descartar_noticia,
            preferencias,
            agregar_favorito,
            quitar_favorito,
            abrir,
            seguir,
            dejar_de_seguir,
            seguimientos,
            ubicar,
            distancias,
            cercanas,
            ocultar_aviso_ubicacion,
            abrir_ajustes_ubicacion
        ])
        .run(tauri::generate_context!())
        .expect("error al iniciar OpenTrains");
}

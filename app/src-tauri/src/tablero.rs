//! Tablero de Estación: los Arribos en una ventana aparte, pensada para un segundo
//! monitor. A diferencia del panel, es una ventana normal: no se oculta al perder el foco.

use std::sync::Mutex;

use opentrains_core::dominio::{Favorito, Tablero, VentanaTablero};
use tauri::{
    AppHandle, Emitter, Manager, PhysicalPosition, PhysicalSize, Runtime, WebviewUrl,
    WebviewWindow, WebviewWindowBuilder, WindowEvent,
};

use crate::{AppError, AppState};

pub const ETIQUETA: &str = "tablero";
/// Evento que avisa al tablero abierto que cambió su Estación.
const EVENTO_CAMBIO: &str = "tablero-cambiado";

#[derive(Default)]
pub struct EstadoTablero {
    /// Última posición y tamaño fuera de pantalla completa. Se guarda en disco al cerrar
    /// o salir, no en cada movimiento.
    ventana: Mutex<Option<VentanaTablero>>,
    /// Mientras está en pantalla completa, Windows no apaga el monitor.
    despierta: Mutex<Option<pantalla::Despierta>>,
}

/// Abre el tablero en esa Estación; si ya está abierto, lo cambia a ella y lo trae al frente.
pub fn abrir<R: Runtime>(app: &AppHandle<R>, favorito: Favorito) -> Result<(), AppError> {
    let preferencias = app.state::<AppState>().favoritos.abrir_tablero(favorito)?;
    let tablero = preferencias.tablero.expect("recién abierto");
    match app.get_webview_window(ETIQUETA) {
        Some(ventana) => {
            let _ = ventana.emit_to(ETIQUETA, EVENTO_CAMBIO, &tablero.favorito);
            let _ = ventana.unminimize();
            let _ = ventana.set_focus();
        }
        None => crear(app, &tablero)?,
    }
    Ok(())
}

/// Desde la bandeja: la última Estación del tablero, o la abierta en el panel.
pub fn abrir_ultimo<R: Runtime>(app: &AppHandle<R>) {
    let Ok(preferencias) = app.state::<AppState>().favoritos.preferencias() else {
        return;
    };
    let favorito = preferencias
        .tablero
        .map(|t| t.favorito)
        .or(preferencias.ultimo_abierto);
    match favorito {
        Some(favorito) => {
            let _ = abrir(app, favorito);
        }
        None => crate::bandeja::mostrar(app),
    }
}

/// Al iniciar la app: si el tablero quedó abierto, vuelve igual que estaba.
pub fn restaurar<R: Runtime>(app: &AppHandle<R>) {
    let tablero = app
        .state::<AppState>()
        .favoritos
        .preferencias()
        .ok()
        .and_then(|p| p.tablero)
        .filter(|t| t.abierto);
    if let Some(tablero) = tablero {
        let _ = crear(app, &tablero);
    }
}

pub fn pantalla_completa<R: Runtime>(app: &AppHandle<R>, activa: bool) -> Result<(), AppError> {
    let Some(ventana) = app.get_webview_window(ETIQUETA) else {
        return Ok(());
    };
    // La geometría en ventana se toma antes de agrandarla.
    if activa {
        recordar_geometria(&ventana);
    }
    ventana.set_fullscreen(activa)?;
    mantener_despierta(app, activa);
    let geometria = app.state::<EstadoTablero>().ventana.lock().unwrap().clone();
    app.state::<AppState>()
        .favoritos
        .recordar_ventana_tablero(geometria, activa)?;
    Ok(())
}

/// Guarda dónde quedó el tablero; se llama al salir de la app.
pub fn guardar<R: Runtime>(app: &AppHandle<R>) {
    let Some(ventana) = app.get_webview_window(ETIQUETA) else {
        return;
    };
    let pantalla_completa = ventana.is_fullscreen().unwrap_or(false);
    let geometria = app.state::<EstadoTablero>().ventana.lock().unwrap().clone();
    let _ = app
        .state::<AppState>()
        .favoritos
        .recordar_ventana_tablero(geometria, pantalla_completa);
}

fn crear<R: Runtime>(app: &AppHandle<R>, tablero: &Tablero) -> Result<(), AppError> {
    let ventana = WebviewWindowBuilder::new(app, ETIQUETA, WebviewUrl::App("tablero".into()))
        .title(format!("{} · OpenTrains", tablero.favorito.estacion_nombre))
        .inner_size(1280.0, 720.0)
        .min_inner_size(640.0, 400.0)
        .visible(false)
        .build()?;
    // Si el monitor donde estaba ya no está conectado, abre en el principal y en ventana.
    let visible = tablero
        .ventana
        .as_ref()
        .filter(|v| entra_en_un_monitor(&ventana, v));
    if let Some(v) = visible {
        let _ = ventana.set_position(PhysicalPosition::new(v.x, v.y));
        let _ = ventana.set_size(PhysicalSize::new(v.ancho, v.alto));
        *app.state::<EstadoTablero>().ventana.lock().unwrap() = Some(v.clone());
    } else {
        let _ = ventana.center();
    }
    let completa = tablero.pantalla_completa && visible.is_some();
    if completa {
        let _ = ventana.set_fullscreen(true);
        mantener_despierta(app, true);
    }
    let _ = ventana.show();
    let manejada = ventana.clone();
    let handle = app.clone();
    ventana.on_window_event(move |evento| match evento {
        WindowEvent::Moved(_) | WindowEvent::Resized(_) => recordar_geometria(&manejada),
        WindowEvent::CloseRequested { .. } => {
            // Lo cerró el usuario: no se reabre al iniciar, pero recuerda dónde estaba.
            guardar(&handle);
            let _ = handle.state::<AppState>().favoritos.cerrar_tablero();
            mantener_despierta(&handle, false);
        }
        WindowEvent::Destroyed => mantener_despierta(&handle, false),
        _ => {}
    });
    Ok(())
}

fn recordar_geometria<R: Runtime>(ventana: &WebviewWindow<R>) {
    let fuera_de_juego = ventana.is_fullscreen().unwrap_or(false)
        || ventana.is_minimized().unwrap_or(false)
        || ventana.is_maximized().unwrap_or(false);
    if fuera_de_juego {
        return;
    }
    if let (Ok(posicion), Ok(tamano)) = (ventana.outer_position(), ventana.inner_size()) {
        *ventana.state::<EstadoTablero>().ventana.lock().unwrap() = Some(VentanaTablero {
            x: posicion.x,
            y: posicion.y,
            ancho: tamano.width,
            alto: tamano.height,
        });
    }
}

/// El centro de la ventana cae dentro de algún monitor conectado.
fn entra_en_un_monitor<R: Runtime>(ventana: &WebviewWindow<R>, v: &VentanaTablero) -> bool {
    let (cx, cy) = (v.x + v.ancho as i32 / 2, v.y + v.alto as i32 / 2);
    ventana
        .available_monitors()
        .unwrap_or_default()
        .iter()
        .any(|m| {
            let (p, s) = (m.position(), m.size());
            cx >= p.x && cx < p.x + s.width as i32 && cy >= p.y && cy < p.y + s.height as i32
        })
}

fn mantener_despierta<R: Runtime>(app: &AppHandle<R>, activa: bool) {
    let estado = app.state::<EstadoTablero>();
    *estado.despierta.lock().unwrap() = activa.then(pantalla::Despierta::nueva);
}

/// Pedirle a Windows que no apague el monitor. El pedido vale mientras viva el hilo que
/// lo hizo, así que vive en un hilo propio que se suelta al soltar `Despierta`.
mod pantalla {
    use std::sync::mpsc;

    pub struct Despierta(#[allow(dead_code)] mpsc::Sender<()>);

    impl Despierta {
        pub fn nueva() -> Self {
            let (fin, esperar) = mpsc::channel::<()>();
            std::thread::spawn(move || {
                pedir(true);
                // Termina cuando se suelta el `Sender`.
                let _ = esperar.recv();
                pedir(false);
            });
            Self(fin)
        }
    }

    #[cfg(windows)]
    fn pedir(despierta: bool) {
        use windows::Win32::System::Power::{
            SetThreadExecutionState, ES_CONTINUOUS, ES_DISPLAY_REQUIRED, ES_SYSTEM_REQUIRED,
        };
        let estado = if despierta {
            ES_CONTINUOUS | ES_DISPLAY_REQUIRED | ES_SYSTEM_REQUIRED
        } else {
            ES_CONTINUOUS
        };
        // SAFETY: solo cambia el estado de energía pedido por este hilo.
        unsafe {
            SetThreadExecutionState(estado);
        }
    }

    #[cfg(not(windows))]
    fn pedir(_: bool) {}
}

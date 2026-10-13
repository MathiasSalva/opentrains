//! Widget en la bandeja del sistema: el ícono abre un panel flotante (flyout) sobre la
//! barra de tareas, como los de Wi-Fi o volumen. La app sigue viva en segundo plano.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use tauri::menu::{CheckMenuItem, Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIcon, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Manager, PhysicalPosition, Runtime, WebviewWindow, WindowEvent};
use tauri_plugin_autostart::ManagerExt;

pub const ID: &str = "principal";
/// Argumento con el que Windows lanza la app al iniciar sesión: arranca oculta.
pub const ARG_OCULTA: &str = "--oculta";
const MARGEN: i32 = 12;

#[derive(Default)]
pub struct EstadoPanel {
    /// Momento en que el panel se ocultó por perder el foco. Al hacer clic en el ícono, el
    /// panel pierde el foco antes de que llegue el clic: sin esto, se ocultaría y volvería
    /// a abrirse en el mismo gesto.
    ultimo_ocultamiento: Mutex<Option<Instant>>,
    /// Si el panel recibió el foco desde que se mostró. Windows puede negarle el foco (por
    /// ejemplo si otra app está en primer plano): en ese caso no hay que ocultarlo al
    /// "perderlo", porque nunca lo tuvo.
    tuvo_foco: AtomicBool,
}

pub fn crear<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<TrayIcon<R>> {
    let abrir = MenuItem::with_id(app, "abrir", "Abrir OpenTrains", true, None::<&str>)?;
    let inicio = CheckMenuItem::with_id(
        app,
        "inicio",
        "Iniciar con Windows",
        true,
        app.autolaunch().is_enabled().unwrap_or(false),
        None::<&str>,
    )?;
    let salir = MenuItem::with_id(app, "salir", "Salir", true, None::<&str>)?;
    let menu = Menu::with_items(
        app,
        &[
            &abrir,
            &inicio,
            &PredefinedMenuItem::separator(app)?,
            &salir,
        ],
    )?;

    TrayIconBuilder::with_id(ID)
        .icon(app.default_window_icon().cloned().expect("ícono de la app"))
        .tooltip("OpenTrains")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(move |app, evento| match evento.id().as_ref() {
            "abrir" => mostrar(app),
            "inicio" => {
                let autostart = app.autolaunch();
                let activar = !autostart.is_enabled().unwrap_or(false);
                let _ = if activar {
                    autostart.enable()
                } else {
                    autostart.disable()
                };
                let _ = inicio.set_checked(autostart.is_enabled().unwrap_or(false));
            }
            "salir" => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|bandeja, evento| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = evento
            {
                alternar(bandeja.app_handle());
            }
        })
        .build(app)
}

fn panel<R: Runtime>(app: &AppHandle<R>) -> Option<WebviewWindow<R>> {
    app.get_webview_window("main")
}

fn alternar<R: Runtime>(app: &AppHandle<R>) {
    let Some(ventana) = panel(app) else { return };
    let recien_oculto = app
        .state::<EstadoPanel>()
        .ultimo_ocultamiento
        .lock()
        .unwrap()
        .is_some_and(|t| t.elapsed() < Duration::from_millis(300));
    if ventana.is_visible().unwrap_or(false) || recien_oculto {
        let _ = ventana.hide();
    } else {
        mostrar(app);
    }
}

/// Muestra el panel en la esquina inferior derecha del área de trabajo (sobre la barra de
/// tareas), en el monitor donde está el cursor.
pub fn mostrar<R: Runtime>(app: &AppHandle<R>) {
    let Some(ventana) = panel(app) else { return };
    if let (Ok(cursor), Ok(tamano)) = (ventana.cursor_position(), ventana.outer_size()) {
        if let Ok(Some(monitor)) = ventana.monitor_from_point(cursor.x, cursor.y) {
            let area = monitor.work_area();
            let x = area.position.x + area.size.width as i32 - tamano.width as i32 - MARGEN;
            let y = area.position.y + area.size.height as i32 - tamano.height as i32 - MARGEN;
            let _ = ventana.set_position(PhysicalPosition::new(x, y));
        }
    }
    app.state::<EstadoPanel>()
        .tuvo_foco
        .store(false, Ordering::SeqCst);
    let _ = ventana.show();
    let _ = ventana.unminimize();
    let _ = ventana.set_focus();
}

/// Comportamiento de panel: se oculta al perder el foco, y cerrarlo lo oculta en vez de
/// salir de la app.
pub fn comportamiento_panel<R: Runtime>(ventana: &WebviewWindow<R>) {
    let app = ventana.app_handle().clone();
    let manejada = ventana.clone();
    ventana.on_window_event(move |evento| match evento {
        WindowEvent::Focused(true) => {
            app.state::<EstadoPanel>()
                .tuvo_foco
                .store(true, Ordering::SeqCst);
        }
        WindowEvent::Focused(false) => {
            let estado = app.state::<EstadoPanel>();
            if estado.tuvo_foco.swap(false, Ordering::SeqCst) {
                let _ = manejada.hide();
                *estado.ultimo_ocultamiento.lock().unwrap() = Some(Instant::now());
            }
        }
        WindowEvent::CloseRequested { api, .. } => {
            api.prevent_close();
            let _ = manejada.hide();
        }
        _ => {}
    });
}

/// Actualiza el tooltip del ícono con el próximo tren (lo arma el frontend).
pub fn tooltip<R: Runtime>(app: &AppHandle<R>, texto: &str) {
    if let Some(bandeja) = app.tray_by_id(ID) {
        let _ = bandeja.set_tooltip(Some(texto));
    }
}

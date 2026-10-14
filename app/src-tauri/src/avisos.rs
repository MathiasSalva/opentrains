//! Adaptador de salida de los Avisos: notificaciones del sistema (en Windows, toasts).

use std::sync::Arc;
use std::time::Duration;

use opentrains_core::casos_de_uso::Seguimientos;
use opentrains_core::dominio::{Aviso, Seguimiento};
use opentrains_core::puertos::{Notificador, SystemClock};
use opentrains_sofse::SofseClient;
use tauri::{AppHandle, Runtime};
use tauri_plugin_notification::NotificationExt;

pub type SeguimientosApp<R> = Seguimientos<Arc<SofseClient>, NotificadorSistema<R>, SystemClock>;

/// Cada cuánto se revisan los Servicios seguidos, con o sin el panel abierto.
const INTERVALO: Duration = Duration::from_secs(30);

pub struct NotificadorSistema<R: Runtime>(pub AppHandle<R>);

impl<R: Runtime> Notificador for NotificadorSistema<R> {
    fn notificar(&self, seguimiento: &Seguimiento, aviso: &Aviso) {
        let (titulo, cuerpo) = textos(seguimiento, aviso);
        let resultado = self
            .0
            .notification()
            .builder()
            .title(&titulo)
            .body(&cuerpo)
            .show();
        match resultado {
            Ok(()) => eprintln!("[avisos] {titulo}: {cuerpo}"),
            Err(error) => eprintln!("[avisos] no se pudo notificar ({titulo}): {error}"),
        }
    }
}

/// Bloqueante: pide el permiso de notificaciones si todavía no se decidió. Si la persona
/// lo rechaza, los Seguimientos siguen igual pero los Avisos no se ven.
#[cfg(mobile)]
pub fn pedir_permiso<R: Runtime>(app: &AppHandle<R>) {
    use tauri::plugin::PermissionState;
    let notificaciones = app.notification();
    if matches!(
        notificaciones.permission_state(),
        Ok(PermissionState::Prompt | PermissionState::PromptWithRationale)
    ) {
        if let Err(error) = notificaciones.request_permission() {
            eprintln!("[avisos] no se pudo pedir el permiso: {error}");
        }
    }
}

/// Título y cuerpo de la notificación, p. ej. "Retiro (LSM): #3907 a Pilar" / "Sale en 9 min".
pub fn textos(seguimiento: &Seguimiento, aviso: &Aviso) -> (String, String) {
    let titulo = format!(
        "{}: #{} a {}",
        seguimiento.estacion_nombre, seguimiento.numero, seguimiento.destino
    );
    let cuerpo = match aviso {
        Aviso::Recordatorio { minutos: 0 } => "Sale en menos de 1 min.".to_owned(),
        Aviso::Recordatorio { minutos } => format!("Sale en {minutos} min."),
        Aviso::Demora { minutos } => format!("Viene con {minutos} min de demora."),
        Aviso::Cancelado => "Fue cancelado.".to_owned(),
    };
    (titulo, cuerpo)
}

/// Revisa los Seguimientos periódicamente mientras la app esté viva; `despues` corre al
/// final de cada vuelta, haya o no Seguimientos. En Android el proceso en segundo plano
/// queda dormido o lo mata el sistema, así que los Avisos llegan solo con la app abierta.
pub fn vigilar<R: Runtime>(
    seguimientos: Arc<SeguimientosApp<R>>,
    despues: impl Fn() + Send + 'static,
) {
    tauri::async_runtime::spawn(async move {
        loop {
            tokio::time::sleep(INTERVALO).await;
            // Un error de red deja los Seguimientos intactos; se reintenta en la próxima vuelta.
            let _ = seguimientos.revisar().await;
            despues();
        }
    });
}

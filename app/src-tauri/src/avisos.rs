//! Adaptador de salida de los Avisos: notificaciones del sistema (en Windows, toasts).

use std::sync::Arc;
use std::time::Duration;

use opentrains_core::casos_de_uso::{Noticias, Seguimientos};
use opentrains_core::dominio::{Aviso, Noticia, Seguimiento};
use opentrains_core::puertos::{Notificador, SystemClock};
use opentrains_sofse::SofseClient;
use opentrains_storage::JsonStore;
use tauri::{AppHandle, Runtime};
use tauri_plugin_notification::NotificationExt;

pub type SeguimientosApp<R> = Seguimientos<Arc<SofseClient>, NotificadorSistema<R>, SystemClock>;
pub type NoticiasApp = Noticias<Arc<SofseClient>, Arc<JsonStore>, SystemClock>;

/// Cada cuánto se revisan los Servicios seguidos, con o sin el panel abierto.
const INTERVALO: Duration = Duration::from_secs(30);

pub struct NotificadorSistema<R: Runtime>(pub AppHandle<R>);

impl<R: Runtime> Notificador for NotificadorSistema<R> {
    fn notificar(&self, seguimiento: &Seguimiento, aviso: &Aviso) {
        let (titulo, cuerpo) = textos(seguimiento, aviso);
        self.mostrar(&titulo, &cuerpo);
    }

    fn notificar_noticia(&self, lugar: &str, noticia: &Noticia) {
        self.mostrar(&format!("Noticia del operador: {lugar}"), &noticia.texto);
    }
}

impl<R: Runtime> NotificadorSistema<R> {
    fn mostrar(&self, titulo: &str, cuerpo: &str) {
        let resultado = self
            .0
            .notification()
            .builder()
            .title(titulo)
            .body(cuerpo)
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

/// Con Favoritos hacen falta las notificaciones aunque no haya Seguimientos: las
/// Noticias destacadas de sus Ramales también son Avisos. Se revisa una sola vez por
/// proceso (la UI puede recargarse y volver a pedir las Preferencias); sin Favoritos no
/// se gasta la vez y se vuelve a revisar al agregar el primero. No bloquea: los comandos
/// sincrónicos corren en el hilo principal, y el pedido espera la respuesta del diálogo.
#[cfg(mobile)]
pub fn pedir_permiso_por_favoritos<R: Runtime>(
    app: &AppHandle<R>,
    preferencias: &opentrains_core::dominio::Preferencias,
) {
    use std::sync::atomic::{AtomicBool, Ordering};
    static REVISADO: AtomicBool = AtomicBool::new(false);
    if preferencias.favoritos.is_empty() || REVISADO.swap(true, Ordering::SeqCst) {
        return;
    }
    let app = app.clone();
    tauri::async_runtime::spawn_blocking(move || pedir_permiso(&app));
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
        Aviso::CambioDeAnden { anden, anterior } => {
            format!("Cambió al andén {anden} (antes andén {anterior}).")
        }
    };
    (titulo, cuerpo)
}

/// Revisa los Seguimientos y las Noticias destacadas periódicamente mientras la app esté
/// viva; `despues` corre al final de cada vuelta, haya o no Seguimientos. En Android el proceso en segundo plano
/// queda dormido o lo mata el sistema, así que los Avisos llegan solo con la app abierta.
pub fn vigilar<R: Runtime>(
    seguimientos: Arc<SeguimientosApp<R>>,
    noticias: Arc<NoticiasApp>,
    notificador: NotificadorSistema<R>,
    despues: impl Fn() + Send + 'static,
) {
    tauri::async_runtime::spawn(async move {
        loop {
            tokio::time::sleep(INTERVALO).await;
            // Un error de red deja los Seguimientos intactos; se reintenta en la próxima vuelta.
            let _ = seguimientos.revisar().await;
            // Después de revisar, para que los Seguimientos ya conozcan el Ramal de su Servicio.
            // Las Noticias se piden a lo sumo cada `REFRESCO_MINUTOS`.
            noticias.avisar(&seguimientos.activos(), &notificador).await;
            despues();
        }
    });
}

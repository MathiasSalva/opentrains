//! Adaptador de salida de la Ubicación. En Windows usa el servicio de ubicación del
//! sistema (WinRT `Geolocator`); en Android e iOS, `tauri-plugin-geolocation`.

use opentrains_core::dominio::Coordenadas;
use opentrains_core::puertos::UbicacionSource;

pub struct UbicacionSistema {
    #[cfg(mobile)]
    app: tauri::AppHandle,
}

impl UbicacionSistema {
    pub fn new(app: &tauri::AppHandle) -> Self {
        #[cfg(mobile)]
        return Self { app: app.clone() };
        #[cfg(not(mobile))]
        {
            let _ = app;
            Self {}
        }
    }
}

impl UbicacionSource for UbicacionSistema {
    async fn ubicacion(&self) -> Option<Coordenadas> {
        #[cfg(mobile)]
        {
            let app = self.app.clone();
            // Las llamadas al plugin bloquean hasta que responde el lado nativo.
            match tauri::async_runtime::spawn_blocking(move || movil::consultar(&app)).await {
                Ok(Ok(coordenadas)) => Some(coordenadas),
                Ok(Err(error)) => {
                    eprintln!("[ubicacion] sin ubicación: {error}");
                    None
                }
                Err(_) => None,
            }
        }
        #[cfg(windows)]
        {
            let resultado = tauri::async_runtime::spawn_blocking(windows::consultar).await;
            match resultado {
                Ok(Ok(coordenadas)) => Some(coordenadas),
                Ok(Err(error)) => {
                    eprintln!("[ubicacion] sin ubicación: {error}");
                    None
                }
                Err(_) => None,
            }
        }
        #[cfg(not(any(windows, mobile)))]
        None
    }
}

#[cfg(mobile)]
mod movil {
    use std::sync::atomic::{AtomicBool, Ordering};

    use opentrains_core::dominio::Coordenadas;
    use tauri::plugin::PermissionState;
    use tauri_plugin_geolocation::{GeolocationExt, PermissionType, PositionOptions};

    /// El permiso se pide una sola vez por proceso: la Ubicación se consulta en cada
    /// refresco y no hay que insistir con el diálogo si la persona lo rechazó.
    static PERMISO_PEDIDO: AtomicBool = AtomicBool::new(false);

    fn concedido(estado: &tauri_plugin_geolocation::PermissionStatus) -> bool {
        estado.location == PermissionState::Granted
            || estado.coarse_location == PermissionState::Granted
    }

    /// Bloqueante. Falla si la ubicación del teléfono está apagada o sin permiso.
    pub fn consultar(app: &tauri::AppHandle) -> Result<Coordenadas, String> {
        let geo = app.geolocation();
        let mut estado = geo.check_permissions().map_err(|e| e.to_string())?;
        if !concedido(&estado) && !PERMISO_PEDIDO.swap(true, Ordering::SeqCst) {
            // Con "Ubicación aproximada" alcanza para ordenar Estaciones por distancia.
            estado = geo
                .request_permissions(Some(vec![PermissionType::Location]))
                .map_err(|e| e.to_string())?;
        }
        if !concedido(&estado) {
            return Err("permiso de ubicación denegado".into());
        }
        let posicion = geo
            .get_current_position(Some(PositionOptions {
                enable_high_accuracy: false,
                timeout: 10_000,
                // Una posición de hasta un minuto atrás sirve: ahorra batería.
                maximum_age: 60_000,
            }))
            .map_err(|e| e.to_string())?;
        Ok(Coordenadas {
            latitud: posicion.coords.latitude,
            longitud: posicion.coords.longitude,
        })
    }
}

#[cfg(windows)]
mod windows {
    use std::sync::Once;

    use ::windows::core::Result;
    use ::windows::Devices::Geolocation::{GeolocationAccessStatus, Geolocator};
    use ::windows::Foundation::TimeSpan;
    use ::windows::Win32::System::Com::CoIncrementMTAUsage;
    use opentrains_core::dominio::Coordenadas;

    /// Una posición de hasta un minuto atrás sirve: ahorra el Wi-Fi.
    const ANTIGUEDAD_MAXIMA: TimeSpan = TimeSpan {
        Duration: 60 * 10_000_000,
    };
    const ESPERA_MAXIMA: TimeSpan = TimeSpan {
        Duration: 10 * 10_000_000,
    };

    /// Bloqueante: corre en un hilo aparte. Falla si la ubicación de Windows está
    /// desactivada o denegada para apps de escritorio.
    pub fn consultar() -> Result<Coordenadas> {
        // WinRT necesita un apartamento COM; el MTA implícito alcanza para estos hilos.
        static MTA: Once = Once::new();
        MTA.call_once(|| {
            let _ = unsafe { CoIncrementMTAUsage() };
        });
        let acceso = Geolocator::RequestAccessAsync()?.join()?;
        if acceso != GeolocationAccessStatus::Allowed {
            return Err(::windows::core::Error::new(
                ::windows::core::HRESULT(0x8007_0005u32 as i32),
                "ubicación de Windows desactivada o denegada",
            ));
        }
        let posicion = Geolocator::new()?
            .GetGeopositionAsyncWithAgeAndTimeout(ANTIGUEDAD_MAXIMA, ESPERA_MAXIMA)?
            .join()?
            .Coordinate()?
            .Point()?
            .Position()?;
        Ok(Coordenadas {
            latitud: posicion.Latitude,
            longitud: posicion.Longitude,
        })
    }
}

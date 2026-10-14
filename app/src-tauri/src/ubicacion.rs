//! Adaptador de salida de la Ubicación. En Windows usa el servicio de ubicación del
//! sistema (WinRT `Geolocator`); en Android se reemplazará por `tauri-plugin-geolocation`.

use opentrains_core::dominio::Coordenadas;
use opentrains_core::puertos::UbicacionSource;

pub struct UbicacionSistema;

impl UbicacionSource for UbicacionSistema {
    async fn ubicacion(&self) -> Option<Coordenadas> {
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
        #[cfg(not(windows))]
        None
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

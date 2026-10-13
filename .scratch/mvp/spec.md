# MVP de OpenTrains

Status: ready-for-agent

Ver `CONTEXT.md` para el glosario y `docs/adr/` para las decisiones de arquitectura.

## Alcance

- Elegir Línea → Ramal → Estación y ver sus Arribos:
  - cuenta regresiva que baja localmente cada segundo;
  - hora programada;
  - demora, solo si es ≥ 1 min;
  - andén, Destino y Estado del Servicio con su color.
- Los Arribos se ordenan por cuenta regresiva.
- Los Servicios ocultos no se muestran. Los cancelados se muestran en rojo y tachados.
- Los Estados desconocidos se muestran con el texto de la API, en gris.
- Favoritos: una Estación, con un Ramal opcional, en un archivo local.
- Se recuerda el último Favorito abierto.
- Instalaciones de la Estación: ascensor, rampa, baños, boletería, wifi, SUBE y conexiones.
- Las horas se muestran siempre en hora argentina (UTC-3).
- Refresco:
  - cada 45 s, con la vista de Arribos visible;
  - se pausa al minimizar;
  - hay un botón de refresco manual.
- Si un refresco falla, se mantienen los datos anteriores con el aviso "Desactualizado hace X".
- Un 401 persistente muestra "La fuente de datos cambió".
- El catálogo (Líneas, Ramales, Estaciones) se cachea en disco durante 7 días.

## Fuera de alcance

Mapa, estaciones cercanas, notificaciones, cuentas de usuario y tray.

## Arquitectura

- `crates/core`: dominio, puertos y casos de uso.
- `crates/sofse`: adaptador de la API.
- `crates/storage`: archivos JSON en `%APPDATA%\OpenTrains`.
- `app/`: Tauri 2 con Svelte + TypeScript.

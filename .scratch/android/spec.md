# Puerto a Android

Status: needs-triage

Ver `CONTEXT.md` (glosario) y `docs/adr/` (0001 shell Tauri 2, 0002 API de SOFSE detrás de un puerto, 0003 widget de bandeja). Esta auditoría se hizo el 2026-10-13 sobre `master`, con cambios en curso (Avisos/Seguimientos y `tauri-plugin-notification`).

## Resumen

El núcleo hexagonal ya está listo para Android. `crates/core`, `crates/sofse` y `crates/storage` no usan APIs de escritorio, y el TLS va con `rustls` + `ring` + `webpki-roots`, sin OpenSSL ni el almacén de certificados del sistema.

El shell de Tauri está casi preparado:

- `bandeja` está detrás de `cfg(desktop)`.
- `autostart` y `single-instance` son dependencias solo de escritorio.
- `run()` lleva `#[cfg_attr(mobile, tauri::mobile_entry_point)]` (`app/src-tauri/src/lib.rs:157`).

Lo que falta:

- La máquina: no hay SDK ni NDK de Android.
- La configuración de ventana, que hoy es la del panel flotante.
- Adaptar el frontend, que está pensado para mouse y Escape.
- Decidir qué pasa con los Seguimientos cuando la app queda en segundo plano.

## Prerrequisitos faltantes (esta máquina)

| Componente | Estado | Evidencia |
|---|---|---|
| Android Studio | **Falta** | No existe `C:\Program Files\Android\Android Studio` y no hay instalación en `%LOCALAPPDATA%\Programs` |
| Android SDK | **Falta** | `ANDROID_HOME` y `ANDROID_SDK_ROOT` están vacías (en la sesión y en el registro de usuario y de máquina). No existe `%LOCALAPPDATA%\Android\Sdk` y `adb` no está en el PATH |
| NDK (side by side) | **Falta** | `NDK_HOME` está vacía y no hay `Sdk\ndk` |
| JDK | OK | `JAVA_HOME=C:\Program Files\Eclipse Adoptium\jdk-17.0.20.101-hotspot`. `java -version` da Temurin 17.0.20.1, que sirve para Gradle. La guía sugiere el JBR de Android Studio |
| Targets de Rust | **Faltan** | `rustup target list --installed` muestra solo `x86_64-pc-windows-msvc` (rustc 1.99.0) |
| Node | OK | v26.11.1, con `@tauri-apps/cli` ^2 en `app/package.json` |

Pasos oficiales (https://v2.tauri.app/start/prerequisites/, sección Android):

1. Instalar Android Studio. En el SDK Manager, instalar Android SDK Platform, Platform-Tools, NDK (Side by side), Build-Tools y Command-line Tools.
2. Definir las variables de usuario (PowerShell):
   - `[System.Environment]::SetEnvironmentVariable("ANDROID_HOME", "$env:LocalAppData\Android\Sdk", "User")`
   - `$VERSION = Get-ChildItem -Name "$env:LocalAppData\Android\Sdk\ndk" | Select-Object -Last 1`, y después `[System.Environment]::SetEnvironmentVariable("NDK_HOME", "$env:LocalAppData\Android\Sdk\ndk\$VERSION", "User")`
   - Opcional: `JAVA_HOME` = `C:\Program Files\Android\Android Studio\jbr`. El Temurin 17 actual también sirve.
3. `rustup target add aarch64-linux-android armv7-linux-androideabi i686-linux-android x86_64-linux-android`
4. Reiniciar la terminal y el IDE. Crear un emulador (AVD) x86_64 o conectar un teléfono con depuración USB.

## Bloqueos de compilación

No se corrió `cargo check --target aarch64-linux-android` porque el target no está instalado. Este análisis sale de leer el código y `Cargo.lock`.

- **`tauri` con `features = ["tray-icon"]`** (`app/src-tauri/Cargo.toml`): no bloquea. En `tauri-2.12.3/Cargo.toml:356`, `tray-icon` es una dependencia opcional y solo para Linux, BSD, Windows y macOS, así que en Android la feature no hace nada. `tauri::tray` y `tauri::menu` solo se usan en `bandeja.rs`, que está detrás de `#[cfg(desktop)]` (`lib.rs:4-5`).
- **`tauri-plugin-autostart` y `tauri-plugin-single-instance`**: ya están bajo `[target.'cfg(not(any(target_os = "android", target_os = "ios")))'.dependencies]`, y se usan dentro de `#[cfg(desktop)]` (`lib.rs:160-172`). Están bien.
- **`std::env::args()` y `bandeja::ARG_OCULTA`** (`lib.rs:190`): están dentro del bloque `#[cfg(desktop)]`. Están bien.
- **`actualizar_bandeja`** (`lib.rs:138-144`): ya descarta sus argumentos con `#[cfg(not(desktop))]`. No hace daño, pero conviene que el frontend no lo llame en móvil.
- **`directorio_datos`** (`lib.rs:146-155`): la rama de `APPDATA` es `#[cfg(windows)]`, así que en Android usa `app.path().app_data_dir()`. Es correcto.
- **`reqwest` 0.12 con `rustls-tls`** (`crates/sofse/Cargo.toml`): trae `ring 0.17.14`, que compila C y ASM con el clang del NDK. `tauri android` configura `CC` y el linker. Un `cargo check` a mano sin `CC_aarch64_linux_android` puede fallar en `ring`. Las raíces de confianza vienen de `webpki-roots`, no del almacén de Android. El riesgo es bajo.
- **`avisos.rs` (en curso)**: `vigilar` lanza `tauri::async_runtime::spawn` con un loop de 30 s (`avisos.rs:49-56`). Compila en Android. El problema es de ciclo de vida (ver Segundo plano).
- **Riesgo real**: los cambios del otro agente en `lib.rs` (registro del plugin y comandos de Seguimientos) tienen que respetar el mismo gating. Cualquier uso nuevo de `bandeja::*`, `WindowEvent::Focused` o APIs de ventana fuera de `cfg(desktop)` rompe Android.

## Cambios de frontend

Todo lo que sigue está en `app/src/routes/+page.svelte`, salvo donde se indica otro archivo.

- **`getCurrentWindow()`** (`:3`, `:192`): en móvil hay una sola webview. El objeto existe, pero ocultar o enfocar no tiene sentido.
- **`onFocusChanged`** (`:199`): hoy decide el refresco al volver al panel. Hay que reemplazarlo por `document.visibilitychange` (`visibilityState === "visible"`), que también funciona en escritorio. Así queda una sola ruta, independiente de la plataforma.
- **Escape → `ventana.hide()`** (`:200-201`): en Android no hay Escape, y el "atrás" del sistema ya minimiza la Activity. Hay que registrarlo solo en escritorio, con `@tauri-apps/plugin-os` `platform()` o con una bandera que exponga un comando.
- **Refresco cada 45 s** (`:193`): el `setInterval` corre aunque el panel esté oculto, para alimentar el tooltip de la bandeja. En móvil hay que pausarlo con `visibilityState === "hidden"`, porque ahí no hay bandeja y ahorra batería y datos. También hay que omitir `actualizar_bandeja`.
- **Layout**: el CSS es fluido (`main { padding: 14px … }`, sin ancho fijo). El 400×640 solo sale de `tauri.conf.json`, así que en pantalla completa no hacen falta cambios estructurales.
- **Safe areas**: agregar `viewport-fit=cover` en `app/src/app.html:6` y `padding: env(safe-area-inset-*)` en `main`/`header`. Lo piden la barra de estado, el gesto inferior y el modo edge-to-edge de Android 15+.
- **Áreas táctiles**: `button.icono` mide 36×36 px (`:579-580`) y `.chip` tiene unos 28 px de alto (`padding: 5px 11px`). Hay que llevarlos a 44 px o más (48 dp según Material) dentro de `@media (pointer: coarse)`, para que el escritorio no cambie.
- **Hover**: `.chip:hover`, `button.icono:hover` y `.mas:hover` (`GrupoRamal.svelte:83`) son solo cosméticos; ninguna función depende del hover. Conviene envolverlos en `@media (hover: hover)` para evitar el hover pegado. Los `title=` (`:295`, `:304`) no se ven en táctil: sumar `aria-label`.
- **Permiso de notificaciones**: en Android 13+, `POST_NOTIFICATIONS` se pide en tiempo de ejecución. El plugin ya lo declara (`tauri-plugin-notification-2.6.0/android/src/main/AndroidManifest.xml`). Hay que usar `isPermissionGranted()` / `requestPermission()` de `@tauri-apps/plugin-notification` al crear el primer Seguimiento, no al arrancar. La capability necesita `notification:default`.

## Configuración

- **Ventana actual**: `tauri.conf.json` > `app.windows[0]` es un flyout: `width 400`, `height 640`, `resizable false`, `decorations false`, `shadow true`, `skipTaskbar true`, `alwaysOnTop true` y `visible false`. `visible false` es crítico, porque en escritorio la ventana la muestra `bandeja::mostrar`, que en Android no existe.
- **Archivos por plataforma**: Tauri lee `tauri.android.conf.json`, `tauri.windows.conf.json`, etc., y los combina con el principal según JSON Merge Patch (RFC 7396). Los arrays, incluido `app.windows`, se reemplazan enteros (https://v2.tauri.app/develop/configuration-files/). Hay dos opciones:
  - **Opción A (recomendada)**: mover la ventana flyout a `tauri.windows.conf.json` y dejar en el archivo base una ventana neutra (`label: "main"`, `title`, visible). Así el valor por defecto sirve para móvil.
  - **Opción B**: dejar el archivo base como está y agregar un `tauri.android.conf.json` con `{"app":{"windows":[{"label":"main","title":"OpenTrains"}]}}`. Funciona porque el array se reemplaza y los campos que faltan vuelven a sus valores por defecto.
- **Bundle**: `bundle.targets: ["nsis"]` no afecta a `tauri android build`. Para la versión del APK hay que revisar `bundle.android` (`minSdkVersion`, `versionCode`).
- **Capabilities**: `capabilities/default.json` usa el `$schema` `desktop-schema.json` y `core:window:allow-hide`. Conviene separarlo en `desktop.json` (`"platforms": ["windows","macOS","linux"]`, con allow-hide) y `mobile.json` (`"platforms": ["android","iOS"]`), las dos con `notification:default`.
- **Íconos**: confirmado. `app/src-tauri/icons/android/` ya tiene `mipmap-{m,h,xh,xxh,xxxh}dpi`, `mipmap-anydpi-v26` (adaptativo) y `values`. Todavía no existe `app/src-tauri/gen/android`, así que falta correr `tauri android init`.
- **Dev en teléfono físico**: `app/vite.config.js` ya respeta `TAURI_DEV_HOST` (host y HMR en 1421).

## Segundo plano y avisos

`avisos::vigilar` revisa cada 30 s "mientras la app esté viva" (`avisos.rs:15-16`, `:48-56`). Cuando la app pasa a segundo plano, Android pone el proceso en caché, le restringe la red, Doze lo duerme y puede matarlo en cualquier momento. El loop de tokio no tiene ninguna garantía de seguir corriendo.

Opciones:

1. **Solo con la app abierta.** No requiere trabajo nativo; alcanza con decirlo en la UI. Es honesta, pero pierde el caso principal: que avise mientras el usuario hace otra cosa.
2. **Recordatorio programado.** `Options.schedule` (`Schedule.at` con `allowWhileIdle`, este último solo en Android) es una función solo móvil del plugin (https://v2.tauri.app/reference/javascript/notification/). Usa `AlarmManager` (`TauriNotificationManager.kt`) y reprograma después de un reinicio (`BOOT_COMPLETED` en su manifest).
   - Al crear un Seguimiento se programa el Recordatorio "sale en N min" sobre la hora estimada.
   - Con la app viva, se cancela o reprograma en cada revisión.
   - Sobrevive al cierre de la app. Demora y Cancelado, en cambio, solo se detectan con la app viva.
3. **Servicio en primer plano (FGS).** Tauri no tiene un plugin oficial, así que haría falta un plugin móvil propio en Kotlin, con notificación persistente mientras haya Seguimientos.
   - El tipo sería `specialUse`, con justificación revisada en Play Console. `location` no aplica y `shortService` dura unos 3 minutos (https://developer.android.com/develop/background-work/services/fgs/service-types).
   - Da Demoras en tiempo real, pero cuesta mucho en Kotlin, revisión de Play y batería.
4. **WorkManager.** El período mínimo de `PeriodicWorkRequest` es de 15 minutos (dato a verificar), demasiado grueso para trenes. Descartada.

**Recomendación**: para la primera versión, combinar 1 y 2. La vigilancia de 30 s queda dentro del proceso mientras viva, y el Recordatorio va como alarma del sistema. Encaja en el puerto `Notificador` como un adaptador móvil que programa en vez de mostrar. Si hace falta que las Demoras lleguen con la app cerrada, el FGS (opción 3) se decide aparte, con un ADR propio.

## Plan por pasos

1. **Toolchain**: instalar SDK, NDK y los targets. Para verificar: `rustup target list --installed` muestra los 4 targets y `$NDK_HOME` existe.
2. **Núcleo en Android**: correr `cargo check -p opentrains-core -p opentrains-sofse -p opentrains-storage --target aarch64-linux-android`, con `CC_aarch64_linux_android` apuntando al clang del NDK. Para verificar: compila sin cambios.
3. **Configuración por plataforma** (opción A, `tauri.windows.conf.json`). Para verificar: en Windows, `npm run tauri dev` mantiene el flyout, Escape y la bandeja.
4. **Capabilities**: crear `desktop.json` y `mobile.json` con `platforms`, más `notification:default`. Para verificar: el build de escritorio no da errores de permisos.
5. **`npm run tauri android init`** en `app/`. Para verificar: se genera `gen/android` con los íconos; commitearlo según la guía.
6. **Primer arranque**: `npm run tauri android dev` en el emulador. Para verificar: aparecen Líneas, Arribos y Favoritos, y la persistencia queda en `app_data_dir`.
7. **Ciclo de vida del frontend**: pasar a `visibilitychange`, pausar el refresco cuando la app no se ve, y dejar Escape y `actualizar_bandeja` solo en escritorio. Para verificar: escritorio sin cambios, y en Android se refresca al volver si los datos están viejos.
8. **Táctil y safe areas**: `viewport-fit=cover`, `env(safe-area-inset-*)`, objetivos de 44 px o más con `pointer: coarse` y hover bajo `hover: hover`. Para verificar: captura en el emulador con barra de gestos, sin nada tapado.
9. **Permiso de notificaciones** al crear el primer Seguimiento. Para verificar: en Android 13+ aparece el diálogo, y si se niega, la UI lo informa.
10. **Recordatorio programado** (opción 2): adaptador `Notificador` móvil que programa y cancela. Para verificar: crear un Seguimiento, cerrar la app desde Recientes y recibir el aviso a tiempo.
11. **APK firmado**: `npm run tauri android build` con keystore. Para verificar: se instala y abre en un teléfono físico.

## Riesgos/abiertos

- La API de SOFSE no es oficial (ADR 0002). Hay que validar que responda igual desde redes móviles y que la caché del catálogo en `JsonStore` alcance sin conexión.
- Probablemente haga falta un ADR nuevo ("Android: Recordatorios programados en vez de vigilancia continua"), porque la semántica de Aviso cambia según la plataforma.
- La hora estimada se mueve con la demora: si la app no corrió para reprogramarlo, el Recordatorio puede salir antes o después de tiempo.
- Los fabricantes con ahorro de batería agresivo (Xiaomi, Samsung) pueden retrasar las alarmas aunque tengan `allowWhileIdle`.
- Cuando se integre el trabajo en curso en `lib.rs` y `avisos.rs`, hay que revisar el gating `cfg(desktop)`.
- Queda sin verificar: el `cargo check` real para Android, `ring` fuera de `tauri android`, el mínimo exacto de WorkManager y los límites de FGS en Android 15.

## Comments

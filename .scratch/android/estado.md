# Estado del puerto a Android

Rama `android-port`. Sigue a la auditoría de `spec.md` (2026-10-13).

## Qué cambió

- **`gen/android`**: generado con `tauri android init` y commiteado (el `.gitignore` de la raíz ahora ignora solo `gen/schemas/`). Los íconos de `icons/android/` se copiaron a `gen/android/app/src/main/res/`.
- **Ventana**: el panel flotante (400×640, sin bordes, `skipTaskbar`, `alwaysOnTop`, `visible: false`) pasó a `tauri.windows.conf.json`. La base (`tauri.conf.json`) tiene una ventana normal y visible, que es la que usa Android.
- **Capabilities**: `default.json` se dividió en `escritorio.json` (Windows/macOS/Linux, con `core:window:allow-hide`) y `movil.json` (Android/iOS, `core:default`). Las dos llevan `notification:default`: el script que inyecta el plugin de notificaciones llama a `is_permission_granted` al cargar y sin el permiso tira un error en consola. La ubicación se consulta desde Rust, así que no necesita permisos de ACL.
- **Ubicación**: `UbicacionSistema` (`ubicacion.rs`) implementa el mismo puerto `UbicacionSource`. En móvil usa `tauri-plugin-geolocation` (dependencia solo para Android/iOS): pide el permiso de ubicación una vez por proceso y consulta una posición de hasta 1 min de antigüedad, sin alta precisión. `crates/core` no cambió.
- **Manifest**: `ACCESS_COARSE_LOCATION`, `ACCESS_FINE_LOCATION` y `android.hardware.location` opcional.
- **Notificaciones**: se pide `POST_NOTIFICATIONS` (Android 13+) si todavía no se decidió (`avisos::pedir_permiso`) al crear un Seguimiento, y también con Favoritos (sus Noticias destacadas son Avisos): al abrir la app si ya hay alguno, o al agregar el primero (`avisos::pedir_permiso_por_favoritos`, una vez por proceso).
- **Frontend**: `$lib/plataforma.ts` (`esEscritorio`). `onFocusChanged` y Escape → `hide()` quedan solo en escritorio; en todas las plataformas se refresca al volver con `visibilitychange`. En móvil el refresco de 45 s se saltea con la app oculta. `viewport-fit=cover` y padding con `env(safe-area-inset-*)`.

## Qué funciona y qué no en Android

- Funciona (compila y empaqueta): Líneas, Arribos, Favoritos, Estaciones cercanas y distancias, Seguimientos y Avisos.
- **Avisos solo con la app abierta**: `avisos::vigilar` es un loop de tokio; en segundo plano Android lo duerme o mata el proceso. Ver opciones 2 (Recordatorio programado) y 3 (servicio en primer plano) en `spec.md`.
- **Sin bandeja ni insignia** (`bandeja.rs`, `insignia.rs` son `cfg(desktop)`), ni autostart ni single-instance.
- Pendiente de `spec.md`: áreas táctiles de 44 px con `pointer: coarse`, hover bajo `hover: hover`, `aria-label` en los botones con `title`, APK firmado de release.
- Probado (2026-10-14): el APK de arm64 se instaló por depuración inalámbrica, el proceso arranca y no hay errores de consola ni panics en logcat. La pantalla del teléfono estaba apagada, así que la UI no se verificó a ojo.
- Sin probar todavía: diálogos de permisos, safe areas reales y la API de SOFSE desde redes móviles.

## Cómo recompilar e instalar

Desde `app/` (Git Bash; si la terminal no ve las variables de usuario, definirlas):

```sh
export PATH="$HOME/.cargo/bin:$PATH"
export ANDROID_HOME="$LOCALAPPDATA/Android/Sdk"
export NDK_HOME="$ANDROID_HOME/ndk/28.2.13676358"
npm install
npm run tauri android build -- --debug --target aarch64 --apk
"$ANDROID_HOME/platform-tools/adb" devices
"$ANDROID_HOME/platform-tools/adb" install -r src-tauri/gen/android/app/build/outputs/apk/universal/debug/app-universal-debug.apk
```

El build de debug con símbolos ocupa varios GB en `target/`; con poco disco, `CARGO_PROFILE_DEV_DEBUG=0` lo achica.

**Sin el Modo de desarrollador de Windows** el CLI compila la `.so` pero falla al crear el symlink en `jniLibs` ("Creation symbolic link is not allowed"). Hay dos salidas: activar el Modo de desarrollador (Configuración > Sistema > Para desarrolladores), o copiar la `.so` y correr Gradle sin las tareas de Rust:

```sh
cp ../target/aarch64-linux-android/debug/libopentrains_app_lib.so src-tauri/gen/android/app/src/main/jniLibs/arm64-v8a/
cd src-tauri/gen/android
./gradlew.bat assembleUniversalDebug -PabiList=arm64-v8a -ParchList=arm64 -PtargetList=aarch64 \
  -x rustBuildUniversalDebug -x rustBuildArm64Debug
```

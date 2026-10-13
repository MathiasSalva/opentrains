# Trenes Argentinos (app móvil) — Modelo de datos y API

> Análisis de ingeniería inversa del APK `trenes_argentinos_7_70_2.apk`
> (paquete `com.mininterior.trenesenvivo`, versión 7.70.2, versionCode 20262).
> Fecha del análisis: 2026-10-12. Herramientas: REA (inventario + JADX headless 0.7.1),
> `hermes-dec` (desensamblado del bundle Hermes), y validación con llamadas reales a la API.

## 1. Arquitectura

- **Stack:** React Native + Expo SDK 55, motor **Hermes**. La lógica de negocio está
  compilada en `assets/index.android.bundle` (bytecode Hermes v96). El `.dex` es solo
  el andamiaje de RN/Expo (`MainApplication` inicializa `ReactHost`; `MainActivity` es
  una `ReactActivity` cuyo componente raíz JS es `"main"`).
- **Backend de datos:** API REST de SOFSE en `https://api-servicios.sofse.gob.ar/v1/`.
- **Firebase:** proyecto `trenes-en-directo`. Realtime Database **solo para datos de
  usuario** (perfil, favoritos, config de notificaciones), NO para trenes. Firebase Auth
  (email/password + Google + Facebook) para cuentas (§7e). FCM + Expo push para notificaciones.
- **Mapa:** `react-native-maps` con tiles propios `https://osm.sofse.gob.ar/tiles/{z}/{x}/{y}.png`.

## 2. Autenticación de la API (credencial de cliente)

No es login de usuario: es una credencial de cliente embebida, derivada de la fecha del día.

```
dateStr  = YYYYMMDD (hora Argentina, UTC-3)
username = base64(dateStr + "sofse")
password = encodePassword(username)
```

`encodePassword(p)`:
1. `s = base64(p)`
2. reemplazos globales: `a→#t  e→#x  i→#f  o→#l  u→#7  =→#g`
3. `s = reverse(s)`
4. `s = base64(s)`
5. reemplazos globales: `a→#j  e→#p  i→#w  o→#8  u→#0  =→#v`
6. `s = reverse(s)`
7. `s = encodeURIComponent(s)`

Request de login:
```
POST /v1/auth/authorize
Content-Type: application/json
{ "username": "<username>", "password": "<password>" }
→ 200 { "token": "<~253 chars>" }   (401 {"code":401,"message":"Contraseña incorrecta"} si falla)
```

Las llamadas posteriores usan el token **crudo** (sin prefijo `Bearer`):
```
Authorization: <token>
```
El token se cachea en almacenamiento local (`tokenAuth`); `checkToken` valida expiración
(`exp*1000` vs ahora); `fetchApi` lo reusa y cachea cada respuesta.

> Nota de seguridad: el "secreto" (`"sofse"` + el esquema `encodePassword`) está en el
> cliente, así que cualquiera con el APK puede generar el token. No es un control de acceso
> real; solo ofusca. Los datos servidos son públicos y de solo lectura.

## 3. Endpoints (todos GET con `Authorization: <token>`)

| Clave en el bundle | Endpoint |
|---|---|
| `url_GetToken` | `POST /v1/auth/authorize` |
| `url_GetGerencias` | `/v1/infraestructura/gerencias?idEmpresa=1` |
| `url_GetRamales` | `/v1/infraestructura/ramales?idGerencia=<id>` |
| `url_GetEstaciones` | `/v1/infraestructura/estaciones?idRamal=<id>` (tb. `?idGerencia=` y `?nombre=`) |
| `url_GetEstacionesPorLinea` | `/v1/infraestructura/estaciones?idGerencia=<id>` |
| `url_GetArribos` | `/v1/arribos/estacion/<id_estacion>` (acepta `?ramal=` y `?tipoServicio=`) |
| `url_InfoEstacion` | `https://infoestaciones.sofse.gob.ar/public/app-estaciones/` (ascensores, boleterías) |
| `url_NearStations` | `/v1/infraestructura/estaciones-cercanas?latitud=&longitud=` (validado §7f) |
| `url_GetTipoDeServicios` | `/v1/servicios/tipos` (**404** sin params — §7f) |
| `url_ContactForm` | `POST /v1/reclamos` |

Otros: `https://trenesapi.org.ar/` (constante de config `API_URL_AUTH`, `API_TIMEOUT: 4000`;
host alternativo del API de datos — las llamadas validadas en vivo usaron
`api-servicios.sofse.gob.ar/v1`), WhatsApp de contacto `wa.me/5491137008736`.

## 4. Gerencias (líneas) — ids reales

| Línea | idGerencia |
|---|---|
| Sarmiento | 1 |
| Mitre | 5 |
| Roca | 11 |
| Belgrano Sur | 21 |
| **San Martín** | **31** |
| Tren de la Costa | 41 |
| Regionales | 501 |

Campos de cada gerencia: `id, id_empresa, nombre, estado, alerta`.
(El `PickerLinea` del bundle tiene ids 1-6 hardcodeados pero son cosméticos; los ids reales
vienen de este endpoint.)

Ejemplo — ramales de San Martín (`idGerencia=31`): `131` Retiro–Junín, `65` Retiro–Cabred.

## 5. `estaciones?idRamal=<id>` → traza

Devuelve el array ordenado de estaciones del ramal. Cada una:

```jsonc
{
  "nombre": "Retiro (LSM)",
  "id_estacion": "463",            // string
  "id_tramo": "8",
  "orden": "1",                     // secuencia para dibujar la polilínea
  "id_referencia": "188076",
  "latitud": "-34.5888579",         // string
  "longitud": "-58.3741843",
  "referencia_orden": "4",
  "radio": "",
  "andenes_habilitados": "5",
  "visibilidad": { "totem": 1, "app_mobile": 1 },
  "incluida_en_ramales": [31, 65, 131],
  "operativa_en_ramales": [31, 65, 131]
}
```

**Armado de la traza:** no hay archivo GeoJSON aparte. La traza se dibuja uniendo las
coordenadas de estas estaciones por `orden`, con `<Polyline>` de react-native-maps
(el `<Geojson>` también está disponible y lee coords en orden `[lng, lat]`). Las estaciones
se pintan como círculos con estado `circuloEstacionActivo/Intermedia/Inactivo` según el avance
del tren. Conversión de campos: la app copia `latitud→latitude`, `longitud→longitude`.

## 6. `arribos/estacion/<id_estacion>` → trenes en vivo

Respuesta: `{ "timestamp": <epoch s>, "results": [ { arribo, servicio }, ... ] }`.
La pantalla hace **polling cada 45 s** (`setInterval`, se limpia al desmontar).

```jsonc
{
  "arribo": {                          // la parada EN esta estación
    "anden":  { "id": 1003, "nombre": "3" },
    "orden":  0,
    "equipo": { "id":70, "nombre":"B953", "esElectrico":0, "gpss":["30007","30006"] },
    "nombre": "Retiro - LSM",
    "parada": true,
    "salida": {
      "programada": "2026-10-13T00:48:00.000Z",          // ISO 8601
      "enAnden": "DENTRO DEL ANDEN dist: 179",
      "ultimaHoraRegistrada": "2026-10-13T00:47:22.000Z", // último fix GPS
      "distancia": 179,                                    // metros al andén
      "velocidad": 2,                                      // km/h
      "latitud": -34.5877664,                              // POSICIÓN GPS REAL del tren
      "longitud": -58.3764406,
      "referencia": 188088
    },
    "llegada": {},                     // vacío en cabecera; si no, { programada, estimada }
    "idElemento": 463,
    "segundos": 0,                     // cuenta regresiva al arribo
    "programada": true
  },
  "servicio": {
    "numero": 3907,                    // número de tren
    "ramal": { "id":65, "nombre":"Retiro-Cabred", "tolerancia":359,
               "cabeceraFinal":   { "id":51,  "nombre":"Dr. Cabred",   "nombreCorto":"Cabred" },
               "cabeceraInicial": { "id":463, "nombre":"Retiro (LSM)", "nombreCorto":"Retiro - LSM" } },
    "gerencia": { "id":31, "nombre":"San Martín" },
    "cancelacion": null,
    "equipo": { "id":70, "nombre":"B953", "esElectrico":0, "gpss":[...] },
    "oculto": false,
    "sentido": 1,
    "tipo": { "id":1, "nombre":"Normal", "clasificacion":{ "id":1, "nombre":"Pasajeros" } },
    "location": { "lat":-34.58773, "long":-58.37631 },     // posición GPS del tren
    "id": "b393572c-…",                                     // UUID del servicio
    "desde": { "estacion":{…}, "punto":{…}, "estado": { "id":25, "nombre":"En Andén" } },
    "hasta": { "estacion": { "nombre":"Pilar", "orden":63, "idElemento":306, … } }
  }
}
```

### Seguimiento en vivo (cómo se mueve el tren en el mapa)
- Es **GPS real**, no interpolación por estación: `salida.latitud/longitud` y
  `servicio.location.{lat,long}` son la posición medida; `velocidad` (km/h),
  `distancia` (m al andén) y `ultimaHoraRegistrada` (hora del fix) la acompañan.
  `equipo.gpss` lista los ids de los equipos GPS a bordo.
- El marcador del mapa usa `animateMarkerToCoordinate` para moverse suave entre fixes.
- Refresco cada 45 s (polling REST). No hay websocket de posiciones.

### Dónde está cada dato (confirmado con tren en movimiento)
- **`arribo`** = la parada de ESTE servicio en la estación consultada: `anden`, `orden`
  (posición de la estación en el recorrido), `llegada:{programada,estimada}`,
  `salida:{programada,estimada}`, `segundos` (cuenta regresiva real), `idElemento`.
- **`servicio.location` = `{lat,long}`**: posición GPS viva del tren (lo que mueve el marcador).
- **`servicio.desde`** = segmento actual (de dónde viene el tren), e incluye el **GPS crudo**
  en `desde.salida`: `ultimaHoraRegistrada`, `distancia` (m), `velocidad` (km/h),
  `latitud`, `longitud`, `referencia`, y `real` (hora real de partida registrada).
  También `desde.estado = {id,nombre}` (estado vivo del tren).
- **`servicio.hasta`** = destino final del recorrido: tiene `estacion` y `punto`
  (ambos objetos completos con `anden`, `tramo:{id,orden}`, `salida`/`llegada` programadas,
  `idElemento`, `idPunto`, `orden` máximo).
- **`servicio.desde`** = `{ estacion, punto, estado:{id,nombre} }`; `estacion` y `punto` traen
  `tramo:{id,orden}`, `idPunto`, `salida/llegada` programadas.
- **`servicio.estaciones[]`** = **recorrido completo paso a paso**: array de todas las paradas,
  cada una con `orden`, `anden`, `tramo:{id,orden}`, `salida`/`llegada` (programada/estimada),
  `idElemento`, `idPunto`, `equipo` y **`segundos`** (cuenta regresiva a esa parada). Permite
  reconstruir el avance del tren estación por estación sin llamadas extra.
- **`servicio.location`** puede ser **`null`** cuando el servicio aún no partió
  (estado `10 "Programado"`); pasa a `{lat,long}` cuando el tren está en movimiento.
- Otros campos: `tipo.subclasificacion`, `tipo.programado`, `leyenda` (null o texto de aviso),
  `equipo.esElectrico` (0/1), `equipo.gpss` (ids de GPS a bordo).

Ej. real (Palermo, servicio #3904, formación B965): salió de Cabred (`estado {35,"Partió"}`),
a 13 km/h, llega a Palermo en `segundos:242`, destino Retiro (orden 83).
Ej. programado (Retiro, #3311, B964): `location:null`, `estado {10,"Programado"}`, sale 07:27Z
rumbo a Pilar (orden 63), arribo a Retiro en `segundos:470`.

### Estado del servicio
`servicio.desde.estado = { id, nombre }`. Códigos observados en vivo: **10 = "Programado"**
(aún no salió, `location:null`), **25 = "En Andén"**, **35 = "Partió"**. El `id` numérico se
mapea en el cliente a color + ícono:

| Color | Ícono | Sentido |
|---|---|---|
| verde `#80c395` | `icoStatusNormal` | normal |
| naranja `#f7a83a` | `icoStatusAdv` | advertencia/demora |
| naranja rojizo `#f08159` | `icoStatusAdvOrange` | advertencia mayor |
| rojo `#e45255` | `icoStatusCancelado` | cancelado |
| gris `#435a6c` | `icoStatusInfo` | información |

(La correspondencia exacta id→color no se verificó para todos los códigos; falta mapear
el abanico completo de `estado.id`.)

## 7. Firebase Realtime Database (solo usuarios)

`https://trenes-en-directo.firebaseio.com`, nodo `users/{uid}`:
```jsonc
{ "fullName": "...", "email": "...",
  "config": { "nearStation": <bool>, "geoAlerts": <bool> },  // notificaciones
  "favs": [...], "docType": "", "docNbr": "" }
```
Operaciones: `storeUserData`, `getUserData`, `updateUserData`, `deleteAccount`.
Config Firebase (en el bundle): apiKey `AIzaSy…(redactado)`,
authDomain `trenes-en-directo.firebaseapp.com`, databaseURL arriba.

### Reglas de seguridad del RTDB (validado, no invasivo)

La `apiKey` y `databaseURL` son identificadores públicos por diseño; lo único que
protege los datos de usuario son las **reglas** del Realtime Database. Verificado con
dos sondeos anónimos (sin token, solo lectura / sin dejar datos):

| Sondeo | Request | Resultado |
|---|---|---|
| Lectura anónima raíz | `GET /.json?shallow=true` | **HTTP 401** `Permission denied` |
| Escritura anónima | `PUT /_probe_readonly_test/<ts>.json` | **HTTP 401** (denegada) |

**Conclusión: las reglas están bien puestas.** El nodo `users/{uid}` **no** es legible ni
escribible sin autenticar — exponer la `apiKey` en el bundle no da acceso a datos ajenos.
El acceso real pasa por Firebase Auth (cada usuario autenticado opera sobre su propio
`uid`; ver §7 operaciones `storeUserData`/`getUserData`/...). El sondeo de escritura usó un
path aislado propio y se diseñó para auto-borrarse si entrara; al dar 401 no escribió nada.

## 7b. Notificaciones y ubicación (hallazgos funcionales)

- **Estaciones cercanas (único uso activo del GPS):** a demanda, en primer plano.
  `requestForegroundPermissionsAsync()` → posición → `GET /v1/infraestructura/estaciones-cercanas?latitud=<lat>&longitud=<lng>`.
  No corre en segundo plano.
- **`geoAlerts` / `nearStation` NO están cableados en este build.** Ambos solo se leen una
  vez, en `storeUserData`, para persistirse en el perfil de Firebase (`users/{uid}.config`).
  No existe ninguna llamada de la app a `startGeofencingAsync`, `watchPositionAsync`,
  `getCurrentPositionAsync`, `defineTask` ni `scheduleNotificationAsync`. Son toggles de
  preferencia sin lógica asociada (placeholder). Defaults: `nearStation:true`, `geoAlerts:false`.
- **Sin ubicación en segundo plano real:** el manifest **no** incluye
  `ACCESS_BACKGROUND_LOCATION`. El `LocationTaskService` y el permiso de fondo que aparecen
  provienen del config plugin de expo-location, pero la app no inicia ninguna tarea de fondo.
- **Notificaciones = push remoto (nativo), no locales:** el JS no registra push token ni
  programa notificaciones locales. Sí están declarados los servicios nativos FCM/Expo
  (`FirebaseMessagingService`, `ExpoFirebaseMessagingService`, `NotificationForwarderActivity`)
  y el módulo de auto-registro de expo-notifications (`ExpoServerRegistrationModule`,
  endpoint `exp.host/--/api/v2/push/updateDeviceToken`). Es decir: las notificaciones
  (alertas de servicio) las envía el backend por FCM/Expo y las muestra el SO/Expo sin
  intervención del JS.

## 7c. Expo Updates (OTA) — desactivado

- `AndroidManifest`: `expo.modules.updates.ENABLED = false`. **Las actualizaciones OTA
  están apagadas.** No hay `EXPO_UPDATE_URL` ni `EXPO_RUNTIME_VERSION` en el manifest.
  (`CHECK_ON_LAUNCH=ALWAYS`, `LAUNCH_WAIT_MS=0` son valores por defecto, irrelevantes con
  updates deshabilitado.)
- Igual se empaquetan la lib nativa `libexpo-updates.so`, un manifiesto embebido
  (`assets/app.manifest`: id `59afe54c-…`, 408 assets) y el cert raíz público de Expo
  (`assets/expo-root.pem`, "Expo Root Certificate", válido 2022-2042) para firma de
  manifiestos — pero no se usan en runtime porque OTA está off.
- La app corre solo el bundle JS empaquetado; **no descarga código por aire** → sin
  superficie de cadena de suministro vía OTA en este build.
- `classic-assets.eascdn.net/~assets/` aparece solo como resolver clásico de `Asset`,
  no como canal OTA activo.
- La API key de Google Maps está expuesta en el manifest
  (`com.google.android.geo.API_KEY = AIzaSy…(redactado)`); conviene que esté restringida por
  package+firma en Google Cloud (no verificable desde el APK).

## 7d. Infoestaciones (accesibilidad/servicios de estación)

- `GET https://infoestaciones.sofse.gob.ar/public/app-estaciones/{id_estacion}` — **host
  distinto y endpoint público (`/public/`), SIN token** (`axios.get(url+id)` sin headers).
- La respuesta (`.data`) alimenta la info de servicios de la estación: ascensores,
  boleterías, escaleras (strings del bundle: `infoAscensor`, `infoBoleteria`, `Ascensor`,
  `Boleterías`, `iconAscensor`, `iconBoleterias`).
- Se dispara desde la pantalla de info de estación, pasando `params.selected.id_estacion`.
- **Esquema real** (validado, `GET .../app-estaciones/463`): devuelve un **array** con un objeto:
  ```jsonc
  {
    "estacion": "Retiro - San Martín",
    "direccion": "Av. Dr. José María Ramos Mejía 1500, CABA.",
    "boleteria": "Lunes a viernes 24h, sábados y domingos de 5 a 21 h",
    "cap": "Lun. a vie. de 6 a 20 hs.",      // horario de atención/CAP
    "banos": "SI", "ascensor": "NO", "rampa": "SI", "wifi": "SI", "etb": "NO",
    "colectivos": "5 - 6 - 7 - 9 - ...",      // líneas de colectivo (string)
    "conexiones": "Subte C - Subte E - Tren Mitre - ...",
    "relevador": "Marina",                    // quién relevó los datos
    "estacion_id_unico": 463,
    "linea_id": 31,                           // = idGerencia (San Martín)
    "terminal_sube": "SI"
  }
  ```
  Campos booleanos como strings `"SI"/"NO"`. Accesibilidad: `ascensor`, `rampa`, `banos`,
  `wifi`, `etb` (escalera/terminal), `terminal_sube`.

## 7e. Login / autenticación de usuario (Firebase Auth)

Esto es **distinto** de la credencial de cliente del API de datos (§2). El login de *usuario*
usa **Firebase Authentication** (SDK JS modular **9.23.0**). Análisis estático (requiere cuentas
reales y OAuth interactivo; no se ejercitó en vivo).

**Init:** `initializeApp(firebaseConfig)` + `initializeAuth(app, { persistence:
getReactNativePersistence(AsyncStorage) })` → la sesión persiste en el dispositivo.
`getAuth().languageCode = 'es'`.

`firebaseConfig` completo (en el bundle): `projectId: trenes-en-directo`, `appId:
1:77224566249:web:ebc5a33b35df6bac973f78`, `messagingSenderId: 77224566249`,
`storageBucket: trenes-en-directo.appspot.com`, `measurementId: G-C7HGNYD701`
(+ apiKey/authDomain/databaseURL ya listados en §7). El `messagingSenderId` **= nº de proyecto
de los client IDs de Google** abajo: Firebase y Google OAuth comparten el mismo proyecto GCP.

### Tres métodos de login, todos → Firebase Auth

1. **Email / password** (nativo Firebase):
   - `loginWithEmail({username, password})` → `signInWithEmailAndPassword(auth, username, password)`
     (el campo del form se llama `username` pero es el email).
   - `registerWithEmail({email, password})` → `createUserWithEmailAndPassword`.
   - `sendPasswordReset({email})` → `sendPasswordResetEmail`.
   - Validación de form: `username` y `password` requeridos ("Este campo es obligatorio").

2. **Google** (vía `expo-auth-session`, flujo OAuth en navegador — **no** el módulo nativo
   GoogleSignin):
   - `Google.useAuthRequest({ webClientId, androidClientId, iosClientId, clientId, expoClientId })`.
   - On `response.type === 'success'` → toma `response.authentication.accessToken` →
     `loginWithGoogle(accessToken)` → `GoogleAuthProvider.credential(null, accessToken)` →
     `signInWithCredential(auth, cred)`. (Usa **accessToken**, con `idToken=null`.)
   - On `error` → reabre el modal de login.
   - Client IDs (proyecto `77224566249`):
     | rol | client ID |
     |---|---|
     | web / `clientId` | `77224566249-jce5rdc8mdks0503nsbe0j4o4u1uhvqj.apps.googleusercontent.com` |
     | android | `77224566249-r7vab0dfu5ogh5qlb3k62i5vs50p7n17.apps.googleusercontent.com` |
     | ios | `77224566249-un6kv13meqininorti5pteast2jlsa4q.apps.googleusercontent.com` |
     | expo | `77224566249-5ucidem6atvmr819nl36r2vv8ala5t0v.apps.googleusercontent.com` |

3. **Facebook** (mismo patrón `expo-auth-session`, botón activo):
   - `Facebook.useAuthRequest(...)` → on success `response.authentication.accessToken` →
     `loginWithFB(accessToken)` → `FacebookAuthProvider.credential(accessToken)` →
     `signInWithCredential(auth, cred)`.

### Post-login (`onAuthStateChanged`) — vínculo uid ↔ perfil RTDB

Al autenticar (cualquier método), `onAuthStateChanged(auth, user => …)` con `user.uid` dispara
`getUserData(uid)` (lee `users/{uid}` del RTDB, §7):
- **Perfil existe** → hidrata el estado `{ id:uid, fullName, docType, docNbr, email,
  config:{nearStation:true,geoAlerts:false}, favs }`, lo persiste local con
  `saveItem('userPersistance', JSON)`, y navega a `Menu`.
- **No existe** (1er login social) → arma el perfil desde `user.providerData[0]`
  (`{ id:uid, fullName:displayName, docType:'', docNbr:'', email,
  config:{nearStation:true,geoAlerts:false} }`) y lo **crea** con `storeUserData(...)`
  (escribe `users/{uid}`), luego navega a `Menu`.

Cada usuario autenticado opera solo sobre su propio `uid` — consistente con las reglas del RTDB
validadas en §7 (sin acceso anónimo). Logout: `signOutUser()` → `signOut(auth)`.

> Nota menor: en la rama "perfil existe" el `config` se fija a los defaults
> `{nearStation:true, geoAlerts:false}` en el estado local en vez de leer el guardado en RTDB —
> en la práctica irrelevante porque esos toggles no están cableados (§7b).

## 7f. Estaciones cercanas y tipos de servicio (validado)

### `GET /v1/infraestructura/estaciones-cercanas?latitud=<lat>&longitud=<lng>` (con token)
Devuelve un **array de estaciones ordenado por cercanía** al punto dado, **cruzando todas las
líneas** (no filtra por gerencia). Mismo esquema que §5, con **una diferencia**: aquí
`incluida_en_ramales` / `operativa_en_ramales` vienen como **string JSON** (`"[141,151,9,...]"`),
no como array real. **No** trae campo de distancia (el orden del array ya es la proximidad).

Ej. (punto junto a Retiro, `-34.5905,-58.3760`) → `Retiro (LGM)` id 332 (Mitre), `Retiro (LSM)`
id 463 (San Martín), `Once` id 293 (Sarmiento), `Constitución` id 93 (Roca), `Palermo` id 297… —
es decir, sirve las cabeceras/estaciones más próximas sin importar la línea. Es el único uso
activo del GPS en la app (§7b): ubicación en primer plano, a demanda.

### `GET /v1/servicios/tipos` → **404** `{"code":404,"message":"Petición inválida"}`
El bundle declara la clave `url_GetTipoDeServicios` → `/v1/servicios/tipos`, pero el servidor
responde **404** a la llamada sin parámetros. Endpoint declarado pero no resoluble tal cual
(deprecado o requiere parámetros no observables en el cliente). El `tipo` de servicio ya viene
embebido en cada arribo (`servicio.tipo = {id,nombre,clasificacion,subclasificacion}`), así que
la app no depende de este endpoint.

## 8. Permisos relevantes (manifest)

Ubicación fina/aproximada (estaciones cercanas, geoAlerts), notificaciones (`POST_NOTIFICATIONS`,
FCM), `FOREGROUND_SERVICE` + `LocationTaskService` (ubicación en segundo plano para alertas de
proximidad), almacenamiento, `RECEIVE_BOOT_COMPLETED`, y permisos de badges de varios launchers.

## 9. Flujo de ejemplo (San Martín, validado contra la API)

```
1. POST /v1/auth/authorize                          → token
2. GET  /v1/infraestructura/gerencias?idEmpresa=1   → San Martín = idGerencia 31
3. GET  /v1/infraestructura/ramales?idGerencia=31   → 131 Retiro–Junín, 65 Retiro–Cabred
4. GET  /v1/infraestructura/estaciones?idRamal=131  → 23 estaciones (traza, lat/long por orden)
5. GET  /v1/arribos/estacion/463                    → servicios en vivo (GPS, estado, horarios)
```
Capturado en vivo: servicio #3907 (formación B953, diésel), ramal Retiro–Cabred, "En Andén"
en Retiro andén 3, salida 00:48Z, rumbo a Pilar, a 179 m y 2 km/h.

## 10. Pendientes / no verificado

- Mapa completo de códigos `estado.id` → etiqueta/color. Conocidos: `10` Programado,
  `25` En Andén, `35` Partió; faltan los de demora/cancelado/info y su color exacto.
- `servicios/tipos`: da 404 sin parámetros (§7f); se desconoce la firma correcta (si existe).
  No es bloqueante: el `tipo` ya viene en cada arribo.
- Flujos que requieren cuenta real y no se ejercitaron en vivo (solo análisis estático):
  Firebase Auth (§7e) y escritura/lectura del perfil `users/{uid}` (§7).
```

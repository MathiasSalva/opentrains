# OpenTrains

Cliente liviano para Windows que muestra en vivo los trenes metropolitanos y regionales de
Argentina: próximos Arribos por Estación, Favoritos e Instalaciones de la Estación.

Uso personal. Consume la API no oficial de SOFSE (ver
[ADR-0002](docs/adr/0002-api-no-oficial-de-sofse-detras-de-un-puerto.md) y
[el análisis de la API](docs/reverse-engineering/trenes-argentinos-api.md)).

## Requisitos

- Rust (stable, toolchain MSVC) y Node.js 20 o más nuevo.
- Windows 10/11 con WebView2 (ya viene en Windows 11).

## Uso

```bash
cd app && npm install
```

```bash
npm run tauri dev
```

Instalador para Windows (NSIS, por usuario, sin permisos de administrador). Crea el acceso
en el menú Inicio y queda en `target/release/bundle/nsis/`:

```bash
npm run tauri build
```

Ejecutable suelto sin instalador (`target/release/opentrains-app.exe`):

```bash
npm run tauri build -- --no-bundle
```

La app vive en la bandeja del sistema: clic en el ícono abre el panel, clic derecho muestra
el menú (abrir, iniciar con Windows, salir). Ver [ADR-0003](docs/adr/0003-widget-de-bandeja.md).

## Desarrollo

```bash
cargo test --workspace
```

```bash
cargo run -p opentrains-sofse --example smoke
```

El segundo comando es un smoke test manual contra la API real. Los tests nunca la llaman.

## Estructura (hexagonal)

| Carpeta | Rol |
|---|---|
| `crates/core` | Dominio, puertos y casos de uso. Sin dependencias de infraestructura. |
| `crates/sofse` | Adaptador de salida: API de SOFSE (credencial diaria, token, mapeo). |
| `crates/storage` | Adaptador de salida: Favoritos y caché del catálogo en `%APPDATA%\OpenTrains`. |
| `app/` | Tauri 2 + Svelte: punto de composición y adaptador de entrada (comandos). |

El glosario del dominio está en [CONTEXT.md](CONTEXT.md).

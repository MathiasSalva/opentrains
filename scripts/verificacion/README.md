# Verificación en vivo

La ventana de la app es un WebView2; con el puerto de depuración abierto se maneja por CDP.

```bash
cd app
WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS=--remote-debugging-port=9333 npm run tauri dev
```

- `node cdp.mjs "<expresión JS>"`: evalúa en la ventana (p. ej. `document.body.innerText`).
- `node shot.mjs captura.png`: captura de la ventana.
- `node estados.mjs captura.png light|dark`: intercepta `ipc.localhost/arribos` (Fetch de CDP)
  con todas las Situaciones de ejemplo y captura. `__TAURI_INTERNALS__.invoke` no se puede pisar.
- `flujo.js`: pegar en `cdp.mjs` para elegir Línea/Ramal/Estación desde los selects.

Logs útiles en stderr: `[avisos]` (notificaciones) y `[bandeja]` (tooltip, solo debug).
No se puede automatizar: clics en el ícono/menú de la bandeja ni ver el toast.
Si hay una instancia instalada abierta, single-instance bloquea la de dev: cerrarla antes.

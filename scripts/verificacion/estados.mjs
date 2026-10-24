// Uso: [VENTANA=tablero] node estados.mjs <png> <light|dark> [ancho, 420 por defecto; 360 = celular]
// Muestra todas las Situaciones, un Cambio de andén y Noticias con datos de ejemplo
// (intercepta los comandos `arribos` y `noticias`) y saca una captura.
import { writeFileSync } from "node:fs";

const [salida, esquema] = process.argv.slice(2);
// VENTANA=tablero elige el tablero; por defecto, el panel.
const esVentana = (t) =>
  t.type === "page" && (process.env.VENTANA === "tablero") === t.url.includes("/tablero");
const targets = await (await fetch("http://127.0.0.1:9333/json")).json();
const ws = new WebSocket(targets.find(esVentana).webSocketDebuggerUrl);
await new Promise((r) => (ws.onopen = r));
let id = 0;
const pendientes = new Map();
ws.onmessage = (m) => {
  const msg = JSON.parse(m.data);
  pendientes.get(msg.id)?.(msg.result);
};
const enviar = (method, params = {}) =>
  new Promise((r) => {
    pendientes.set(++id, r);
    ws.send(JSON.stringify({ id, method, params }));
  });

await enviar("Emulation.setDeviceMetricsOverride", { width: Number(process.argv[4] ?? 420), height: 760, deviceScaleFactor: 1, mobile: false });
await enviar("Emulation.setEmulatedMedia", { features: [{ name: "prefers-color-scheme", value: esquema }] });

// Datos de ejemplo con todas las Situaciones; se sirven interceptando la IPC de Tauri
// (http://ipc.localhost/arribos) con el dominio Fetch de CDP.
const ahora = Date.now();
const t = (s) => new Date(ahora + s * 1000).toISOString();
const A = { id: 65, nombre: "Retiro-Cabred" };
const B = { id: 131, nombre: "Retiro - Junín" };
const a = (numero, ramal, destino, estado, situacion, seg, extra = {}) => ({
  numero, ramal, destino, estado, situacion, formacion: null, anden: null,
  anden_anterior: null, noticia: null,
  programada: t(seg), demora_minutos: null, momento: t(seg), ...extra,
});
const datos = [
  a(3301, A, "Pilar", "Programado", "cancelado", 180),
  a(3303, A, "Pilar", "Programado", "programado", 40, {
    anden: "3", anden_anterior: "1", noticia: "Circula con 4 coches.",
  }),
  a(3305, B, "Junín", "En Andén", "en_anden", 300, { anden: "5" }),
  a(3307, A, "José C. Paz", "Partió", "demorado", 540, { demora_minutos: 6, programada: t(180) }),
  a(3309, B, "Junín", "Partió", "en_camino", 720),
  a(3311, B, "Mercedes", "Confirmado", "desconocido", 900),
  a(3313, A, "Pilar", "Programado", "programado", 1500),
  a(3315, A, "Dr. Cabred", "Programado", "programado", 2220),
];
const noticias = [
  { id: 1, destacada: true, texto: "Servicio limitado entre Retiro y Hurlingham por obras en la vía." },
  { id: 2, destacada: false, texto: "Las personas con CUD pueden viajar en tren sin costo con SUBE." },
  { id: 3, destacada: false, texto: "Ascensor del andén 3 fuera de servicio." },
];
const cuerpos = {
  arribos: Buffer.from(JSON.stringify(datos)).toString("base64"),
  noticias: Buffer.from(JSON.stringify(noticias)).toString("base64"),
};
const anterior = ws.onmessage;
ws.onmessage = (m) => {
  const msg = JSON.parse(m.data);
  if (msg.method === "Fetch.requestPaused") {
    const preflight = msg.params.request.method === "OPTIONS";
    ws.send(JSON.stringify({
      id: ++id,
      method: "Fetch.fulfillRequest",
      params: {
        requestId: msg.params.requestId,
        responseCode: preflight ? 204 : 200,
        responseHeaders: [
          { name: "Content-Type", value: "application/json" },
          { name: "Tauri-Response", value: "ok" },
          { name: "Access-Control-Allow-Origin", value: "*" },
          { name: "Access-Control-Allow-Headers", value: "*" },
          { name: "Access-Control-Expose-Headers", value: "Tauri-Response" },
        ],
        body: preflight ? "" : cuerpos[msg.params.request.url.includes("/noticias") ? "noticias" : "arribos"],
      },
    }));
    return;
  }
  anterior(m);
};
await enviar("Fetch.enable", {
  patterns: [{ urlPattern: "*ipc.localhost/arribos*" }, { urlPattern: "*ipc.localhost/noticias*" }],
});
const mock = `(() => {
  document.querySelector('button[title="Actualizar"]').click();
  return new Promise((r) => setTimeout(() => { window.scrollTo(0, document.querySelector('.estacion').offsetTop - 8); r("ok"); }, 1800));
})()`;
if (process.env.VENTANA === "tablero") {
  // En el tablero no hay botón de actualizar: se recarga con la intercepción activa.
  await enviar("Page.reload");
  await new Promise((r) => setTimeout(r, 3000));
} else {
  await enviar("Runtime.evaluate", { expression: mock, awaitPromise: true });
}
await enviar("Fetch.disable");
const shot = await enviar("Page.captureScreenshot", { format: "png", captureBeyondViewport: true });
writeFileSync(salida, Buffer.from(shot.data, "base64"));
ws.close();

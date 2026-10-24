import { writeFileSync } from "node:fs";
// VENTANA=tablero elige el tablero; por defecto, el panel.
const esVentana = (t) =>
  t.type === "page" && (process.env.VENTANA === "tablero") === t.url.includes("/tablero");
const targets = await (await fetch("http://127.0.0.1:9333/json")).json();
const ws = new WebSocket(targets.find(esVentana).webSocketDebuggerUrl);
await new Promise((r) => (ws.onopen = r));
ws.onmessage = (m) => { const msg = JSON.parse(m.data); if (msg.id === 1) { writeFileSync(process.argv[2], Buffer.from(msg.result.data, "base64")); ws.close(); } };
ws.send(JSON.stringify({ id: 1, method: "Page.captureScreenshot", params: { format: "png", captureBeyondViewport: true } }));

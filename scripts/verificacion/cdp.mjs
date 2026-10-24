// Uso: node cdp.mjs "<expresión JS>"  — evalúa en la ventana de la app vía CDP.
// VENTANA=tablero elige el tablero; por defecto, el panel.
const esVentana = (t) =>
  t.type === "page" && (process.env.VENTANA === "tablero") === t.url.includes("/tablero");
const targets = await (await fetch("http://127.0.0.1:9333/json")).json();
const page = targets.find(esVentana);
const ws = new WebSocket(page.webSocketDebuggerUrl);
await new Promise((r) => (ws.onopen = r));
ws.onmessage = (m) => {
  const msg = JSON.parse(m.data);
  if (msg.id === 1) {
    const r = msg.result.result;
    console.log(r.value ?? JSON.stringify(msg.result));
    ws.close();
  }
};
ws.send(JSON.stringify({ id: 1, method: "Runtime.evaluate", params: { expression: process.argv[2], awaitPromise: true, returnByValue: true } }));

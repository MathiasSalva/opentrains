# Widget de bandeja en vez de ventana normal

En el MVP, OpenTrains era una ventana común, sin ícono en la bandeja. Ahora vive en la bandeja del sistema, que es lo que en Windows equivale al dock de iOS o Linux. El ícono abre un panel flotante sobre la barra de tareas, como los de Wi-Fi o volumen. El panel se oculta al perder el foco o con Escape, y cerrarlo no sale de la app.

Se eligió así para consultar el próximo tren de un vistazo sin perder la ventana entre otras. Se descartó un widget flotante siempre visible porque tapa el trabajo.

## Consequences

- Los Arribos se siguen refrescando con el panel oculto. El tooltip del ícono muestra el próximo tren con hora absoluta, que no envejece entre refrescos. Con la ventana oculta, WebView2 puede espaciar los timers hasta uno por minuto.
- "Iniciar con Windows" es opcional y viene apagado. Cuando arranca así, la app abre oculta (`--oculta`).
- Hay una sola instancia: abrir la app de nuevo, por ejemplo desde el menú Inicio, muestra el panel de la que ya está corriendo.
- Todo esto es solo de escritorio (`cfg(desktop)`). El puerto a Android no lo usa.

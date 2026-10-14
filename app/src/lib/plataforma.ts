/**
 * `true` en el panel flotante de escritorio; `false` en Android e iOS, donde hay una sola
 * webview a pantalla completa y no tienen sentido ocultar la ventana ni el foco.
 */
export const esEscritorio =
  typeof navigator === "undefined" || !/Android|iPhone|iPad|iPod/i.test(navigator.userAgent);

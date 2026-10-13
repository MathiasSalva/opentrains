// Los horarios se muestran siempre en hora de Argentina, sin importar la zona de la PC.
const horaArgentina = new Intl.DateTimeFormat("es-AR", {
  timeZone: "America/Argentina/Buenos_Aires",
  hour: "2-digit",
  minute: "2-digit",
  hour12: false,
});

export function hora(iso: string): string {
  return horaArgentina.format(new Date(iso));
}

export function cuentaRegresiva(iso: string, ahora: number): string {
  const segundos = Math.round((new Date(iso).getTime() - ahora) / 1000);
  if (segundos <= 0) return "Ahora";
  if (segundos < 60) return "< 1 min";
  return `${Math.floor(segundos / 60)} min`;
}

export function hace(desde: number, ahora: number): string {
  const segundos = Math.max(0, Math.round((ahora - desde) / 1000));
  if (segundos < 60) return `${segundos} s`;
  return `${Math.floor(segundos / 60)} min`;
}

# API no oficial de SOFSE detrás de un puerto

Los datos en vivo (Arribos, estado y posición de los Servicios) y el catálogo (Líneas, Ramales, Estaciones) vienen de la API REST no documentada que usa la app oficial "Trenes Argentinos", descubierta por ingeniería inversa (ver `docs/reverse-engineering/trenes-argentinos-api.md`). No existe una fuente oficial pública con datos en tiempo real. SOFSE puede cambiar el esquema de credencial o los endpoints en cualquier momento, así que todo el acceso vive en el adaptador `crates/sofse` detrás de los puertos de `core`, y la app es de uso personal, no se publica. Si aparece una fuente oficial (por ejemplo, el GTFS de datos.gob.ar para horarios), se reemplaza o se suma un adaptador sin tocar el dominio.

## Consequences

- Un 401 persistente se presenta al usuario como "la fuente de datos cambió", no como un error genérico.
- Se respeta el ritmo de la app oficial: Arribos cada 45 s solo con la vista visible, y catálogo cacheado en disco.

# Investigación: históricos de ocupación (A) y de cancelaciones/demoras por Servicio (B)

Status: needs-triage

Fecha: 2026-10-13. Llamadas en vivo: 5 (1 auth + `estaciones?nombre=Paz`, `arribos/estacion/194`, 3 sondeos de endpoints).

## Resumen

- **A (horarios con más gente):** factible solo como **perfil típico estático**, no en vivo. La API no
  trae ocupación. La mejor fuente es SUBE: `operaciones-sube-hexagonos` (día hábil tipo, por **hora** y
  hexágono H3 de ~0,7 km², modo `TREN`, octubre 2025), que se puede cruzar con las coordenadas de cada
  Estación. Para el día de la semana, `sube-cantidad-de-transacciones-usos-por-fecha` (diario por Línea,
  actualizado a diario). Fines de semana por hora: no hay dato.
- **B (Servicios con muchas cancelaciones/demoras):** **no existe dato público por Servicio.** CNRT publica
  programados/cancelados/puntuales **por Ramal y mes** (hasta agosto 2026): sirve de línea base, no para
  "el de las 7:30 del sábado". Lo de cada Servicio solo sale de un **historial propio**. Es viable porque cada
  Arribo trae `servicio.estaciones[]` con horas **`real`** de las paradas ya pasadas.

## Lo que trae la API

Verificado en vivo con `GET /v1/arribos/estacion/194` (José C. Paz, idEstacion 194). La respuesta trajo 11 Servicios del
Ramal Retiro-Cabred, programados entre 21:32Z y 22:55Z, o sea unos **85 min hacia adelante**. Scripts
descartables en el scratchpad, no en el repo.

- `servicio` trae estas claves: `cancelacion, desde, equipo, estaciones, gerencia, hasta, id, leyenda, location, numero,
  oculto, ramal, sentido, tipo`. **No hay ningún campo de ocupación ni de carga.**
- `cancelacion` vino `null` en los 11 Servicios. Su forma cuando no es nula **sigue sin conocerse**: el
  mapeo actual solo mira si es no nula (`crates/sofse/src/mapeo.rs:203`). `leyenda` también vino `null`.
- `tipo` = `{id, nombre: "Normal"|"Semi Rapido", programado, clasificacion, subclasificacion}`. Sirve para
  separar los Servicios rápidos, pero no aporta nada histórico.
- `desde.estado`: aparecieron 20 "Confirmado", 25 "En Andén" y 35 "Partió". El id 20 no estaba documentado en §10.
- **Clave para B:** cada ítem de `servicio.estaciones[]` de una parada ya pasada trae `llegada.real` /
  `salida.real` (ej. #3405: Retiro programada 20:18, real 20:20:30). También aparece `conciliacion.salida`. Las
  paradas futuras solo traen `estimada`, y en las lejanas se repite el mismo valor (21:32:58 de Manzanares a
  Cabred), así que las estimadas a más de unas pocas estaciones no son confiables.
- `arribo.salida.estimada` falta en los Servicios "Confirmado"/"En Andén" (solo hay `programada`).
- **No hay endpoint histórico:** `/v1/servicios` → 404, `/v1/estadisticas` → 404 (`{"code":404,"message":"Petición
  inválida"}`). `?fecha=2026-10-11` sobre `arribos` se ignora: la respuesta es idéntica byte a byte, con el mismo
  `timestamp`. `/v1/servicios/tipos` ya figuraba como 404 en el doc §7f.
- Consecuencia: un Servicio que pasó solo deja de aparecer. Sin un horario de referencia no hay forma de
  distinguir "pasó" de "fue cancelado y retirado de la lista". Queda **por verificar** si los cancelados se listan con
  `cancelacion` o desaparecen. Hace falta observar uno en vivo.

## Datos abiertos

| Fuente | Granularidad | Actualización / último dato | Formato, licencia | ¿Mapea al dominio? |
|---|---|---|---|---|
| SUBE usos por fecha (datos.transporte.gob.ar `sube-cantidad-de-transacciones-usos-por-fecha`) | **día × línea** (`FFCC_SANM`, `FFCC_MITRE_CAPILLA`…), con `DATO_PRELIMINAR` | **diaria**. `dat-ab-usos-2026.csv` modificado 2026-10-13 con datos hasta 2026-10-11 | CSV (55 MB/año), CC-BY 4.0 | Línea, y algunos Ramales por código. Sin Estación ni hora. Sirve para A (día de la semana). |
| SUBE hexágonos (`operaciones-sube-hexagonos`) | **día hábil tipo × hora × H3 res8 × modo** (`TREN`) | esporádica: tipo oct-2019 y **oct-2025** (subido 2026-04-08) | CSV, licencia no especificada | Estación por cercanía lat/long (catálogo de la API). Son validaciones (entradas), no carga a bordo. Mejor fuente para A por hora. |
| Viajes y etapas AMBA (data.buenosaires.gob.ar `viajes-etapas-transporte-publico`) | día hábil tipo × `rango_horario` × `linea_etapa` × origen/destino (H3 res10) × factor de expansión | anual 2019–2024 (oct-nov) | CSV de **1,2 GB** por año, CC-BY | Estación por coordenadas. Con O/D se puede estimar la **carga por tramo y hora**. Pesado. Solo hábiles. |
| CNRT "Cumplimiento de la programación de trenes" | **Ramal × mes**: programados, cancelados, corridos, puntuales, atrasados, regularidad | mensual, **agosto 2026** | XLSX (`…cumplimiento_de_programa_2026-08_cnrt.xlsx`, hojas por Línea), licencia no indicada | Ramal por nombre ("Ramal Retiro-Pilar-Cabred (diesel)"), con mapeo manual a RamalId. **Línea base para B.** |
| CNRT "Boletos vendidos por estación" | **Estación × mes** | mensual, agosto 2026 (zip con 8 XLSX, uno por Línea) | XLSX, licencia no indicada | Estación por nombre ("Jose C. Paz"). Ranking de las estaciones más concurridas, sin hora. |
| CNRT informe estadístico anual (PDF 2021) | red/línea × mes | anual | PDF | Solo contexto. Define regularidad absoluta/relativa. |
| GTFS trenes (data.buenosaires `trenes-gtfs`, MobilityDatabase mdb-647) | horario por viaje | **vigencia 2020-02-01 a 2020-04-30**. Recurso de 2019 | ZIP, CC-BY 2.5 AR | Obsoleto, no sirve como horario de referencia. |
| API transporte CABA (`api-transporte-publico`) | tiempo real | **suspendida** ("todos los datasets con Formato API y GTFS, están suspendidos") | API | No. |
| Estaciones FFCC 2022 (`estaciones-de-trenes-y-servicios-activos-a-2022`) | estación (geo) | 2022 | CSV/JSON/SHP, CC-BY 4.0 | Redundante con el catálogo de la API. |

No encontré ningún dataset público de puntualidad o cancelación **por Servicio**, ni de ocupación por hora en fines de
semana. Búsquedas CKAN en datos.gob.ar y datos.transporte.gob.ar por `gtfs`, `puntualidad` y `tren`: 0 resultados relevantes.

## Historial propio (diseño)

**Qué registrar.** Una `Observacion` por (fecha operativa AR, `numero`, Estación) que el sistema ve:
Línea, Ramal, `sentido`, `tipo`, programada, estimada/real, `cancelado`, estado, `formacion`, primera y última vez vista.
Se hace upsert y queda solo la última observación, que es la más informativa. Además, de `servicio.estaciones[]` se
guardan las horas `real` de las paradas ya pasadas. **Una sola respuesta de Arribos describe todo el recorrido del Servicio**, no
solo la Estación consultada.

**Métricas.** Por (Servicio `numero`, tipo de día L–V/sáb/dom-feriado, franja de 30 min, Estación):
- tasa de cancelación = cancelados / vistos;
- demora real en la Estación: mediana, p80 y % de casos con más de 5 min.
- "Esperado pero no visto": un Servicio que apareció en ≥ 3 días del mismo tipo y falta en un día con
  cobertura completa de esa franja. Es un **posible cancelado silencioso** y se cuenta aparte, nunca como cancelación confirmada.

**Cuántas semanas.** Intervalo de Wilson al 95 %: 2 cancelaciones en 8 obs. → 7–59 %; 3/10 → 11–60 %;
5/15 → 15–58 %; 0/12 → 0–24 %. Para afirmar "alto" con algún respaldo (cota inferior por encima de la
base del Ramal según CNRT) hacen falta **~10–15 observaciones** por Servicio:
- un Servicio de lunes a viernes (mismo `numero` cada día hábil, a confirmar): **2–3 semanas**;
- **el sábado de las 7:30: 10–15 semanas (≈ 3 meses)**. Antes de eso conviene mostrar "pocos datos (n=4)".
- Para que una señal se vea antes, conviene agregar con contracción bayesiana: prior Beta centrado en la tasa mensual del Ramal
  (CNRT) y agrupar franjas vecinas. Los cambios de cronograma renumeran Servicios y reinician la cuenta, así que hay
  que detectarlos cuando aparezcan números nuevos.

**Sesgo de observar solo lo que el usuario abre.** La app solo hace polling con la vista visible y la Estación
elegida. El resultado: (1) solo hay datos de las franjas en que el usuario mira, que suelen ser las de su viaje, y está
bien si B se limita a "mis Servicios"; (2) la cobertura de una franja es incompleta, así que no se puede inferir "no vino" en
horas sin observar; (3) sesgo de selección: el usuario abre la app más cuando hay problemas. Con los Seguimientos ya activos
el sesgo baja un poco, porque se hace polling de ese Servicio hasta que pasa.

**Recolector de fondo (opcional).** Un conjunto fijo y chico: 1–2 Estaciones por Ramal de interés, **cerca de la cabecera de
llegada de cada sentido**, para que `estaciones[]` traiga casi todo el recorrido con `real`. Una llamada **cada 30 min**
alcanza, porque la ventana de la API es de unos 85 min. Son 48 llamadas por día y Estación. Como referencia: mirar una Estación
1 hora en la app son 80 llamadas (45 s). Un Ramal en los dos sentidos son ~96 llamadas por día, por debajo del uso normal de la
app oficial. ADR-0002 aplica igual: es una API no oficial, así que el recolector tiene que ser opt-in, estar apagado por defecto,
detenerse ante un 401/429 y no más de 1 llamada simultánea. Requiere la PC prendida. Las horas con la PC apagada se marcan
como "sin cobertura" y no cuentan como ausencia.

**Tamaño.** Una fila son ~80–120 B. Una Estación × ~150 Servicios/día son ~15 KB/día, **~5 MB/año**. Guardar todas
las paradas de `estaciones[]` de un Ramal (~150 × 22) son ~350 KB/día, ~120 MB/año: conviene limitarlo a las
Estaciones de interés o compactar mes a mes en agregados. Hoy `crates/storage` es un `JsonStore` (JSON completo
reescrito). Para el historial conviene JSONL append-only por mes o SQLite (`rusqlite` bundled, una dependencia nueva que
habría que decidir).

**Encaje hexagonal.**
- `core/dominio`: `Observacion`, `TipoDeDia`, `Franja`, `Desempeno { vistos, cancelados, demora_p50, demora_p80, cota_inferior }`.
  **La agregación es una función pura del dominio** (`desempeno(observaciones, prior) -> Desempeno`) y se puede testear sin I/O.
- `core/puertos`: `HistorialRepo { registrar(&[Observacion]), observaciones(filtro: Ramal/Estación/numero/rango) }`.
- `core/casos_de_uso`: `RegistrarHistorial`, un **decorador de `ArribosSource`** que devuelve los Arribos sin cambios y los
  registra al pasar. Así ConsultaArribos y Seguimientos alimentan el historial **sin llamadas extra**.
  `ConsultaDesempeno` responde "¿qué tan confiable es este Servicio en esta franja?". `Recolector`, opcional, comparte el
  `ArribosSource` y lo programa la app (tokio task en `app/`).
- `crates/storage`: adaptador `HistorialJsonl` o `HistorialSqlite`. `crates/sofse`: exponer `real` y `cancelacion` en el
  mapeo, que hoy no llegan a `Arribo`.
- Antes de implementar hay que sumar a CONTEXT.md términos como "Historial" y "Desempeño del Servicio" (y evitar "estadísticas").

## Recomendación

**A — perfil típico de ocupación: factible con datos públicos, no en vivo.**
- Fuente: SUBE hexágonos oct-2025 (hora × zona, `TREN`) + coordenadas de las Estaciones del catálogo para la curva
  horaria de cada Estación en día hábil. SUBE usos diarios para el factor por día de la semana y Línea. CNRT boletos por
  estación para el ranking mensual. Más adelante, opcional, la carga por tramo con viajes-etapas (1,2 GB).
- Limitaciones: mide **entradas** (validaciones), no gente a bordo ni en el andén. Es un día típico y no el de hoy. No hay
  datos por hora de fines de semana. El H3 mezcla Estaciones cercanas (por ejemplo, hubs con varias Líneas). Licencia del
  dataset de hexágonos sin especificar. El historial propio **no** ayuda, porque la API no trae ocupación.
- Primer paso: un script offline, fuera de la app, que tome `trx2025.csv` (modo `TREN`) y las Estaciones del Ramal 131
  y genere un JSON `{estacion_id: [24 valores normalizados]}`. Validarlo a ojo con José C. Paz y Retiro. Si convence, el
  JSON va como asset estático y se muestra como "suele haber mucha gente 7–9 h".

**B — Servicios propensos a cancelación/demora: factible solo con historial propio, con base de CNRT.**
- Fuente: el historial propio (pasivo primero) para el nivel Servicio. CNRT cumplimiento (Ramal × mes) como prior y como
  contexto ("en agosto el Ramal X canceló N %").
- Limitaciones: pocos datos por meses en Servicios semanales (sábado). No se sabe cómo se ve un cancelado en la API.
  Los cancelados silenciosos son inferencias. Hay sesgo hacia las franjas del usuario. Un cambio de cronograma reinicia todo.
- Primer paso: (1) observar en vivo un Servicio cancelado y documentar la forma de `cancelacion` y si sigue listado
  (§10 del doc de la API). (2) Implementar el decorador pasivo `RegistrarHistorial` + JSONL, sin llamadas extra, y solo
  después de 4–6 semanas de datos decidir si vale la pena el recolector de fondo y la UI.

## Fuentes

- Repo: `CONTEXT.md`, `docs/reverse-engineering/trenes-argentinos-api.md` (§6, §7f, §10), `docs/adr/0002-api-no-oficial-de-sofse-detras-de-un-puerto.md`,
  `crates/core/src/puertos.rs`, `crates/sofse/src/mapeo.rs`, `crates/storage/src/lib.rs`.
- API (en vivo, 2026-10-13 ~21:28 ART): `https://api-servicios.sofse.gob.ar/v1/infraestructura/estaciones?nombre=Paz`,
  `/v1/arribos/estacion/194` (+ `?fecha=`), `/v1/servicios`, `/v1/estadisticas`.
- https://datos.gob.ar/api/3/action/package_search?q=trenes (11 datasets, Secretaría de Transporte)
- https://datos.transporte.gob.ar/api/3/action/package_show?id=sube-cantidad-de-transacciones-usos-por-fecha
- https://archivos-datos.transporte.gob.ar/upload/Dat_Ab_Usos/dat-ab-usos-2026.csv (encabezado y cola leídos)
- https://datos.transporte.gob.ar/api/3/action/package_show?id=operaciones-sube-hexagonos (`trx2025.csv`: `id,id_h3,hora,modo,cantidad_trx`)
- https://datos.transporte.gob.ar/api/3/action/package_show?id=boletos-vendidos (CNRT 2016, viejo)
- https://data.buenosaires.gob.ar/api/3/action/package_show?id=viajes-etapas-transporte-publico (encabezado de `etapas_BAdata_20241016.csv`)
- https://data.buenosaires.gob.ar/api/3/action/package_show?id=api-transporte-publico (suspendida); `trenes-gtfs` vía package_search
- https://mobilitydatabase.org/feeds/gtfs/mdb-647 (GTFS Trenes Argentinos, vigencia feb–abr 2020)
- https://www.argentina.gob.ar/transporte/cnrt/estadisticas-ferroviarias
- https://www.argentina.gob.ar/sites/default/files/ffcc_amba_cumplimiento_de_programa_2026-08_cnrt.xlsx (hojas por Línea, columnas Programados/Cancelados/Puntuales/Atrasados por Ramal y mes)
- https://www.argentina.gob.ar/sites/default/files/ffcc_amba_pax_x_estacion_2026-08_cnrt.zip (Año/Mes/Estación/PAX)
- https://www.argentina.gob.ar/sites/default/files/informe_estadictico_2021_ffcc_pasajeros_metropolitanos_amba_if-2022-34127307-apn-sfgsmcnrt.pdf (definiciones de regularidad)

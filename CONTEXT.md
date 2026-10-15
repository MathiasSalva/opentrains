# OpenTrains

Cliente ligero para consultar en vivo los trenes metropolitanos y regionales de Argentina: qué Servicios van a pasar por una Estación, cuándo y en qué estado.

## Language

### Red ferroviaria

**Línea**:
Agrupación comercial de la red que el usuario reconoce por nombre (Sarmiento, Mitre, Roca, Belgrano Sur, San Martín, Tren de la Costa, Regionales).
_Avoid_: Gerencia

**Ramal**:
Recorrido definido dentro de una Línea, entre dos Estaciones cabecera (ej. Retiro–Pilar).
_Avoid_: Branch, ruta

**Estación**:
Punto de la red donde los Servicios se detienen; puede pertenecer a varios Ramales.
_Avoid_: Parada, station

**Cabecera**:
Estación en uno de los dos extremos de un Ramal.
_Avoid_: Terminal

**Instalaciones de la Estación**:
Lo que ofrece una Estación al pasajero: accesibilidad (ascensor, rampa), baños, boletería, wifi, carga SUBE y conexiones con otros transportes.
_Avoid_: Servicios de la estación, info de estación

### Operación

**Servicio**:
Un viaje numerado concreto (ej. #3907) que recorre un Ramal en un sentido con horarios propios; es lo que el usuario llama "el tren de las 21:48 a Pilar".
_Avoid_: Tren (fuera de la UI), viaje

**Destino**:
Estación donde termina un Servicio concreto; puede no ser una Cabecera de su Ramal.
_Avoid_: Hasta, cabecera final

**Formación**:
El material rodante físico (ej. B953) que ejecuta un Servicio; puede cubrir distintos Servicios a lo largo del día.
_Avoid_: Equipo, tren

**Arribo**:
El paso de un Servicio por una Estación concreta, con su andén, horarios programado/estimado y cuenta regresiva.
_Avoid_: Llegada (es solo uno de sus horarios), parada

**Estado del Servicio**:
Situación operativa actual de un Servicio según el operador (ej. Programado, En Andén, Partió); puede ser un estado aún no catalogado.
_Avoid_: Status

**Situación del Arribo**:
Lo que el pasajero necesita saber de un Arribo de un vistazo: cancelado, en andén, demorado, en camino, programado o sin catalogar. Se deriva del Estado del Servicio, la demora y la cancelación, en ese orden de prioridad.
_Avoid_: Tono, status

**Servicio cancelado**:
Servicio que el operador marcó con una cancelación; se sigue mostrando, señalado como tal.

**Noticia**:
Texto que publica el operador sobre una Línea entera, un Ramal o una Estación, con una criticidad y un período de vigencia (ej. un corte por obras, un beneficio de pasaje).
_Avoid_: Alerta, comunicado, aviso (Aviso es lo que notifica la app)

**Noticia del Servicio**:
Texto suelto que el operador adjunta a un Servicio concreto; no tiene criticidad ni vigencia propias, dura lo que dura el Servicio.
_Avoid_: Leyenda, nota

**Cambio de andén**:
Cuando un Arribo que ya tenía andén pasa a tener otro distinto. Que se le asigne andén por primera vez, o que lo pierda, no es un Cambio de andén.
_Avoid_: Reasignación

**Servicio oculto**:
Servicio que el operador marcó para no mostrarse al público; nunca se le presenta al usuario.

### Usuario

**Seguimiento**:
Un Servicio concreto que el usuario sigue en una Estación para recibir Avisos; termina cuando el Servicio ya pasó.
_Avoid_: Alarma, suscripción

**Aviso**:
Lo que la app le notifica al usuario sobre un Servicio que sigue: que está por pasar (según la anticipación elegida), que tiene demora o que fue cancelado.
_Avoid_: Alerta, notificación (es el medio, no el concepto)

**Ubicación**:
Dónde está el usuario ahora, según el dispositivo; se consulta al abrir o refrescar y no se guarda. Las distancias a las Estaciones se miden desde acá, en línea recta.
_Avoid_: Posición GPS, geolocalización

**Estaciones cercanas**:
Las Estaciones más próximas a la Ubicación, de cualquier Línea.
_Avoid_: Cerca tuyo (es el rótulo de la UI)

**Favorito**:
Una Estación guardada por el usuario, opcionalmente acotada a un Ramal, para ver sus Arribos en un clic.
_Avoid_: Bookmark, guardado

<script lang="ts">
  import { onMount } from "svelte";
  import { listen } from "@tauri-apps/api/event";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import {
    api,
    coordenadas,
    describirError,
    mismoFavorito,
    type Arribo,
    type Estacion,
    type Favorito,
    type Linea,
    type Noticia,
    type Ramal,
  } from "$lib/api";
  import { claseSituacion, clave } from "$lib/arribos";
  import { cuentaRegresiva, hace, hora } from "$lib/formato";
  import SituacionPill from "$lib/components/SituacionPill.svelte";
  import WarningIcon from "phosphor-svelte/lib/WarningIcon";
  import MegaphoneIcon from "phosphor-svelte/lib/MegaphoneIcon";
  import CornersOutIcon from "phosphor-svelte/lib/CornersOutIcon";
  import CornersInIcon from "phosphor-svelte/lib/CornersInIcon";
  import XIcon from "phosphor-svelte/lib/XIcon";
  import WifiSlashIcon from "phosphor-svelte/lib/WifiSlashIcon";

  // Tablero de Estación: solo para mirar, de lejos y en un segundo monitor. La barra de
  // arriba (cambiar Estación, pantalla completa, cerrar) aparece al mover el mouse.
  const REFRESCO_MS = 45_000;
  const ROTACION_MS = 10_000;
  const QUIETO_MS = 3_000;

  const ventana = getCurrentWindow();

  let favorito = $state<Favorito | null>(null);
  let arribos = $state<Arribo[]>([]);
  let noticias = $state<Noticia[]>([]);
  let ultimaActualizacion = $state<number | null>(null);
  let error = $state<string | null>(null);
  let ahora = $state(Date.now());
  let pantallaCompleta = $state(false);
  let quieto = $state(false);
  let favoritos = $state<Favorito[]>([]);
  // Selector de otra Estación, dentro de la barra.
  let eligiendo = $state(false);
  let lineas = $state<Linea[]>([]);
  let ramales = $state<Ramal[]>([]);
  let estaciones = $state<Estacion[]>([]);
  let lineaId = $state<number | null>(null);
  let ramalId = $state<number | null>(null);
  let estacionId = $state<number | null>(null);
  let ultimaConsulta = 0;

  const graves = $derived(noticias.filter((n) => n.destacada));
  const resto = $derived(noticias.filter((n) => !n.destacada));
  let girando = $state(0);
  const rotando = $derived(resto.length ? resto[girando % resto.length] : null);
  const desactualizado = $derived(
    ultimaActualizacion !== null && ahora - ultimaActualizacion > 2 * REFRESCO_MS,
  );

  async function refrescar() {
    if (!favorito) return;
    const consulta = ++ultimaConsulta;
    const pedido = $state.snapshot(favorito);
    try {
      const [nuevos, nuevas] = await Promise.all([
        api.arribosTablero(pedido),
        // Las Noticias no ensucian los Arribos: si fallan, quedan las últimas.
        api.noticias(pedido).catch(() => null),
      ]);
      if (consulta !== ultimaConsulta) return;
      arribos = nuevos;
      if (nuevas) noticias = nuevas;
      ultimaActualizacion = Date.now();
      error = null;
    } catch (e) {
      if (consulta === ultimaConsulta) error = describirError(e);
    }
  }

  function mostrar(nuevo: Favorito) {
    favorito = nuevo;
    arribos = [];
    noticias = [];
    ultimaActualizacion = null;
    error = null;
    eligiendo = false;
    ventana.setTitle(`${nuevo.estacion_nombre} · OpenTrains`).catch(() => {});
    refrescar();
  }

  async function cambiar(nuevo: Favorito) {
    try {
      await api.abrirTablero($state.snapshot(nuevo));
      mostrar(nuevo);
    } catch (e) {
      error = describirError(e);
    }
  }

  async function alternarPantallaCompleta() {
    try {
      await api.pantallaCompletaTablero(!pantallaCompleta);
      pantallaCompleta = await ventana.isFullscreen();
    } catch (e) {
      error = describirError(e);
    }
  }

  async function abrirSelector() {
    eligiendo = !eligiendo;
    if (eligiendo && lineas.length === 0) {
      try {
        lineas = await api.lineas();
      } catch (e) {
        error = describirError(e);
      }
    }
  }

  async function elegirLinea() {
    ramales = [];
    estaciones = [];
    ramalId = null;
    estacionId = null;
    if (lineaId !== null) ramales = await api.ramales(lineaId).catch(() => []);
  }

  async function elegirRamal() {
    estaciones = [];
    estacionId = null;
    if (ramalId !== null) estaciones = await api.estaciones(ramalId).catch(() => []);
  }

  function elegirEstacion() {
    const estacion = estaciones.find((e) => e.id === estacionId);
    if (!estacion) return;
    cambiar({
      estacion: estacion.id,
      estacion_nombre: estacion.nombre,
      ramal: null,
      coordenadas: coordenadas(estacion),
      ramales: estacion.ramales?.length ? estacion.ramales : ramalId !== null ? [ramalId] : [],
    });
  }

  function etiqueta(f: Favorito): string {
    return f.ramal ? `${f.estacion_nombre} · ${f.ramal.nombre}` : f.estacion_nombre;
  }

  // El cursor y la barra se esconden cuando el mouse queda quieto.
  let temporizador: ReturnType<typeof setTimeout> | undefined;
  function movio() {
    quieto = false;
    clearTimeout(temporizador);
    temporizador = setTimeout(() => (quieto = !eligiendo), QUIETO_MS);
  }

  function tecla(e: KeyboardEvent) {
    if (e.key === "F11") {
      e.preventDefault();
      alternarPantallaCompleta();
    } else if (e.key === "Escape") {
      // Escape solo sale de pantalla completa: nunca cierra el tablero.
      if (eligiendo) eligiendo = false;
      else if (pantallaCompleta) alternarPantallaCompleta();
    }
  }

  onMount(() => {
    document.documentElement.classList.add("oscuro");
    movio();
    let quitarEscucha: (() => void) | undefined;
    let quitarTamano: (() => void) | undefined;
    (async () => {
      // La pantalla completa también puede cambiar desde Rust (al restaurar al iniciar).
      quitarTamano = await ventana.onResized(async () => {
        pantallaCompleta = await ventana.isFullscreen().catch(() => pantallaCompleta);
      });
      quitarEscucha = await listen<Favorito>("tablero-cambiado", (e) => {
        if (!favorito || !mismoFavorito(e.payload, favorito)) mostrar(e.payload);
      });
      pantallaCompleta = await ventana.isFullscreen().catch(() => false);
      try {
        favoritos = (await api.preferencias()).favoritos;
        const inicial = await api.tablero();
        if (inicial) mostrar(inicial);
      } catch (e) {
        error = describirError(e);
      }
    })();
    const refresco = setInterval(refrescar, REFRESCO_MS);
    const reloj = setInterval(() => (ahora = Date.now()), 1000);
    const rotacion = setInterval(() => girando++, ROTACION_MS);
    return () => {
      quitarEscucha?.();
      quitarTamano?.();
      clearInterval(refresco);
      clearInterval(reloj);
      clearInterval(rotacion);
      clearTimeout(temporizador);
    };
  });
</script>

<svelte:window onkeydown={tecla} onmousemove={movio} onmousedown={movio} />

<main class="tablero" class:quieto>
  <nav class="barra" aria-label="Tablero">
    <div class="chips">
      {#each favoritos as f (etiqueta(f))}
        <button
          class="chip"
          class:activo={favorito !== null && mismoFavorito(f, favorito)}
          onclick={() => cambiar(f)}>{etiqueta(f)}</button
        >
      {/each}
      <button class="chip" aria-expanded={eligiendo} onclick={abrirSelector}>Otra estación</button>
    </div>
    <div class="botones">
      <button
        class="boton"
        aria-label={pantallaCompleta ? "Salir de pantalla completa (F11)" : "Pantalla completa (F11)"}
        title={pantallaCompleta ? "Salir de pantalla completa (F11)" : "Pantalla completa (F11)"}
        onclick={alternarPantallaCompleta}
      >
        {#if pantallaCompleta}<CornersInIcon size={20} weight="bold" />{:else}<CornersOutIcon size={20} weight="bold" />{/if}
      </button>
      <button class="boton" aria-label="Cerrar el tablero" title="Cerrar" onclick={() => ventana.close()}>
        <XIcon size={20} weight="bold" />
      </button>
    </div>
    {#if eligiendo}
      <div class="selector">
        <select aria-label="Línea" bind:value={lineaId} onchange={elegirLinea}>
          <option value={null}>Línea</option>
          {#each lineas as l (l.id)}<option value={l.id}>{l.nombre}</option>{/each}
        </select>
        <select aria-label="Ramal" bind:value={ramalId} onchange={elegirRamal} disabled={!ramales.length}>
          <option value={null}>Ramal</option>
          {#each ramales as r (r.id)}<option value={r.id}>{r.nombre}</option>{/each}
        </select>
        <select
          aria-label="Estación"
          bind:value={estacionId}
          onchange={elegirEstacion}
          disabled={!estaciones.length}
        >
          <option value={null}>Estación</option>
          {#each estaciones as e (e.id)}<option value={e.id}>{e.nombre}</option>{/each}
        </select>
      </div>
    {/if}
  </nav>

  <header class="cabecera">
    <h1>{favorito?.estacion_nombre ?? "Tablero"}</h1>
    <p>
      {#if favorito?.ramal}{favorito.ramal.nombre} · {/if}Próximos trenes
      <span class="reloj">{hora(new Date(ahora).toISOString())}</span>
    </p>
  </header>

  {#if graves.length || rotando}
    <section class="noticias" aria-label="Noticias del operador">
      {#each graves as n (n.id)}
        <p class="noticia grave"><WarningIcon size={24} weight="bold" /><span>{n.texto}</span></p>
      {/each}
      {#if rotando}
        <p class="noticia" aria-live="polite">
          <MegaphoneIcon size={22} weight="bold" /><span>{rotando.texto}</span>
          {#if resto.length > 1}<span class="cuenta-noticias">{(girando % resto.length) + 1}/{resto.length}</span>{/if}
        </p>
      {/if}
    </section>
  {/if}

  {#if desactualizado || (error && ultimaActualizacion !== null)}
    <p class="aviso"><WifiSlashIcon size={20} weight="bold" />Desactualizado, última actualización hace {hace(ultimaActualizacion!, ahora)}</p>
  {:else if error}
    <p class="aviso">{error}</p>
  {/if}

  <div class="lista" role="table" aria-label="Próximos trenes">
    <div class="fila encabezados" role="row">
      <span role="columnheader">Hora</span>
      <span role="columnheader">Destino</span>
      <span role="columnheader" class="centro">Andén</span>
      <span role="columnheader">Estado</span>
      <span role="columnheader" class="derecha">Sale en</span>
    </div>
    {#each arribos as a (clave(a))}
      {@const demorado = a.demora_minutos !== null && a.demora_minutos > 0}
      <div class="fila sit-{claseSituacion(a, ahora)}" class:cancelado={a.situacion === "cancelado"} role="row">
        <span class="hora" role="cell">
          <span class:estimada={demorado}>{hora(a.momento)}</span>
          {#if demorado}<s aria-label="programado {hora(a.programada)}">{hora(a.programada)}</s>{/if}
        </span>
        <span class="destino" role="cell">
          <strong>{a.destino}</strong>
          <small>{a.ramal.nombre} · #{a.numero}</small>
        </span>
        <span class="anden centro" role="cell" class:cambio={a.anden !== null && a.anden_anterior !== null}>
          {a.anden ?? "–"}
          {#if a.anden && a.anden_anterior}<small>antes {a.anden_anterior}</small>{/if}
        </span>
        <span role="cell"><SituacionPill arribo={a} {ahora} compacta /></span>
        <span class="cuenta derecha" role="cell">{cuentaRegresiva(a.momento, ahora)}</span>
      </div>
    {:else}
      {#if ultimaActualizacion !== null}
        <p class="vacio">No hay próximos trenes informados.</p>
      {/if}
    {/each}
  </div>
</main>

<style>
  /* La escala sigue al tamaño de la ventana: se lee igual en una ventana chica que en
     un monitor entero. Los componentes compartidos toman estos tokens. */
  .tablero {
    --t-micro: clamp(12px, 1.15vw, 22px);
    --t-meta: clamp(13px, 1.3vw, 26px);
    --t-cuerpo: clamp(16px, 1.9vw, 38px);
    --t-titulo: clamp(26px, 3.2vw, 64px);
    min-height: 100vh;
    box-sizing: border-box;
    padding: clamp(16px, 2.5vw, 48px);
    padding-top: 0;
    display: flex;
    flex-direction: column;
    gap: clamp(10px, 1.4vw, 24px);
    background: var(--fondo);
    color: var(--texto);
    font-family: var(--fuente);
    overflow: hidden;
    height: 100vh;
  }
  .quieto,
  .quieto * {
    cursor: none;
  }

  .barra {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 8px;
    padding: 10px 0;
    min-height: 52px;
    transition: opacity var(--mov-ui);
  }
  .quieto .barra {
    opacity: 0;
    pointer-events: none;
  }
  .chips {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
    flex: 1;
    min-width: 0;
  }
  .chip,
  .boton,
  select {
    min-height: 36px;
    padding: 0 12px;
    border-radius: var(--radio-pill);
    border: 1px solid var(--borde-fuerte);
    background: var(--superficie);
    color: var(--texto);
    font: inherit;
    font-size: 14px;
    cursor: pointer;
  }
  .chip.activo {
    border-color: var(--acento);
    color: var(--acento);
  }
  .botones {
    display: flex;
    gap: 6px;
  }
  .boton {
    display: inline-grid;
    place-items: center;
    width: 36px;
    padding: 0;
  }
  @media (hover: hover) {
    .chip:hover,
    .boton:hover {
      background: var(--superficie-hover);
    }
  }
  .selector {
    flex-basis: 100%;
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
  }
  select {
    border-radius: var(--radio-s);
  }

  .cabecera {
    display: flex;
    flex-wrap: wrap;
    align-items: baseline;
    justify-content: space-between;
    gap: 4px 24px;
    border-bottom: 2px solid var(--borde-fuerte);
    padding-bottom: clamp(8px, 1vw, 16px);
  }
  h1 {
    margin: 0;
    font-family: var(--fuente-display);
    font-size: var(--t-titulo);
    font-weight: 700;
    line-height: 1.05;
  }
  .cabecera p {
    margin: 0;
    font-size: var(--t-meta);
    color: var(--texto-suave);
  }
  .reloj {
    margin-left: 12px;
    font-family: var(--fuente-display);
    font-size: var(--t-cuerpo);
    font-weight: 600;
    color: var(--texto);
    font-variant-numeric: tabular-nums;
  }

  .noticias {
    display: grid;
    gap: 8px;
  }
  .noticia {
    display: flex;
    align-items: center;
    gap: 12px;
    margin: 0;
    padding: clamp(8px, 0.9vw, 16px) clamp(12px, 1.2vw, 20px);
    border-radius: var(--radio-m);
    font-size: var(--t-meta);
    background: var(--superficie);
    border: 1px solid var(--borde);
  }
  .noticia :global(svg) {
    flex-shrink: 0;
    color: var(--texto-suave);
  }
  .noticia.grave {
    font-size: var(--t-cuerpo);
    font-weight: 600;
    background: color-mix(in srgb, var(--atencion) 14%, var(--superficie));
    border-color: color-mix(in srgb, var(--atencion) 40%, transparent);
  }
  .noticia.grave :global(svg) {
    color: var(--atencion);
  }
  .cuenta-noticias {
    margin-left: auto;
    color: var(--texto-tenue);
    font-variant-numeric: tabular-nums;
  }
  .aviso {
    display: flex;
    align-items: center;
    gap: 8px;
    margin: 0;
    font-size: var(--t-meta);
    color: var(--texto-suave);
  }

  .lista {
    flex: 1;
    min-height: 0;
    overflow: hidden;
  }
  .fila {
    /* Columnas en unidades del cuerpo, no en em: los encabezados usan letra más chica. */
    --u: var(--t-cuerpo);
    display: grid;
    grid-template-columns:
      calc(var(--u) * 4.4) minmax(0, 1fr) calc(var(--u) * 3.5) calc(var(--u) * 7.5)
      calc(var(--u) * 4.5);
    align-items: center;
    gap: 0 clamp(12px, 1.5vw, 32px);
    padding: clamp(8px, 1vw, 18px) clamp(8px, 0.8vw, 14px);
    border-bottom: 1px solid var(--borde);
    font-size: var(--t-cuerpo);
    position: relative;
  }
  /* Barra con el color de la Situación, como en el panel. */
  .fila:not(.encabezados)::before {
    content: "";
    position: absolute;
    left: 0;
    top: 22%;
    bottom: 22%;
    width: 4px;
    border-radius: 4px;
    background: var(--sit);
  }
  .encabezados {
    padding-top: 0;
    font-size: var(--t-micro);
    font-weight: 600;
    letter-spacing: 0.05em;
    text-transform: uppercase;
    color: var(--texto-tenue);
  }
  .hora {
    display: grid;
    font-family: var(--fuente-display);
    font-weight: 700;
    font-variant-numeric: tabular-nums;
  }
  .hora s {
    font-size: var(--t-meta);
    font-weight: 400;
    color: var(--texto-tenue);
  }
  .estimada {
    color: var(--sit-texto);
  }
  .destino {
    display: grid;
    min-width: 0;
  }
  .destino strong {
    font-weight: 650;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .destino small {
    font-size: var(--t-meta);
    color: var(--texto-tenue);
  }
  .anden {
    display: grid;
    font-family: var(--fuente-display);
    font-weight: 700;
    font-variant-numeric: tabular-nums;
  }
  .anden small {
    font-family: var(--fuente);
    font-size: var(--t-micro);
    font-weight: 500;
  }
  .anden.cambio {
    color: color-mix(in srgb, var(--atencion) 82%, var(--texto));
  }
  .cuenta {
    font-family: var(--fuente-display);
    font-weight: 700;
    font-variant-numeric: tabular-nums;
  }
  .sit-arribando .cuenta,
  .sit-en_anden .cuenta {
    color: var(--sit-texto);
  }
  .cancelado .hora,
  .cancelado .destino,
  .cancelado .cuenta {
    text-decoration: line-through;
    color: var(--texto-tenue);
  }
  .centro {
    text-align: center;
    justify-items: center;
  }
  .derecha {
    text-align: right;
  }
  .vacio {
    margin: 2em 0;
    font-size: var(--t-cuerpo);
    color: var(--texto-suave);
  }
  /* Ventanas angostas: el Estado queda en ícono y color, como en las filas del panel. */
  @media (max-width: 900px) {
    .fila {
      grid-template-columns:
        calc(var(--u) * 3.6) minmax(0, 1fr) calc(var(--u) * 2.6) calc(var(--u) * 2.2)
        calc(var(--u) * 4);
    }
    .fila :global(.pill .texto) {
      display: none;
    }
  }
</style>

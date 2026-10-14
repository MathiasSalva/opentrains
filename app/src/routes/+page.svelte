<script lang="ts">
  import { onMount } from "svelte";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import {
    api,
    coordenadas,
    describirError,
    esFuenteCambiada,
    mismoFavorito,
    type Arribo,
    type Coordenadas,
    type Estacion,
    type EstacionCercana,
    type Favorito,
    type Instalaciones,
    type Linea,
    type Preferencias,
    type Ramal,
  } from "$lib/api";
  import { fade, fly, slide } from "svelte/transition";
  import StarIcon from "phosphor-svelte/lib/StarIcon";
  import ArrowClockwiseIcon from "phosphor-svelte/lib/ArrowClockwiseIcon";
  import CaretRightIcon from "phosphor-svelte/lib/CaretRightIcon";
  import WarningIcon from "phosphor-svelte/lib/WarningIcon";
  import WifiSlashIcon from "phosphor-svelte/lib/WifiSlashIcon";
  import MapPinIcon from "phosphor-svelte/lib/MapPinIcon";
  import { agruparPorRamal, clave, proximo } from "$lib/arribos";
  import { hace } from "$lib/formato";
  import { duracion, EASE } from "$lib/movimiento";
  import ProximoArribo from "$lib/components/ProximoArribo.svelte";
  import GrupoRamal from "$lib/components/GrupoRamal.svelte";
  import SkeletonArribos from "$lib/components/SkeletonArribos.svelte";
  import { recargarSeguimientos, seguimientos } from "$lib/seguimientos.svelte";
  import { esEscritorio } from "$lib/plataforma";

  const REFRESCO_MS = 45_000;

  let preferencias = $state<Preferencias>({ favoritos: [], ultimo_abierto: null });

  let lineas = $state<Linea[]>([]);
  let ramales = $state<Ramal[]>([]);
  let estaciones = $state<Estacion[]>([]);
  let lineaId = $state<number | null>(null);
  let ramalId = $state<number | null>(null);
  let estacionId = $state<number | null>(null);
  // Por defecto se ven todos los Ramales de la Estación, agrupados.
  let soloEsteRamal = $state(false);
  let errorCatalogo = $state<string | null>(null);

  let actual = $state<Favorito | null>(null);
  let arribos = $state<Arribo[]>([]);
  let ultimaActualizacion = $state<number | null>(null);
  let errorArribos = $state<string | null>(null);
  let fuenteCambiada = $state(false);
  let cargando = $state(false);
  // Cada refresco lleva un número: si mientras tanto se abrió otra estación o se lanzó
  // otro refresco, la respuesta vieja se descarta.
  let ultimaConsulta = 0;

  let instalaciones = $state<Instalaciones | null | undefined>(undefined);
  let verInstalaciones = $state(false);

  let ahora = $state(Date.now());

  // Ubicación: `null` mientras no se sabe, `false` si Windows no la da.
  let hayUbicacion = $state<boolean | null>(null);
  let versionUbicacion = $state(0);
  let cercanas = $state<EstacionCercana[]>([]);
  /** Distancia ya formateada a cada Estación visible, por id. */
  let distancias = $state<Record<number, string>>({});

  const esFavorito = $derived(
    actual !== null && preferencias.favoritos.some((f) => mismoFavorito(f, actual!)),
  );
  const destacado = $derived(proximo(arribos));
  const grupos = $derived(agruparPorRamal(arribos.filter((a) => a !== destacado)));
  const desactualizado = $derived(
    errorArribos !== null && ultimaActualizacion !== null,
  );

  async function cargarLineas() {
    try {
      lineas = await api.lineas();
      errorCatalogo = null;
    } catch (e) {
      errorCatalogo = describirError(e);
    }
  }

  async function elegirLinea() {
    ramales = [];
    estaciones = [];
    ramalId = null;
    estacionId = null;
    if (lineaId === null) return;
    try {
      ramales = await api.ramales(lineaId);
      errorCatalogo = null;
    } catch (e) {
      errorCatalogo = describirError(e);
    }
  }

  async function elegirRamal() {
    estaciones = [];
    estacionId = null;
    if (ramalId === null) return;
    try {
      estaciones = await api.estaciones(ramalId);
      errorCatalogo = null;
    } catch (e) {
      errorCatalogo = describirError(e);
    }
  }

  function elegirEstacion() {
    const estacion = estaciones.find((e) => e.id === estacionId);
    const ramal = ramales.find((r) => r.id === ramalId) ?? null;
    if (!estacion) return;
    abrir({
      estacion: estacion.id,
      estacion_nombre: estacion.nombre,
      ramal: soloEsteRamal ? ramal : null,
      coordenadas: coordenadas(estacion),
    });
  }

  async function abrir(favorito: Favorito) {
    actual = favorito;
    seguimientos.estacion = { id: favorito.estacion, nombre: favorito.estacion_nombre };
    arribos = [];
    ultimaActualizacion = null;
    errorArribos = null;
    instalaciones = undefined;
    verInstalaciones = false;
    try {
      preferencias = await api.abrir($state.snapshot(favorito));
      seguimientos.anticipaciones = preferencias.anticipaciones ?? {};
    } catch (e) {
      // Recordar el último abierto es una comodidad: no bloquea la consulta.
      console.warn(describirError(e));
    }
    await refrescar();
  }

  async function refrescar() {
    if (!actual) return;
    const consulta = ++ultimaConsulta;
    cargando = true;
    try {
      const nuevos = await api.arribos($state.snapshot(actual));
      if (consulta !== ultimaConsulta) return;
      arribos = nuevos;
      ultimaActualizacion = Date.now();
      errorArribos = null;
      fuenteCambiada = false;
      recargarSeguimientos();
    } catch (e) {
      if (consulta !== ultimaConsulta) return;
      errorArribos = describirError(e);
      fuenteCambiada = esFuenteCambiada(e);
    } finally {
      if (consulta === ultimaConsulta) cargando = false;
    }
  }

  async function alternarFavorito() {
    if (!actual) return;
    try {
      preferencias = esFavorito
        ? await api.quitarFavorito(actual)
        : await api.agregarFavorito(actual);
    } catch (e) {
      errorArribos = describirError(e);
    }
  }

  async function alternarInstalaciones() {
    verInstalaciones = !verInstalaciones;
    if (verInstalaciones && instalaciones === undefined && actual) {
      try {
        instalaciones = await api.instalaciones(actual.estacion);
      } catch {
        instalaciones = null;
      }
    }
  }

  async function ubicar() {
    try {
      hayUbicacion = await api.ubicar();
    } catch {
      hayUbicacion = false;
    }
    versionUbicacion++;
    if (!hayUbicacion) return;
    try {
      cercanas = (await api.cercanas()) ?? [];
    } catch {
      // Sin conexión: quedan las últimas cercanas conocidas.
    }
  }

  // Distancias a lo que se ve: Estaciones del selector, Favoritos y la Estación abierta.
  $effect(() => {
    void versionUbicacion;
    if (!hayUbicacion) {
      distancias = {};
      return;
    }
    const destinos: [number, Coordenadas][] = [
      ...estaciones.map((e): [number, Coordenadas] => [e.id, coordenadas(e)]),
      ...[...preferencias.favoritos, ...(actual ? [actual] : [])]
        .filter((f) => f.coordenadas)
        .map((f): [number, Coordenadas] => [f.estacion, f.coordenadas!]),
    ];
    api
      .distancias($state.snapshot(destinos.map(([, c]) => c)))
      .then((textos) => {
        if (!textos) return;
        distancias = Object.fromEntries(destinos.map(([id], i) => [id, textos[i]]));
      })
      .catch(() => {});
  });

  function abrirCercana({ estacion }: EstacionCercana) {
    abrir({
      estacion: estacion.id,
      estacion_nombre: estacion.nombre,
      ramal: null,
      coordenadas: coordenadas(estacion),
    });
  }

  function etiqueta(f: Favorito): string {
    return f.ramal ? `${f.estacion_nombre} · ${f.ramal.nombre}` : f.estacion_nombre;
  }

  onMount(() => {
    (async () => {
      ubicar();
      try {
        preferencias = await api.preferencias();
        seguimientos.anticipaciones = preferencias.anticipaciones ?? {};
      } catch (e) {
        errorCatalogo = describirError(e);
      }
      await cargarLineas();
      if (preferencias.ultimo_abierto) await abrir(preferencias.ultimo_abierto);
    })();

    // Refresco cada 45 s también con el panel oculto: así el ícono de la bandeja muestra
    // siempre el próximo tren. Con la ventana oculta, WebView2 puede espaciar los timers
    // hasta 1 por minuto; al volver a mostrarse se refresca si los datos quedaron viejos.
    // En móvil no hay bandeja: con la app en segundo plano no se refresca (batería y datos).
    const refresco = setInterval(() => {
      if (!esEscritorio && document.visibilityState === "hidden") return;
      refrescar();
      ubicar();
    }, REFRESCO_MS);
    const reloj = setInterval(() => (ahora = Date.now()), 1_000);
    const alMostrarse = () => {
      ahora = Date.now();
      if (ultimaActualizacion === null || ahora - ultimaActualizacion >= REFRESCO_MS) {
        refrescar();
        ubicar();
      }
    };
    const alCambiarVisibilidad = () => document.visibilityState === "visible" && alMostrarse();
    document.addEventListener("visibilitychange", alCambiarVisibilidad);
    // Solo el panel de escritorio: el foco marca cuándo se vuelve a mostrar y, como los
    // paneles de Windows, Escape lo oculta. En móvil el "atrás" del sistema ya minimiza.
    let dejarDeEscuchar: Promise<() => void> | undefined;
    let alTeclear: ((e: KeyboardEvent) => void) | undefined;
    if (esEscritorio) {
      const ventana = getCurrentWindow();
      dejarDeEscuchar = ventana.onFocusChanged(({ payload: enfocada }) => enfocada && alMostrarse());
      alTeclear = (e) => e.key === "Escape" && ventana.hide();
      document.addEventListener("keydown", alTeclear);
    }
    return () => {
      clearInterval(refresco);
      clearInterval(reloj);
      document.removeEventListener("visibilitychange", alCambiarVisibilidad);
      if (alTeclear) document.removeEventListener("keydown", alTeclear);
      dejarDeEscuchar?.then((fn) => fn());
    };
  });
</script>

<main>
  <header class="barra">
    <h1>OpenTrains</h1>
  </header>

  {#if preferencias.favoritos.length > 0}
    <nav class="favoritos" aria-label="Favoritos">
      {#each preferencias.favoritos as favorito}
        <button
          class="chip"
          class:activo={actual !== null && mismoFavorito(favorito, actual)}
          onclick={() => abrir(favorito)}
        >
          <StarIcon size={13} weight="fill" />
          {etiqueta(favorito)}
          {#if distancias[favorito.estacion]}
            <span class="distancia">{distancias[favorito.estacion]}</span>
          {/if}
        </button>
      {/each}
    </nav>
  {/if}

  {#if hayUbicacion === false}
    <p class="cerca-vacio">
      <MapPinIcon size={14} />
      Activá la ubicación de Windows para ver estaciones cercanas
    </p>
  {:else if cercanas.length > 0}
    <nav class="cerca" aria-label="Estaciones cercanas">
      <span class="cerca-rotulo">Cerca tuyo</span>
      {#each cercanas as cercana (cercana.estacion.id)}
        <button
          class="chip"
          class:activo={actual !== null && actual.estacion === cercana.estacion.id && !actual.ramal}
          onclick={() => abrirCercana(cercana)}
        >
          <MapPinIcon size={13} weight="fill" />
          {cercana.estacion.nombre}
          <span class="distancia">{cercana.distancia}</span>
        </button>
      {/each}
    </nav>
  {/if}

  <section class="selector panel">
    <label>
      <span>Línea</span>
      <select bind:value={lineaId} onchange={elegirLinea}>
        <option value={null}>Elegí una línea</option>
        {#each lineas as linea}
          <option value={linea.id}>{linea.nombre}</option>
        {/each}
      </select>
    </label>
    <div class="par">
      <label>
        <span>Ramal</span>
        <select bind:value={ramalId} onchange={elegirRamal} disabled={ramales.length === 0}>
          <option value={null}>Elegí un ramal</option>
          {#each ramales as ramal}
            <option value={ramal.id}>{ramal.nombre}</option>
          {/each}
        </select>
      </label>
      <label>
        <span>Estación</span>
        <select
          bind:value={estacionId}
          onchange={elegirEstacion}
          disabled={estaciones.length === 0}
        >
          <option value={null}>Elegí una estación</option>
          {#each estaciones as estacion}
            <option value={estacion.id}
              >{estacion.nombre}{distancias[estacion.id] ? ` · ${distancias[estacion.id]}` : ""}</option
            >
          {/each}
        </select>
      </label>
    </div>
    <label class="check">
      <input type="checkbox" bind:checked={soloEsteRamal} onchange={elegirEstacion} />
      <span>Solo el ramal elegido</span>
    </label>
    {#if errorCatalogo}
      <p class="aviso error">
        <WarningIcon size={16} weight="bold" />
        <span>{errorCatalogo}</span>
        <button class="enlace" onclick={cargarLineas}>Reintentar</button>
      </p>
    {/if}
  </section>

  {#if actual}
    <section class="estacion">
      <div class="titulo">
        <div>
          <h2>{actual.estacion_nombre}</h2>
          <p class="sub">
            <span>{actual.ramal ? actual.ramal.nombre : "Todos los ramales"}</span>
            {#if distancias[actual.estacion]}
              <span>{distancias[actual.estacion]}</span>
            {/if}
            {#if ultimaActualizacion !== null && !desactualizado}
              <span class="sello">Actualizado hace {hace(ultimaActualizacion, ahora)}</span>
            {/if}
          </p>
        </div>
        <div class="acciones">
          <button
            class="icono"
            class:marcado={esFavorito}
            title={esFavorito ? "Quitar de favoritos" : "Agregar a favoritos"}
            aria-pressed={esFavorito}
            onclick={alternarFavorito}
          >
            <StarIcon size={20} weight={esFavorito ? "fill" : "regular"} />
          </button>
          <button
            class="icono"
            class:girando={cargando}
            title="Actualizar"
            disabled={cargando}
            onclick={refrescar}
          >
            <ArrowClockwiseIcon size={20} weight="bold" />
          </button>
        </div>
      </div>

      {#if fuenteCambiada}
        <p class="aviso error" transition:slide={{ duration: duracion(200) }}>
          <WarningIcon size={16} weight="bold" /><span>{errorArribos}</span>
        </p>
      {:else if desactualizado}
        <p class="aviso" transition:slide={{ duration: duracion(200) }}>
          <WifiSlashIcon size={16} weight="bold" />
          <span>Desactualizado, última actualización hace {hace(ultimaActualizacion!, ahora)}</span>
        </p>
      {:else if errorArribos}
        <p class="aviso error" transition:slide={{ duration: duracion(200) }}>
          <WarningIcon size={16} weight="bold" /><span>{errorArribos}</span>
        </p>
      {/if}

      {#if ultimaActualizacion === null && cargando}
        <SkeletonArribos />
      {:else if ultimaActualizacion !== null && arribos.length === 0}
        <div class="vacio" in:fade={{ duration: duracion(200) }}>
          <strong>No hay próximos trenes informados</strong>
          <span>Probá con todos los ramales o volvé a consultar en unos minutos.</span>
        </div>
      {:else}
        {#if destacado}
          {#key clave(destacado)}
            <div in:fly={{ y: 12, duration: duracion(380), easing: EASE }}>
              <ProximoArribo arribo={destacado} {ahora} />
            </div>
          {/key}
        {/if}
        {#each grupos as grupo (grupo.ramal.id)}
          <div in:fly={{ y: 12, duration: duracion(320), easing: EASE }}>
            <GrupoRamal {grupo} {ahora} />
          </div>
        {/each}
      {/if}
    </section>

    <section class="panel instalaciones">
      <button class="desplegar" aria-expanded={verInstalaciones} onclick={alternarInstalaciones}>
        <span class="caret" class:abierto={verInstalaciones}>
          <CaretRightIcon size={14} weight="bold" />
        </span>
        Instalaciones de la estación
      </button>
      {#if verInstalaciones}
        <div transition:slide={{ duration: duracion(220) }}>
          {#if instalaciones === undefined}
            <p class="tenue">Cargando...</p>
          {:else if instalaciones === null}
            <p class="tenue">Sin información para esta estación.</p>
          {:else}
            <ul class="servicios">
              <li class:no={!instalaciones.ascensor}>Ascensor</li>
              <li class:no={!instalaciones.rampa}>Rampa</li>
              <li class:no={!instalaciones.banos}>Baños</li>
              <li class:no={!instalaciones.wifi}>Wifi</li>
              <li class:no={!instalaciones.terminal_sube}>Carga SUBE</li>
            </ul>
            <dl class="fichas">
              {#if instalaciones.boleteria}<dt>Boletería</dt><dd>{instalaciones.boleteria}</dd>{/if}
              {#if instalaciones.direccion}<dt>Dirección</dt><dd>{instalaciones.direccion}</dd>{/if}
              {#if instalaciones.conexiones}<dt>Conexiones</dt><dd>{instalaciones.conexiones}</dd>{/if}
              {#if instalaciones.colectivos}<dt>Colectivos</dt><dd>{instalaciones.colectivos}</dd>{/if}
            </dl>
          {/if}
        </div>
      {/if}
    </section>
  {/if}
</main>

<style>
  :global(:root) {
    color-scheme: light dark;
    /* Fuente del sistema de Windows: la app se siente nativa y no carga fuentes. */
    font-family: "Segoe UI Variable Text", "Segoe UI", system-ui, sans-serif;
    font-size: 15px;
    --fondo: #f3f4f6;
    --superficie: #fdfdfe;
    --texto: #17191e;
    --texto-suave: #5e6573;
    --borde: #e2e5ea;
    --acento: #1f5fd1;
    /* Situación del Arribo: colores semánticos, no decorativos. */
    --en_anden: #13803f;
    --en_camino: #0e7490;
    --demorado: #b45309;
    --cancelado: #c0263f;
    --programado: #5e6573;
    --desconocido: #64708a;
  }
  @media (prefers-color-scheme: dark) {
    :global(:root) {
      --fondo: #0e1014;
      --superficie: #161a20;
      --texto: #eceef2;
      --texto-suave: #959cab;
      --borde: #262b34;
      --acento: #79a7ff;
      --en_anden: #34d17a;
      --en_camino: #38bdd8;
      --demorado: #fbb13c;
      --cancelado: #fb6f84;
      --programado: #9aa1ae;
      --desconocido: #a3adc2;
    }
  }
  :global(.sit-en_anden),
  :global(.sit-arribando) {
    --sit: var(--en_anden);
  }
  :global(.sit-en_camino) {
    --sit: var(--en_camino);
  }
  :global(.sit-demorado) {
    --sit: var(--demorado);
  }
  :global(.sit-cancelado) {
    --sit: var(--cancelado);
  }
  :global(.sit-programado) {
    --sit: var(--programado);
  }
  :global(.sit-desconocido) {
    --sit: var(--desconocido);
  }
  :global(body) {
    margin: 0;
    background: var(--fondo);
    color: var(--texto);
    -webkit-font-smoothing: antialiased;
  }
  :global(:focus-visible) {
    outline: 2px solid var(--acento);
    outline-offset: 2px;
  }

  main {
    /* En Android 15+ la webview va de borde a borde: se respetan la barra de estado y la de gestos. */
    padding: calc(14px + env(safe-area-inset-top)) calc(14px + env(safe-area-inset-right))
      calc(24px + env(safe-area-inset-bottom)) calc(14px + env(safe-area-inset-left));
    display: grid;
    gap: 12px;
  }
  .barra h1 {
    margin: 0;
    font-size: 1.15rem;
    letter-spacing: -0.01em;
  }
  .panel {
    background: var(--superficie);
    border: 1px solid var(--borde);
    border-radius: 14px;
    padding: 12px;
  }

  .favoritos {
    display: flex;
    gap: 6px;
    overflow-x: auto;
    scrollbar-width: none;
  }
  .chip {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    flex-shrink: 0;
    border: 1px solid var(--borde);
    background: var(--superficie);
    color: var(--texto);
    border-radius: 999px;
    padding: 5px 11px;
    cursor: pointer;
    font: inherit;
    font-size: 0.82rem;
    transition:
      border-color 0.2s ease,
      color 0.2s ease,
      transform 0.15s ease;
  }
  .chip :global(svg) {
    color: var(--demorado);
  }
  .chip:hover {
    border-color: var(--acento);
  }
  .chip:active {
    transform: scale(0.97);
  }
  .chip.activo {
    border-color: var(--acento);
    color: var(--acento);
  }
  .distancia {
    color: var(--texto-suave);
    font-variant-numeric: tabular-nums;
  }

  .cerca {
    display: flex;
    align-items: center;
    gap: 6px;
    overflow-x: auto;
    scrollbar-width: none;
  }
  .cerca .chip :global(svg) {
    color: var(--acento);
  }
  .cerca-rotulo {
    flex-shrink: 0;
    font-size: 0.72rem;
    font-weight: 600;
    letter-spacing: 0.03em;
    text-transform: uppercase;
    color: var(--texto-suave);
  }
  .cerca-vacio {
    display: flex;
    align-items: center;
    gap: 6px;
    margin: 0;
    font-size: 0.78rem;
    color: var(--texto-suave);
  }

  .selector {
    display: grid;
    gap: 10px;
  }
  .par {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 8px;
  }
  label {
    display: grid;
    gap: 4px;
    font-size: 0.78rem;
    font-weight: 600;
    color: var(--texto-suave);
    min-width: 0;
  }
  label.check {
    display: flex;
    align-items: center;
    gap: 7px;
    font-weight: 500;
  }
  select {
    font: inherit;
    font-size: 0.9rem;
    padding: 7px 8px;
    border-radius: 8px;
    border: 1px solid var(--borde);
    background: var(--fondo);
    color: var(--texto);
    min-width: 0;
  }
  select:disabled {
    opacity: 0.55;
  }
  input[type="checkbox"] {
    accent-color: var(--acento);
  }

  .estacion {
    display: grid;
    gap: 10px;
  }
  .titulo {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    padding: 2px 2px 0;
  }
  h2 {
    margin: 0;
    font-size: 1.25rem;
    letter-spacing: -0.01em;
  }
  .sub {
    display: flex;
    flex-wrap: wrap;
    gap: 2px 10px;
    margin: 2px 0 0;
    font-size: 0.8rem;
    color: var(--texto-suave);
  }
  .sello {
    font-variant-numeric: tabular-nums;
  }
  .acciones {
    display: flex;
    gap: 2px;
  }
  button.icono {
    display: grid;
    place-items: center;
    width: 36px;
    height: 36px;
    border-radius: 10px;
    background: none;
    border: none;
    color: var(--texto-suave);
    cursor: pointer;
    transition:
      background-color 0.2s ease,
      color 0.2s ease,
      transform 0.15s ease;
  }
  button.icono:hover {
    background: var(--borde);
    color: var(--texto);
  }
  button.icono:active {
    transform: scale(0.94);
  }
  button.icono:disabled {
    cursor: default;
  }
  button.icono.marcado {
    color: var(--demorado);
  }
  @media (prefers-reduced-motion: no-preference) {
    button.girando :global(svg) {
      animation: girar 0.9s linear infinite;
    }
  }
  @keyframes girar {
    to {
      transform: rotate(360deg);
    }
  }

  .aviso {
    display: flex;
    align-items: center;
    gap: 8px;
    margin: 0;
    padding: 8px 10px;
    border-radius: 10px;
    font-size: 0.85rem;
    color: var(--demorado);
    background: color-mix(in srgb, var(--demorado) 13%, transparent);
  }
  .aviso span {
    color: var(--texto);
  }
  .aviso.error {
    color: var(--cancelado);
    background: color-mix(in srgb, var(--cancelado) 13%, transparent);
  }
  .aviso :global(svg) {
    flex-shrink: 0;
  }
  button.enlace {
    margin-left: auto;
    background: none;
    border: none;
    padding: 0;
    color: var(--acento);
    cursor: pointer;
    font: inherit;
    font-weight: 600;
  }

  .vacio {
    display: grid;
    gap: 4px;
    padding: 22px 16px;
    text-align: center;
    border: 1px dashed var(--borde);
    border-radius: 14px;
  }
  .vacio span {
    font-size: 0.85rem;
    color: var(--texto-suave);
  }

  .instalaciones {
    padding: 4px 12px;
  }
  .desplegar {
    display: flex;
    align-items: center;
    gap: 6px;
    width: 100%;
    padding: 8px 0;
    background: none;
    border: none;
    color: var(--texto);
    font: inherit;
    font-weight: 600;
    cursor: pointer;
  }
  .caret {
    display: grid;
    color: var(--texto-suave);
    transition: transform 0.2s ease;
  }
  .caret.abierto {
    transform: rotate(90deg);
  }
  .tenue {
    margin: 0 0 10px;
    font-size: 0.85rem;
    color: var(--texto-suave);
  }
  .servicios {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
    list-style: none;
    margin: 0 0 10px;
    padding: 0;
  }
  .servicios li {
    font-size: 0.78rem;
    font-weight: 600;
    padding: 3px 10px;
    border-radius: 999px;
    color: var(--en_anden);
    background: color-mix(in srgb, var(--en_anden) 14%, transparent);
  }
  .servicios li.no {
    color: var(--texto-suave);
    background: none;
    border: 1px solid var(--borde);
    text-decoration: line-through;
  }
  .fichas {
    display: grid;
    grid-template-columns: auto 1fr;
    gap: 6px 12px;
    margin: 0 0 12px;
    font-size: 0.83rem;
  }
  .fichas dt {
    color: var(--texto-suave);
  }
  .fichas dd {
    margin: 0;
  }
</style>

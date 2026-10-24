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
    type Noticia,
    type Preferencias,
    type Ramal,
  } from "$lib/api";
  import { fade, fly, slide } from "svelte/transition";
  import StarIcon from "phosphor-svelte/lib/StarIcon";
  import CheckIcon from "phosphor-svelte/lib/CheckIcon";
  import MinusIcon from "phosphor-svelte/lib/MinusIcon";
  import XIcon from "phosphor-svelte/lib/XIcon";
  import MagnifyingGlassIcon from "phosphor-svelte/lib/MagnifyingGlassIcon";
  import ArrowClockwiseIcon from "phosphor-svelte/lib/ArrowClockwiseIcon";
  import CaretRightIcon from "phosphor-svelte/lib/CaretRightIcon";
  import WarningIcon from "phosphor-svelte/lib/WarningIcon";
  import WifiSlashIcon from "phosphor-svelte/lib/WifiSlashIcon";
  import MapPinIcon from "phosphor-svelte/lib/MapPinIcon";
  import MonitorIcon from "phosphor-svelte/lib/MonitorIcon";
  import { agruparPorRamal, clave, proximo } from "$lib/arribos";
  import { hace } from "$lib/formato";
  import { duracion, EASE } from "$lib/movimiento";
  import ProximoArribo from "$lib/components/ProximoArribo.svelte";
  import GrupoRamal from "$lib/components/GrupoRamal.svelte";
  import FilaChips from "$lib/components/FilaChips.svelte";
  import Hoja from "$lib/components/Hoja.svelte";
  import DetalleServicio from "$lib/components/DetalleServicio.svelte";
  import NoticiasEstacion from "$lib/components/NoticiasEstacion.svelte";
  import { detalle } from "$lib/detalle.svelte";
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
  // Con una Estación abierta el selector se pliega a una fila: lo primero es el próximo tren.
  let selectorAbierto = $state(false);

  let actual = $state<Favorito | null>(null);
  let arribos = $state<Arribo[]>([]);
  let noticias = $state<Noticia[]>([]);
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

  const verSelector = $derived(actual === null || selectorAbierto || errorCatalogo !== null);
  const esFavorito = $derived(
    actual !== null && preferencias.favoritos.some((f) => mismoFavorito(f, actual!)),
  );
  const destacado = $derived(proximo(arribos));
  // El Servicio de la hoja se busca en cada refresco: si ya pasó, la hoja se cierra.
  const enDetalle = $derived(arribos.find((a) => clave(a) === detalle.clave) ?? null);
  const grupos = $derived(agruparPorRamal(arribos.filter((a) => a !== destacado)));
  // Un solo Ramal que ya se nombra arriba (Favorito acotado o el del próximo tren): sin encabezado.
  const ramalYaNombrado = $derived(
    grupos.length === 1 &&
      (actual?.ramal?.id === grupos[0].ramal.id || destacado?.ramal.id === grupos[0].ramal.id),
  );
  const verAvisoUbicacion = $derived(
    hayUbicacion === false && !preferencias.aviso_ubicacion_oculto,
  );
  // La cabecera de la Estación queda fija arriba; al despegarse muestra una línea.
  let cabeceraPegada = $state(false);
  let centinela = $state<HTMLElement>();
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
    selectorAbierto = false;
    abrir({
      estacion: estacion.id,
      estacion_nombre: estacion.nombre,
      ramal: soloEsteRamal ? ramal : null,
      coordenadas: coordenadas(estacion),
      // Catálogos guardados antes de saber los Ramales de cada Estación: al menos este.
      ramales: estacion.ramales?.length ? estacion.ramales : ramal ? [ramal.id] : [],
    });
  }

  async function abrir(favorito: Favorito) {
    actual = favorito;
    detalle.clave = null;
    seguimientos.estacion = { id: favorito.estacion, nombre: favorito.estacion_nombre };
    arribos = [];
    noticias = [];
    ultimaActualizacion = null;
    errorArribos = null;
    instalaciones = undefined;
    verInstalaciones = false;
    try {
      preferencias = await api.abrir($state.snapshot(favorito));
      seguimientos.anticipaciones = preferencias.anticipaciones ?? {};
      // El backend completa coordenadas y Ramales de Favoritos viejos.
      const completo = preferencias.ultimo_abierto;
      if (completo && actual && mismoFavorito(completo, actual)) actual = completo;
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
    refrescarNoticias(consulta);
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

  // Las Noticias no bloquean ni ensucian los Arribos: si fallan, quedan las últimas.
  async function refrescarNoticias(consulta: number) {
    if (!actual) return;
    try {
      const nuevas = await api.noticias($state.snapshot(actual));
      if (consulta === ultimaConsulta) noticias = nuevas;
    } catch (e) {
      console.warn(describirError(e));
    }
  }

  async function descartarNoticia(id: number) {
    noticias = noticias.filter((n) => n.id !== id);
    try {
      preferencias = await api.descartarNoticia(id);
    } catch (e) {
      console.warn(describirError(e));
    }
  }

  async function abrirTablero() {
    if (!actual) return;
    try {
      await api.abrirTablero($state.snapshot(actual));
    } catch (e) {
      errorArribos = describirError(e);
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
      ramales: estacion.ramales ?? [],
    });
  }

  async function ocultarAvisoUbicacion() {
    try {
      preferencias = await api.ocultarAvisoUbicacion();
    } catch (e) {
      console.warn(describirError(e));
    }
  }

  async function activarUbicacion() {
    // En Windows abre la configuración; en el celular, volver a consultar pide el permiso.
    if (esEscritorio) await api.abrirAjustesUbicacion();
    else await ubicar();
  }

  $effect(() => {
    if (!centinela) return;
    const observador = new IntersectionObserver(
      ([entrada]) => (cabeceraPegada = !entrada.isIntersecting),
      { rootMargin: "-1px 0px 0px 0px" },
    );
    observador.observe(centinela);
    return () => observador.disconnect();
  });

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
      // Escape cierra primero la capa más interna (el menú de una campana lo marca como
      // usado), después el selector desplegado y recién entonces oculta el panel.
      alTeclear = (e) => {
        if (e.key !== "Escape" || e.defaultPrevented) return;
        if (detalle.clave) detalle.clave = null;
        else if (selectorAbierto && actual) selectorAbierto = false;
        else ventana.hide();
      };
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
  <!-- La ventana no tiene marco: el título queda para lectores de pantalla. -->
  <h1 class="solo-lector">OpenTrains</h1>

  {#if preferencias.favoritos.length > 0}
    <FilaChips etiqueta="Favoritos">
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
    </FilaChips>
  {/if}

  {#if cercanas.length > 0 || verAvisoUbicacion}
    <FilaChips etiqueta="Estaciones cercanas">
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
      {#if verAvisoUbicacion}
        <span class="chip-aviso">
          <button class="chip" onclick={activarUbicacion}>
            <MapPinIcon size={13} />
            Activar ubicación para ver estaciones cercanas
          </button>
          <button
            class="boton-icono cerrar"
            aria-label="No mostrar más el aviso de ubicación"
            title="No mostrar más"
            onclick={ocultarAvisoUbicacion}
          >
            <XIcon size={13} weight="bold" />
          </button>
        </span>
      {/if}
    </FilaChips>
  {/if}

  {#if verSelector}
    <section class="selector panel" transition:slide={{ duration: duracion(220), easing: EASE }}>
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
      <div class="pie-selector">
        <label class="check">
          <input type="checkbox" bind:checked={soloEsteRamal} onchange={elegirEstacion} />
          <span>Solo el ramal elegido</span>
        </label>
        {#if actual && selectorAbierto}
          <button class="enlace" onclick={() => (selectorAbierto = false)}>Listo</button>
        {/if}
      </div>
      {#if errorCatalogo}
        <p class="aviso error">
          <WarningIcon size={16} weight="bold" />
          <span>{errorCatalogo}</span>
          <button class="enlace" onclick={cargarLineas}>Reintentar</button>
        </p>
      {/if}
    </section>
  {:else}
    <!-- Selector plegado: una fila de acciones (acá entra después el filtro "→ Destino"). -->
    <div class="selector-plegado">
      <button class="buscar" onclick={() => (selectorAbierto = true)} aria-expanded="false">
        <MagnifyingGlassIcon size={15} weight="bold" />
        Buscar otra estación
      </button>
    </div>
  {/if}

  {#if actual}
    <section class="estacion">
      <div class="centinela" bind:this={centinela} aria-hidden="true"></div>
      <div class="cabecera" class:pegada={cabeceraPegada}>
        <div class="titulo">
          <div class="nombre">
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
              class="boton-icono"
              class:marcado={esFavorito}
              aria-label={esFavorito ? "Quitar de favoritos" : "Agregar a favoritos"}
              title={esFavorito ? "Quitar de favoritos" : "Agregar a favoritos"}
              aria-pressed={esFavorito}
              onclick={alternarFavorito}
            >
              <StarIcon size={20} weight={esFavorito ? "fill" : "regular"} />
            </button>
            {#if esEscritorio}
              <button
                class="boton-icono"
                aria-label="Abrir el tablero de esta estación"
                title="Abrir tablero"
                onclick={abrirTablero}
              >
                <MonitorIcon size={20} weight="bold" />
              </button>
            {/if}
            <button
              class="boton-icono"
              class:girando={cargando}
              aria-label="Actualizar los trenes"
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
      </div>

      {#if noticias.length > 0}
        <div class="noticias">
          <NoticiasEstacion {noticias} descartar={descartarNoticia} />
        </div>
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
            <GrupoRamal {grupo} {ahora} encabezado={!ramalYaNombrado} />
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
              {#each [["Ascensor", instalaciones.ascensor], ["Rampa", instalaciones.rampa], ["Baños", instalaciones.banos], ["Wifi", instalaciones.wifi], ["Carga SUBE", instalaciones.terminal_sube]] as const as [nombre, hay]}
                <li class:no={!hay}>
                  {#if hay}<CheckIcon size={12} weight="bold" />{:else}<MinusIcon size={12} weight="bold" />{/if}
                  {nombre}<span class="solo-lector">{hay ? ": sí" : ": no"}</span>
                </li>
              {/each}
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
  {#if enDetalle}
    <Hoja titulo="a {enDetalle.destino}" alCerrar={() => (detalle.clave = null)}>
      <DetalleServicio arribo={enDetalle} {ahora} />
    </Hoja>
  {/if}
</main>

<style>
  main {
    /* En Android 15+ la webview va de borde a borde: se respetan la barra de estado y la de gestos. */
    padding: calc(14px + env(safe-area-inset-top)) calc(14px + env(safe-area-inset-right))
      calc(24px + env(safe-area-inset-bottom)) calc(14px + env(safe-area-inset-left));
    display: grid;
    gap: 10px;
  }
  /* Debajo de la barra de estado del celular nada se ve pasar al hacer scroll. */
  main::before {
    content: "";
    position: fixed;
    inset: 0 0 auto;
    height: env(safe-area-inset-top);
    background: var(--fondo);
    z-index: 6;
  }
  .panel {
    background: var(--superficie);
    border: 1px solid var(--borde);
    border-radius: var(--radio-l);
    padding: 12px;
  }

  .chip-aviso {
    display: inline-flex;
    align-items: center;
    flex-shrink: 0;
  }
  .chip-aviso .chip {
    border-style: dashed;
    color: var(--texto-suave);
  }
  .cerrar {
    color: var(--texto-tenue);
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
    font-family: var(--fuente-chica);
    font-size: var(--t-micro);
    font-weight: 600;
    color: var(--texto-suave);
    min-width: 0;
  }
  .pie-selector {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }
  label.check {
    display: flex;
    align-items: center;
    gap: 7px;
    min-height: var(--objetivo-min);
    font-family: var(--fuente);
    font-size: var(--t-meta);
    font-weight: 500;
  }
  select {
    font: inherit;
    font-family: var(--fuente);
    font-size: var(--t-cuerpo);
    font-weight: 400;
    min-height: var(--objetivo-min);
    padding: 6px 8px;
    border-radius: var(--radio-s);
    border: 1px solid var(--borde-fuerte);
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

  .selector-plegado {
    display: flex;
    gap: 6px;
  }
  .buscar {
    display: inline-flex;
    align-items: center;
    gap: 7px;
    flex: 1;
    min-height: var(--objetivo-min);
    padding: 0 12px;
    border: 1px solid var(--borde);
    border-radius: var(--radio-m);
    background: var(--superficie);
    color: var(--texto-suave);
    font: inherit;
    font-size: var(--t-meta);
    cursor: pointer;
    transition:
      border-color var(--mov-ui),
      background-color var(--mov-ui),
      color var(--mov-ui);
  }
  @media (hover: hover) {
    .buscar:hover {
      border-color: var(--borde-fuerte);
      background: var(--superficie-elevada);
      color: var(--texto);
    }
  }

  .estacion {
    display: grid;
    gap: 10px;
  }
  .centinela {
    height: 1px;
    margin-bottom: -11px; /* no suma espacio: compensa el gap de la grilla */
  }
  /* Nombre, frescura y actualizar quedan a mano al recorrer una lista larga. */
  .cabecera {
    position: sticky;
    top: env(safe-area-inset-top);
    z-index: 5;
    display: grid;
    gap: 8px;
    margin: 0 -14px;
    padding: 6px 14px;
    background: var(--fondo);
    transition: box-shadow var(--mov-ui);
  }
  .cabecera.pegada {
    box-shadow: 0 1px 0 var(--borde);
  }
  .titulo {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    gap: 8px;
  }
  .nombre {
    min-width: 0;
  }
  h2 {
    margin: 0;
    font-family: var(--fuente-display);
    font-size: var(--t-titulo);
    font-weight: 600;
    letter-spacing: -0.01em;
  }
  .sub {
    display: flex;
    flex-wrap: wrap;
    gap: 0 10px;
    margin: 2px 0 0;
    font-size: var(--t-meta);
    color: var(--texto-tenue);
  }
  .sub span:first-child {
    color: var(--texto-suave);
  }
  .sello {
    font-variant-numeric: tabular-nums;
  }
  .acciones {
    display: flex;
    gap: 2px;
  }
  .acciones .boton-icono {
    min-width: 36px;
    min-height: 36px;
  }
  .boton-icono.marcado {
    color: var(--acento);
  }
  .girando :global(svg) {
    animation: girar 0.9s linear infinite;
  }
  @keyframes girar {
    to {
      transform: rotate(360deg);
    }
  }

  .noticias {
    display: grid;
    gap: 6px;
  }

  /* Avisos: neutros salvo los errores, que van en rojo. El ámbar es de "Demorado". */
  .aviso {
    display: flex;
    align-items: center;
    gap: 8px;
    margin: 0;
    padding: 8px 10px;
    border-radius: var(--radio-m);
    font-size: var(--t-meta);
    color: var(--texto-suave);
    background: var(--superficie-elevada);
    border: 1px solid var(--borde);
  }
  .aviso span {
    color: var(--texto);
  }
  .aviso.error {
    color: var(--cancelado);
    background: color-mix(in srgb, var(--cancelado) 12%, var(--superficie));
    border-color: color-mix(in srgb, var(--cancelado) 30%, transparent);
  }
  .aviso :global(svg) {
    flex-shrink: 0;
  }
  button.enlace {
    margin-left: auto;
    min-height: var(--objetivo-min);
    background: none;
    border: none;
    padding: 0 4px;
    color: var(--acento);
    cursor: pointer;
    font: inherit;
    font-size: var(--t-meta);
    font-weight: 600;
  }

  .vacio {
    display: grid;
    gap: 4px;
    padding: 22px 16px;
    text-align: center;
    border: 1px dashed var(--borde-fuerte);
    border-radius: var(--radio-l);
  }
  .vacio span {
    font-size: var(--t-meta);
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
    min-height: var(--objetivo-min);
    padding: 6px 0;
    background: none;
    border: none;
    color: var(--texto);
    font: inherit;
    font-weight: 600;
    cursor: pointer;
  }
  .caret {
    display: grid;
    color: var(--texto-tenue);
    transition: transform var(--mov-ui);
  }
  .caret.abierto {
    transform: rotate(90deg);
  }
  .tenue {
    margin: 0 0 10px;
    font-size: var(--t-meta);
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
    display: inline-flex;
    align-items: center;
    gap: 4px;
    font-size: var(--t-meta);
    font-weight: 500;
    padding: 3px 10px 3px 8px;
    border-radius: var(--radio-pill);
    color: var(--texto);
    background: var(--superficie-elevada);
    border: 1px solid var(--borde);
  }
  .servicios li.no {
    color: var(--texto-tenue);
    background: none;
  }
  .fichas {
    display: grid;
    grid-template-columns: auto 1fr;
    gap: 6px 12px;
    margin: 0 0 12px;
    font-size: var(--t-meta);
  }
  .fichas dt {
    color: var(--texto-tenue);
  }
  .fichas dd {
    margin: 0;
  }
</style>

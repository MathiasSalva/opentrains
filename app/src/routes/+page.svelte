<script lang="ts">
  import { onMount } from "svelte";
  import {
    api,
    describirError,
    esFuenteCambiada,
    mismoFavorito,
    type Arribo,
    type Estacion,
    type Favorito,
    type Instalaciones,
    type Linea,
    type Preferencias,
    type Ramal,
  } from "$lib/api";
  import { cuentaRegresiva, hace, hora } from "$lib/formato";

  const REFRESCO_MS = 45_000;

  let preferencias = $state<Preferencias>({ favoritos: [], ultimo_abierto: null });

  let lineas = $state<Linea[]>([]);
  let ramales = $state<Ramal[]>([]);
  let estaciones = $state<Estacion[]>([]);
  let lineaId = $state<number | null>(null);
  let ramalId = $state<number | null>(null);
  let estacionId = $state<number | null>(null);
  let soloEsteRamal = $state(true);
  let errorCatalogo = $state<string | null>(null);

  let actual = $state<Favorito | null>(null);
  let arribos = $state<Arribo[]>([]);
  let ultimaActualizacion = $state<number | null>(null);
  let errorArribos = $state<string | null>(null);
  let fuenteCambiada = $state(false);
  let cargando = $state(false);

  let instalaciones = $state<Instalaciones | null | undefined>(undefined);
  let verInstalaciones = $state(false);

  let ahora = $state(Date.now());

  const esFavorito = $derived(
    actual !== null && preferencias.favoritos.some((f) => mismoFavorito(f, actual!)),
  );
  const desactualizado = $derived(
    errorArribos !== null && ultimaActualizacion !== null && arribos.length > 0,
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
    });
  }

  async function abrir(favorito: Favorito) {
    actual = favorito;
    arribos = [];
    ultimaActualizacion = null;
    errorArribos = null;
    instalaciones = undefined;
    verInstalaciones = false;
    try {
      preferencias = await api.abrir(favorito);
    } catch (e) {
      // Recordar el último abierto es una comodidad: no bloquea la consulta.
      console.warn(describirError(e));
    }
    await refrescar();
  }

  async function refrescar() {
    if (!actual || cargando) return;
    const consultado = actual;
    cargando = true;
    try {
      const nuevos = await api.arribos(consultado);
      if (actual !== consultado) return;
      arribos = nuevos;
      ultimaActualizacion = Date.now();
      errorArribos = null;
      fuenteCambiada = false;
    } catch (e) {
      if (actual !== consultado) return;
      errorArribos = describirError(e);
      fuenteCambiada = esFuenteCambiada(e);
    } finally {
      cargando = false;
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

  function etiqueta(f: Favorito): string {
    return f.ramal ? `${f.estacion_nombre} · ${f.ramal.nombre}` : f.estacion_nombre;
  }

  onMount(() => {
    (async () => {
      try {
        preferencias = await api.preferencias();
      } catch (e) {
        errorCatalogo = describirError(e);
      }
      await cargarLineas();
      if (preferencias.ultimo_abierto) await abrir(preferencias.ultimo_abierto);
    })();

    // Refresco cada 45 s y cuenta regresiva cada 1 s, solo con la ventana visible.
    const visible = () => document.visibilityState === "visible";
    const refresco = setInterval(() => visible() && refrescar(), REFRESCO_MS);
    const reloj = setInterval(() => visible() && (ahora = Date.now()), 1_000);
    const alVolver = () => {
      if (!visible()) return;
      ahora = Date.now();
      if (ultimaActualizacion === null || ahora - ultimaActualizacion >= REFRESCO_MS) refrescar();
    };
    document.addEventListener("visibilitychange", alVolver);
    return () => {
      clearInterval(refresco);
      clearInterval(reloj);
      document.removeEventListener("visibilitychange", alVolver);
    };
  });
</script>

<main>
  <header>
    <h1>OpenTrains</h1>
  </header>

  {#if preferencias.favoritos.length > 0}
    <nav class="favoritos" aria-label="Favoritos">
      {#each preferencias.favoritos as favorito}
        <button
          class="chip"
          class:activo={actual !== null && mismoFavorito(favorito, actual)}
          onclick={() => abrir(favorito)}>★ {etiqueta(favorito)}</button
        >
      {/each}
    </nav>
  {/if}

  <section class="selector">
    <label>
      Línea
      <select bind:value={lineaId} onchange={elegirLinea}>
        <option value={null}>Elegí una línea</option>
        {#each lineas as linea}
          <option value={linea.id}>{linea.nombre}</option>
        {/each}
      </select>
    </label>
    <label>
      Ramal
      <select bind:value={ramalId} onchange={elegirRamal} disabled={ramales.length === 0}>
        <option value={null}>Elegí un ramal</option>
        {#each ramales as ramal}
          <option value={ramal.id}>{ramal.nombre}</option>
        {/each}
      </select>
    </label>
    <label>
      Estación
      <select
        bind:value={estacionId}
        onchange={elegirEstacion}
        disabled={estaciones.length === 0}
      >
        <option value={null}>Elegí una estación</option>
        {#each estaciones as estacion}
          <option value={estacion.id}>{estacion.nombre}</option>
        {/each}
      </select>
    </label>
    <label class="check">
      <input type="checkbox" bind:checked={soloEsteRamal} onchange={elegirEstacion} />
      Solo este ramal
    </label>
    {#if errorCatalogo}
      <p class="aviso error">
        {errorCatalogo} <button class="link" onclick={cargarLineas}>Reintentar</button>
      </p>
    {/if}
  </section>

  {#if actual}
    <section class="arribos">
      <div class="titulo">
        <div>
          <h2>{actual.estacion_nombre}</h2>
          <p class="sub">{actual.ramal ? actual.ramal.nombre : "Todos los ramales"}</p>
        </div>
        <div class="acciones">
          <button
            class="icono"
            title={esFavorito ? "Quitar de favoritos" : "Agregar a favoritos"}
            aria-pressed={esFavorito}
            onclick={alternarFavorito}>{esFavorito ? "★" : "☆"}</button
          >
          <button class="icono" title="Actualizar" disabled={cargando} onclick={refrescar}
            >⟳</button
          >
        </div>
      </div>

      {#if fuenteCambiada}
        <p class="aviso error">{errorArribos}</p>
      {:else if desactualizado}
        <p class="aviso">
          Desactualizado — última actualización hace {hace(ultimaActualizacion!, ahora)}
        </p>
      {:else if errorArribos}
        <p class="aviso error">{errorArribos}</p>
      {/if}

      {#if cargando && ultimaActualizacion === null}
        <p class="vacio">Cargando…</p>
      {:else if ultimaActualizacion !== null && arribos.length === 0}
        <p class="vacio">No hay próximos trenes informados.</p>
      {/if}

      <ul>
        {#each arribos as arribo (`${arribo.numero}-${arribo.momento}`)}
          <li class="arribo {arribo.tono}">
            <div class="cuenta">{cuentaRegresiva(arribo.momento, ahora)}</div>
            <div class="detalle">
              <div class="destino" class:tachado={arribo.cancelado}>a {arribo.destino}</div>
              <div class="meta">
                {hora(arribo.programada)}
                {#if arribo.demora_minutos !== null}
                  <span class="demora">+{arribo.demora_minutos} min</span>
                {/if}
                {#if arribo.anden}· Andén {arribo.anden}{/if}
                · #{arribo.numero}
              </div>
            </div>
            <span class="estado">{arribo.cancelado ? "Cancelado" : arribo.estado}</span>
          </li>
        {/each}
      </ul>
      {#if ultimaActualizacion !== null && !desactualizado}
        <p class="pie">Actualizado hace {hace(ultimaActualizacion, ahora)}</p>
      {/if}
    </section>

    <section class="instalaciones">
      <button class="link" onclick={alternarInstalaciones}>
        {verInstalaciones ? "▾" : "▸"} Instalaciones de la estación
      </button>
      {#if verInstalaciones}
        {#if instalaciones === undefined}
          <p class="vacio">Cargando…</p>
        {:else if instalaciones === null}
          <p class="vacio">Sin información para esta estación.</p>
        {:else}
          <ul class="instalaciones-lista">
            <li class:no={!instalaciones.ascensor}>Ascensor</li>
            <li class:no={!instalaciones.rampa}>Rampa</li>
            <li class:no={!instalaciones.banos}>Baños</li>
            <li class:no={!instalaciones.wifi}>Wifi</li>
            <li class:no={!instalaciones.terminal_sube}>Carga SUBE</li>
          </ul>
          {#if instalaciones.boleteria}<p><strong>Boletería:</strong> {instalaciones.boleteria}</p>{/if}
          {#if instalaciones.direccion}<p><strong>Dirección:</strong> {instalaciones.direccion}</p>{/if}
          {#if instalaciones.conexiones}<p><strong>Conexiones:</strong> {instalaciones.conexiones}</p>{/if}
          {#if instalaciones.colectivos}<p><strong>Colectivos:</strong> {instalaciones.colectivos}</p>{/if}
        {/if}
      {/if}
    </section>
  {/if}
</main>

<style>
  :global(:root) {
    --fondo: #f6f7f9;
    --superficie: #ffffff;
    --texto: #1d2733;
    --texto-suave: #5d6b7a;
    --borde: #dde2e8;
    --acento: #2f6db5;
    --normal: #80c395;
    --cancelado: #e45255;
    --neutro: #435a6c;
    --aviso: #f7a83a;
    color-scheme: light dark;
    font-family: "Segoe UI", system-ui, sans-serif;
    font-size: 15px;
  }
  @media (prefers-color-scheme: dark) {
    :global(:root) {
      --fondo: #14181d;
      --superficie: #1d232a;
      --texto: #e7ecf1;
      --texto-suave: #9aa8b6;
      --borde: #2e3740;
      --acento: #6aa3e6;
      --neutro: #8a9bab;
    }
  }
  :global(body) {
    margin: 0;
    background: var(--fondo);
    color: var(--texto);
  }
  main {
    padding: 12px 14px 20px;
    display: flex;
    flex-direction: column;
    gap: 12px;
  }
  header h1 {
    margin: 0;
    font-size: 1.25rem;
  }
  h2 {
    margin: 0;
    font-size: 1.1rem;
  }
  section {
    background: var(--superficie);
    border: 1px solid var(--borde);
    border-radius: 10px;
    padding: 10px 12px;
  }
  .favoritos {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
  }
  .chip {
    border: 1px solid var(--borde);
    background: var(--superficie);
    color: var(--texto);
    border-radius: 999px;
    padding: 4px 10px;
    cursor: pointer;
    font: inherit;
    font-size: 0.85rem;
  }
  .chip.activo {
    border-color: var(--acento);
    color: var(--acento);
  }
  .selector {
    display: grid;
    gap: 8px;
  }
  label {
    display: grid;
    gap: 2px;
    font-size: 0.8rem;
    color: var(--texto-suave);
  }
  label.check {
    display: flex;
    align-items: center;
    gap: 6px;
  }
  select {
    font: inherit;
    padding: 5px 6px;
    border-radius: 6px;
    border: 1px solid var(--borde);
    background: var(--fondo);
    color: var(--texto);
  }
  .titulo {
    display: flex;
    justify-content: space-between;
    align-items: start;
  }
  .sub,
  .pie,
  .vacio {
    margin: 2px 0 0;
    font-size: 0.8rem;
    color: var(--texto-suave);
  }
  .acciones {
    display: flex;
    gap: 4px;
  }
  button.icono {
    font-size: 1.2rem;
    background: none;
    border: none;
    color: var(--acento);
    cursor: pointer;
    padding: 2px 6px;
  }
  button.icono:disabled {
    opacity: 0.4;
  }
  button.link {
    background: none;
    border: none;
    color: var(--acento);
    cursor: pointer;
    font: inherit;
    padding: 0;
  }
  .aviso {
    margin: 8px 0 0;
    padding: 6px 8px;
    border-radius: 6px;
    font-size: 0.85rem;
    border-left: 4px solid var(--aviso);
    background: color-mix(in srgb, var(--aviso) 15%, transparent);
  }
  .aviso.error {
    border-left-color: var(--cancelado);
    background: color-mix(in srgb, var(--cancelado) 15%, transparent);
  }
  ul {
    list-style: none;
    margin: 8px 0 0;
    padding: 0;
  }
  .arribo {
    display: grid;
    grid-template-columns: 4.5rem 1fr auto;
    align-items: center;
    gap: 8px;
    padding: 8px 0;
    border-top: 1px solid var(--borde);
  }
  .cuenta {
    font-size: 1.05rem;
    font-weight: 600;
  }
  .destino {
    font-weight: 600;
  }
  .tachado {
    text-decoration: line-through;
  }
  .meta {
    font-size: 0.8rem;
    color: var(--texto-suave);
  }
  .demora {
    color: var(--aviso);
    font-weight: 600;
  }
  .estado {
    font-size: 0.75rem;
    padding: 2px 8px;
    border-radius: 999px;
    color: #fff;
    white-space: nowrap;
  }
  .normal .estado {
    background: var(--normal);
    color: #12301c;
  }
  .cancelado .estado {
    background: var(--cancelado);
  }
  .cancelado .destino,
  .cancelado .cuenta {
    color: var(--cancelado);
  }
  .neutro .estado {
    background: var(--neutro);
  }
  .instalaciones p {
    margin: 6px 0 0;
    font-size: 0.85rem;
  }
  .instalaciones-lista {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
  }
  .instalaciones-lista li {
    font-size: 0.8rem;
    padding: 2px 8px;
    border-radius: 999px;
    background: color-mix(in srgb, var(--normal) 25%, transparent);
  }
  .instalaciones-lista li.no {
    background: none;
    text-decoration: line-through;
    color: var(--texto-suave);
  }
</style>

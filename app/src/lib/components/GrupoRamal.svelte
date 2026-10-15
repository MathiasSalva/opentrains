<script lang="ts">
  import { flip } from "svelte/animate";
  import { fade, fly } from "svelte/transition";
  import { clave, type GrupoRamal } from "$lib/arribos";
  import { duracion, EASE } from "$lib/movimiento";
  import FilaArribo from "./FilaArribo.svelte";

  let {
    grupo,
    ahora,
    encabezado = true,
  }: {
    grupo: GrupoRamal;
    ahora: number;
    /** Sin encabezado cuando es el único Ramal en pantalla y ya se nombra arriba. */
    encabezado?: boolean;
  } = $props();

  // Se ven los próximos; el resto, a pedido.
  const VISIBLES = 3;
  let expandido = $state(false);
  const visibles = $derived(expandido ? grupo.arribos : grupo.arribos.slice(0, VISIBLES));
  const ocultos = $derived(grupo.arribos.length - VISIBLES);
</script>

<section class="grupo" aria-label={encabezado ? undefined : grupo.ramal.nombre}>
  {#if encabezado}
    <header>
      <h3>{grupo.ramal.nombre}</h3>
      <span>{grupo.arribos.length} {grupo.arribos.length === 1 ? "tren" : "trenes"}</span>
    </header>
  {/if}
  <ul class:sin-encabezado={!encabezado}>
    {#each visibles as arribo (clave(arribo))}
      <!-- Al refrescar, los trenes que siguen se reacomodan y los que pasaron se van. -->
      <li
        animate:flip={{ duration: duracion(350), easing: EASE }}
        in:fly={{ y: 8, duration: duracion(300), easing: EASE }}
        out:fade={{ duration: duracion(180) }}
      >
        <FilaArribo {arribo} {ahora} />
      </li>
    {/each}
  </ul>
  {#if ocultos > 0}
    <button class="mas" onclick={() => (expandido = !expandido)}>
      {expandido ? "Ver menos" : `Ver ${ocultos} más`}
    </button>
  {/if}
</section>

<style>
  .grupo {
    background: var(--superficie);
    border: 1px solid var(--borde);
    border-radius: var(--radio-l);
  }
  header {
    display: flex;
    justify-content: space-between;
    align-items: baseline;
    padding: 10px 12px 4px 14px;
  }
  h3 {
    margin: 0;
    font-size: var(--t-cuerpo);
    font-weight: 600;
  }
  header span {
    font-size: var(--t-meta);
    color: var(--texto-tenue);
  }
  ul {
    list-style: none;
    margin: 0;
    padding: 0 0 4px;
  }
  ul.sin-encabezado {
    padding-top: 4px;
  }
  li + li {
    border-top: 1px solid var(--borde);
  }
  .mas {
    width: 100%;
    min-height: var(--objetivo-min);
    padding: 8px;
    background: none;
    border: none;
    border-top: 1px solid var(--borde);
    color: var(--acento);
    font: inherit;
    font-size: var(--t-meta);
    font-weight: 600;
    cursor: pointer;
    border-radius: 0 0 calc(var(--radio-l) - 1px) calc(var(--radio-l) - 1px);
    transition: background-color var(--mov-ui);
  }
  @media (hover: hover) {
    .mas:hover {
      background: var(--superficie-hover);
    }
  }
</style>

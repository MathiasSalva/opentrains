<script lang="ts">
  import { flip } from "svelte/animate";
  import { fade, fly } from "svelte/transition";
  import { clave, type GrupoRamal } from "$lib/arribos";
  import { duracion, EASE } from "$lib/movimiento";
  import FilaArribo from "./FilaArribo.svelte";

  let { grupo, ahora }: { grupo: GrupoRamal; ahora: number } = $props();

  // Se ven los próximos; el resto, a pedido.
  const VISIBLES = 3;
  let expandido = $state(false);
  const visibles = $derived(expandido ? grupo.arribos : grupo.arribos.slice(0, VISIBLES));
  const ocultos = $derived(grupo.arribos.length - VISIBLES);
</script>

<section class="grupo">
  <header>
    <h3>{grupo.ramal.nombre}</h3>
    <span>{grupo.arribos.length} {grupo.arribos.length === 1 ? "tren" : "trenes"}</span>
  </header>
  <ul>
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
    border-radius: 14px;
    overflow: hidden;
  }
  header {
    display: flex;
    justify-content: space-between;
    align-items: baseline;
    padding: 10px 12px 4px 14px;
  }
  h3 {
    margin: 0;
    font-size: 0.9rem;
  }
  header span {
    font-size: 0.75rem;
    color: var(--texto-suave);
  }
  ul {
    list-style: none;
    margin: 0;
    padding: 0 0 4px;
  }
  li + li {
    border-top: 1px solid var(--borde);
  }
  .mas {
    width: 100%;
    padding: 8px;
    background: none;
    border: none;
    border-top: 1px solid var(--borde);
    color: var(--acento);
    font: inherit;
    font-size: 0.82rem;
    font-weight: 600;
    cursor: pointer;
    transition: background-color 0.2s ease;
  }
  .mas:hover {
    background: color-mix(in srgb, var(--acento) 8%, transparent);
  }
</style>

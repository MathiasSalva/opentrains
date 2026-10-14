<script lang="ts">
  import BellIcon from "phosphor-svelte/lib/BellIcon";
  import BellRingingIcon from "phosphor-svelte/lib/BellRingingIcon";
  import { scale } from "svelte/transition";
  import type { Arribo } from "$lib/api";
  import {
    anticipacion,
    anticipacionSugerida,
    dejarDeSeguir,
    seguir,
  } from "$lib/seguimientos.svelte";
  import { duracion, EASE } from "$lib/movimiento";

  let { arribo }: { arribo: Arribo } = $props();

  const OPCIONES = [5, 10, 15];
  let abierto = $state(false);
  let raiz: HTMLElement;

  const minutos = $derived(anticipacion(arribo.numero));
  const sugerida = $derived(anticipacionSugerida());
  const cancelado = $derived(arribo.situacion === "cancelado");

  async function alternar() {
    if (minutos !== null) {
      await dejarDeSeguir(arribo.numero);
    } else {
      abierto = !abierto;
    }
  }

  async function elegir(m: number) {
    abierto = false;
    await seguir(arribo.numero, arribo.destino, m);
  }

  function alHacerClicAfuera(e: MouseEvent) {
    if (abierto && !raiz.contains(e.target as Node)) abierto = false;
  }
</script>

<svelte:window onclick={alHacerClicAfuera} />

<span class="avisar" bind:this={raiz}>
  <button
    class="campana"
    class:activa={minutos !== null}
    disabled={cancelado && minutos === null}
    aria-pressed={minutos !== null}
    aria-expanded={abierto}
    title={minutos !== null
      ? `Te aviso ${minutos} min antes. Clic para dejar de seguirlo`
      : "Avisarme de este tren"}
    onclick={alternar}
  >
    {#if minutos !== null}
      <BellRingingIcon size={17} weight="fill" />
    {:else}
      <BellIcon size={17} />
    {/if}
  </button>
  {#if abierto}
    <div class="menu" role="menu" transition:scale={{ start: 0.92, duration: duracion(140), easing: EASE }}>
      <span class="rotulo">Avisarme antes</span>
      {#each OPCIONES as m}
        <button
          role="menuitem"
          class:sugerida={m === sugerida}
          title={m === sugerida ? "La última que elegiste en esta estación" : undefined}
          onclick={() => elegir(m)}>{m} min</button
        >
      {/each}
    </div>
  {/if}
</span>

<style>
  .avisar {
    position: relative;
    display: inline-grid;
  }
  .campana {
    display: grid;
    place-items: center;
    width: 30px;
    height: 30px;
    border-radius: 8px;
    border: none;
    background: none;
    color: var(--texto-suave);
    cursor: pointer;
    transition:
      background-color 0.2s ease,
      color 0.2s ease,
      transform 0.15s ease;
  }
  .campana:hover:not(:disabled) {
    background: var(--borde);
    color: var(--texto);
  }
  .campana:active:not(:disabled) {
    transform: scale(0.92);
  }
  .campana:disabled {
    opacity: 0.35;
    cursor: default;
  }
  .campana.activa {
    color: var(--acento);
  }
  .menu {
    position: absolute;
    right: 0;
    top: calc(100% + 4px);
    z-index: 10;
    display: grid;
    min-width: 140px;
    padding: 6px;
    border-radius: 12px;
    background: var(--superficie);
    border: 1px solid var(--borde);
    box-shadow: 0 10px 28px color-mix(in srgb, var(--texto) 18%, transparent);
    transform-origin: top right;
  }
  .rotulo {
    padding: 4px 8px 6px;
    font-size: 0.72rem;
    font-weight: 600;
    color: var(--texto-suave);
  }
  .menu button {
    text-align: left;
    padding: 7px 8px;
    border: none;
    border-radius: 8px;
    background: none;
    color: var(--texto);
    font: inherit;
    font-size: 0.88rem;
    cursor: pointer;
  }
  .menu button:hover {
    background: color-mix(in srgb, var(--acento) 12%, transparent);
  }
  .menu button.sugerida {
    color: var(--acento);
    font-weight: 600;
  }
</style>

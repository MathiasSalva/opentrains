<script lang="ts">
  import { tick } from "svelte";
  import BellIcon from "phosphor-svelte/lib/BellIcon";
  import BellRingingIcon from "phosphor-svelte/lib/BellRingingIcon";
  import { scale } from "svelte/transition";
  import type { Arribo } from "$lib/api";
  import { hora } from "$lib/formato";
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
  let campana: HTMLButtonElement;
  let menu = $state<HTMLElement>();

  const minutos = $derived(anticipacion(arribo.numero));
  const sugerida = $derived(anticipacionSugerida());
  const cancelado = $derived(arribo.situacion === "cancelado");
  // Cada campana se distingue por su tren: en una lista no son todas "Avisarme".
  const tren = $derived(`tren a ${arribo.destino} de las ${hora(arribo.momento)}`);

  async function alternar() {
    if (minutos !== null) {
      await dejarDeSeguir(arribo.numero);
    } else if (abierto) {
      cerrar();
    } else {
      abierto = true;
      await tick();
      // El foco entra al menú en la opción sugerida, o en la primera.
      const opciones = items();
      (opciones.find((b) => b.classList.contains("sugerida")) ?? opciones[0])?.focus();
    }
  }

  function cerrar(devolverFoco = true) {
    abierto = false;
    if (devolverFoco) campana.focus();
  }

  async function elegir(m: number) {
    cerrar();
    await seguir(arribo.numero, arribo.destino, m);
  }

  function items(): HTMLButtonElement[] {
    return menu ? [...menu.querySelectorAll<HTMLButtonElement>("button")] : [];
  }

  // Escape cierra el menú y nada más: la página no oculta el panel si ya se usó.
  function alTeclear(e: KeyboardEvent) {
    const opciones = items();
    const i = opciones.indexOf(document.activeElement as HTMLButtonElement);
    if (e.key === "Escape") {
      e.preventDefault();
      cerrar();
    } else if (e.key === "ArrowDown" || e.key === "ArrowUp") {
      e.preventDefault();
      const paso = e.key === "ArrowDown" ? 1 : -1;
      opciones[(i + paso + opciones.length) % opciones.length]?.focus();
    } else if (e.key === "Home" || e.key === "End") {
      e.preventDefault();
      opciones[e.key === "Home" ? 0 : opciones.length - 1]?.focus();
    } else if (e.key === "Tab") {
      cerrar(false);
    }
  }

  function alHacerClicAfuera(e: MouseEvent) {
    if (abierto && !raiz.contains(e.target as Node)) cerrar(false);
  }
</script>

<svelte:window onclick={alHacerClicAfuera} />

<span class="avisar" bind:this={raiz}>
  <button
    bind:this={campana}
    class="boton-icono campana"
    class:activa={minutos !== null}
    disabled={cancelado && minutos === null}
    aria-pressed={minutos !== null}
    aria-haspopup="menu"
    aria-expanded={abierto}
    aria-label={minutos !== null
      ? `Dejar de seguir el ${tren} (aviso ${minutos} min antes)`
      : `Avisarme del ${tren}`}
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
    <div
      bind:this={menu}
      class="menu"
      role="menu"
      tabindex="-1"
      aria-label="Avisarme antes"
      onkeydown={alTeclear}
      transition:scale={{ start: 0.94, duration: duracion(140), easing: EASE }}
    >
      <span class="rotulo" aria-hidden="true">Avisarme antes</span>
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
  .campana:disabled {
    opacity: 0.35;
  }
  .campana.activa {
    color: var(--acento);
  }
  /* Menú flotante: superficie elevada, borde translúcido y sombra negra (también en
     modo oscuro, donde la profundidad la da sobre todo el escalón de superficie). */
  .menu {
    position: absolute;
    right: 0;
    top: calc(100% + 4px);
    z-index: 10;
    display: grid;
    min-width: 140px;
    padding: 6px;
    border-radius: var(--radio-m);
    background: var(--superficie-elevada);
    border: 1px solid var(--borde-fuerte);
    box-shadow: var(--sombra-flotante);
    transform-origin: top right;
  }
  .menu:focus {
    outline: none;
  }
  .rotulo {
    padding: 4px 8px 6px;
    font-family: var(--fuente-chica);
    font-size: var(--t-micro);
    font-weight: 600;
    color: var(--texto-tenue);
  }
  .menu button {
    text-align: left;
    min-height: var(--objetivo-min);
    padding: 6px 8px;
    border: none;
    border-radius: var(--radio-s);
    background: none;
    color: var(--texto);
    font: inherit;
    cursor: pointer;
    transition: background-color var(--mov-ui);
  }
  .menu button:hover,
  .menu button:focus-visible {
    background: var(--superficie-hover);
    outline: none;
  }
  .menu button.sugerida {
    color: var(--acento);
    font-weight: 600;
  }
</style>

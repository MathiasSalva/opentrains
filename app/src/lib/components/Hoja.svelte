<script lang="ts">
  import type { Snippet } from "svelte";
  import { fade, fly } from "svelte/transition";
  import XIcon from "phosphor-svelte/lib/XIcon";
  import { duracion } from "$lib/movimiento";

  // Hoja inferior sobre el panel: el detalle de un Servicio (y, más adelante, su recorrido
  // parada por parada y el diagrama del Ramal). En el celular es el patrón nativo.
  let {
    titulo,
    alCerrar,
    children,
  }: { titulo: string; alCerrar: () => void; children: Snippet } = $props();

  let hoja: HTMLElement;
  // Al cerrar, el foco vuelve a lo que abrió la hoja.
  const anterior = document.activeElement as HTMLElement | null;

  // Curva de "cajón": arranca rápido y se asienta despacio (como --mov-hoja).
  const CAJON = (t: number) => 1 - Math.pow(1 - t, 3.2);

  $effect(() => {
    hoja.focus();
    return () => anterior?.focus();
  });

  function alTeclear(e: KeyboardEvent) {
    // Escape cierra la hoja y nada más (la página no oculta el panel si ya se usó).
    if (e.key === "Escape" && !e.defaultPrevented) {
      e.preventDefault();
      alCerrar();
    }
  }
</script>

<div class="fondo" transition:fade={{ duration: duracion(200) }} onclick={alCerrar} aria-hidden="true"></div>
<div
  bind:this={hoja}
  class="hoja"
  role="dialog"
  aria-modal="true"
  aria-label={titulo}
  tabindex="-1"
  onkeydown={alTeclear}
  transition:fly={{ y: 320, duration: duracion(420), easing: CAJON }}
>
  <div class="agarre" aria-hidden="true"></div>
  <header>
    <h2>{titulo}</h2>
    <button class="boton-icono" aria-label="Cerrar" title="Cerrar" onclick={alCerrar}>
      <XIcon size={18} weight="bold" />
    </button>
  </header>
  <div class="contenido">
    {@render children()}
  </div>
</div>

<style>
  .fondo {
    position: fixed;
    inset: 0;
    z-index: 20;
    background: rgb(0 0 0 / 0.4);
  }
  .hoja {
    position: fixed;
    inset: auto 0 0;
    z-index: 21;
    max-height: 85vh;
    overflow-y: auto;
    padding: 6px 16px calc(18px + env(safe-area-inset-bottom));
    border-radius: var(--radio-l) var(--radio-l) 0 0;
    background: var(--superficie-elevada);
    border-top: 1px solid var(--borde-fuerte);
    box-shadow: var(--sombra-flotante);
  }
  .hoja:focus {
    outline: none;
  }
  .agarre {
    width: 36px;
    height: 4px;
    margin: 2px auto 6px;
    border-radius: var(--radio-pill);
    background: var(--borde-fuerte);
  }
  header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
  }
  h2 {
    margin: 0;
    font-family: var(--fuente-display);
    font-size: var(--t-titulo);
    font-weight: 600;
  }
  .contenido {
    margin-top: 8px;
  }
</style>

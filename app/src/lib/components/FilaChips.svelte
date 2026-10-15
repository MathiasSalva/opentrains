<script lang="ts">
  import type { Snippet } from "svelte";

  // Una fila de chips con scroll horizontal. Lo que no entra se anuncia con un degradé
  // en el borde, y la rueda del mouse también la desplaza (en escritorio no hay swipe).
  let { etiqueta, children }: { etiqueta: string; children: Snippet } = $props();

  let fila: HTMLElement;
  let antes = $state(false);
  let despues = $state(false);

  function medir() {
    antes = fila.scrollLeft > 1;
    despues = fila.scrollLeft + fila.clientWidth < fila.scrollWidth - 1;
  }

  function alGirarRueda(e: WheelEvent) {
    if (!despues && !antes) return;
    if (Math.abs(e.deltaY) <= Math.abs(e.deltaX)) return;
    e.preventDefault();
    fila.scrollLeft += e.deltaY;
  }

  $effect(() => {
    medir();
    const observador = new ResizeObserver(medir);
    observador.observe(fila);
    // Chips que llegan o se van (Favoritos, Estaciones cercanas) cambian lo que no entra.
    const cambios = new MutationObserver(medir);
    cambios.observe(fila, { childList: true, subtree: true, characterData: true });
    // Svelte registra `wheel` como pasivo; acá hace falta poder cancelarlo.
    fila.addEventListener("wheel", alGirarRueda, { passive: false });
    return () => {
      observador.disconnect();
      cambios.disconnect();
      fila.removeEventListener("wheel", alGirarRueda);
    };
  });
</script>

<nav
  bind:this={fila}
  class="chips"
  class:antes
  class:despues
  aria-label={etiqueta}
  onscroll={medir}
>
  {@render children()}
</nav>

<style>
  .chips {
    display: flex;
    align-items: center;
    gap: 6px;
    overflow-x: auto;
    scrollbar-width: none;
    --degrade: 28px;
  }
  .chips.despues {
    mask-image: linear-gradient(to right, #000 calc(100% - var(--degrade)), transparent);
  }
  .chips.antes {
    mask-image: linear-gradient(to right, transparent, #000 var(--degrade));
  }
  .chips.antes.despues {
    mask-image: linear-gradient(
      to right,
      transparent,
      #000 var(--degrade),
      #000 calc(100% - var(--degrade)),
      transparent
    );
  }
  /* Chip compartido: Favoritos, Estaciones cercanas y acciones de la fila. */
  .chips :global(.chip) {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    flex-shrink: 0;
    min-height: var(--objetivo-min);
    padding: 0 11px;
    border: 1px solid var(--borde);
    border-radius: var(--radio-pill);
    background: var(--superficie);
    color: var(--texto);
    font: inherit;
    font-size: var(--t-meta);
    cursor: pointer;
    transition:
      border-color var(--mov-ui),
      background-color var(--mov-ui),
      color var(--mov-ui),
      transform var(--mov-ui);
  }
  .chips :global(.chip svg) {
    color: var(--texto-tenue);
  }
  @media (hover: hover) {
    .chips :global(.chip:hover) {
      border-color: var(--borde-fuerte);
      background: var(--superficie-elevada);
    }
  }
  .chips :global(.chip:active) {
    transform: scale(0.97);
  }
  .chips :global(.chip.activo) {
    border-color: color-mix(in srgb, var(--acento) 55%, transparent);
    color: var(--acento);
  }
  .chips :global(.chip.activo svg) {
    color: var(--acento);
  }
  .chips :global(.chip .distancia) {
    color: var(--texto-tenue);
    font-variant-numeric: tabular-nums;
  }
</style>

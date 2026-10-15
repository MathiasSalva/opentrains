<script lang="ts">
  import type { Snippet } from "svelte";

  // Insignia chica para metadatos de una fila (andén, y más adelante puntualidad).
  // Neutra: nunca toma colores de Situación. `atencion` la marca para algo que cambió.
  let { atencion = false, children }: { atencion?: boolean; children: Snippet } = $props();
</script>

<span class="insignia" class:atencion>{@render children()}</span>

<style>
  .insignia {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    padding: 0 6px;
    border-radius: var(--radio-pill);
    font-family: var(--fuente-chica);
    font-size: var(--t-micro);
    font-weight: 600;
    line-height: 18px;
    white-space: nowrap;
    color: var(--texto-suave);
    background: var(--superficie-elevada);
    border: 1px solid var(--borde);
    font-variant-numeric: tabular-nums;
  }
  .atencion {
    color: color-mix(in srgb, var(--atencion) 82%, var(--texto));
    background: color-mix(in srgb, var(--atencion) 12%, var(--superficie));
    border-color: color-mix(in srgb, var(--atencion) 35%, transparent);
  }
</style>

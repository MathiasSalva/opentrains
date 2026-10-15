<script lang="ts">
  import type { Noticia } from "$lib/api";
  import { slide } from "svelte/transition";
  import { duracion } from "$lib/movimiento";
  import MegaphoneIcon from "phosphor-svelte/lib/MegaphoneIcon";
  import WarningIcon from "phosphor-svelte/lib/WarningIcon";
  import CaretRightIcon from "phosphor-svelte/lib/CaretRightIcon";
  import XIcon from "phosphor-svelte/lib/XIcon";

  // Noticias del operador para la Estación abierta: las destacadas enteras, el resto
  // plegado en una línea. Solo las no destacadas se pueden descartar.
  let {
    noticias,
    descartar,
  }: { noticias: Noticia[]; descartar: (id: number) => void } = $props();

  let abiertas = $state(false);
  const destacadas = $derived(noticias.filter((n) => n.destacada));
  const resto = $derived(noticias.filter((n) => !n.destacada));
</script>

{#each destacadas as noticia (noticia.id)}
  <p class="noticia destacada" role="alert" transition:slide={{ duration: duracion(200) }}>
    <WarningIcon size={16} weight="bold" /><span>{noticia.texto}</span>
  </p>
{/each}

{#if resto.length > 0}
  <div class="resto" transition:slide={{ duration: duracion(200) }}>
    <button class="desplegar" aria-expanded={abiertas} onclick={() => (abiertas = !abiertas)}>
      <MegaphoneIcon size={16} weight="bold" />
      <span>{resto.length === 1 ? "1 noticia" : `${resto.length} noticias`} del operador</span>
      <span class="caret" class:abierto={abiertas}><CaretRightIcon size={12} weight="bold" /></span>
    </button>
    {#if abiertas}
      <ul transition:slide={{ duration: duracion(200) }}>
        {#each resto as noticia (noticia.id)}
          <li transition:slide={{ duration: duracion(160) }}>
            <span>{noticia.texto}</span>
            <button
              class="descartar"
              aria-label="Descartar esta noticia"
              title="Descartar"
              onclick={() => descartar(noticia.id)}
            >
              <XIcon size={14} weight="bold" />
            </button>
          </li>
        {/each}
      </ul>
    {/if}
  </div>
{/if}

<style>
  .noticia,
  .resto {
    margin: 0;
    border-radius: var(--radio-m);
    font-size: var(--t-meta);
    background: var(--superficie-elevada);
    border: 1px solid var(--borde);
  }
  .noticia {
    display: flex;
    align-items: flex-start;
    gap: 8px;
    padding: 8px 10px;
    color: var(--texto);
  }
  .destacada {
    background: color-mix(in srgb, var(--atencion) 10%, var(--superficie));
    border-color: color-mix(in srgb, var(--atencion) 32%, transparent);
  }
  .destacada :global(svg) {
    color: var(--atencion);
    flex-shrink: 0;
    margin-top: 1px;
  }
  .desplegar {
    display: flex;
    align-items: center;
    gap: 8px;
    width: 100%;
    min-height: var(--objetivo-min);
    padding: 4px 10px;
    border: none;
    border-radius: var(--radio-m);
    background: none;
    color: var(--texto-suave);
    font: inherit;
    text-align: left;
    cursor: pointer;
  }
  @media (hover: hover) {
    .desplegar:hover {
      background: var(--superficie-hover);
    }
  }
  .caret {
    display: inline-flex;
    margin-left: auto;
    transition: transform var(--mov-ui);
  }
  .caret.abierto {
    transform: rotate(90deg);
  }
  ul {
    list-style: none;
    margin: 0;
    padding: 0 4px 4px 10px;
    display: grid;
  }
  li {
    display: grid;
    grid-template-columns: minmax(0, 1fr) auto;
    align-items: start;
    gap: 6px;
    padding: 6px 0;
    border-top: 1px solid var(--borde);
    color: var(--texto);
  }
  .descartar {
    display: inline-grid;
    place-items: center;
    min-width: var(--objetivo-min);
    min-height: var(--objetivo-min);
    margin: -6px 0;
    border: none;
    border-radius: var(--radio-s);
    background: none;
    color: var(--texto-tenue);
    cursor: pointer;
  }
  @media (hover: hover) {
    .descartar:hover {
      background: var(--superficie-hover);
      color: var(--texto);
    }
  }
</style>

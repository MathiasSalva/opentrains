<script lang="ts">
  import type { Arribo } from "$lib/api";
  import { claseSituacion } from "$lib/arribos";
  import { cuentaRegresiva, hora } from "$lib/formato";
  import SituacionPill from "./SituacionPill.svelte";
  import BotonAvisar from "./BotonAvisar.svelte";

  let { arribo, ahora }: { arribo: Arribo; ahora: number } = $props();

  const clase = $derived(claseSituacion(arribo, ahora));
  // Con demora, la hora que importa es la estimada; la programada queda tachada.
  const demorado = $derived(arribo.demora_minutos !== null && arribo.demora_minutos > 0);
</script>

<div class="fila sit-{clase}">
  <span class="cuenta">{cuentaRegresiva(arribo.momento, ahora)}</span>
  <div class="detalle">
    <span class="destino">a {arribo.destino}</span>
    <span class="meta">
      <span class:estimada={demorado}>{hora(arribo.momento)}</span>
      {#if demorado}<s aria-label="programado {hora(arribo.programada)}">{hora(arribo.programada)}</s>{/if}
      {#if arribo.anden}<span>andén {arribo.anden}</span>{/if}
    </span>
  </div>
  <SituacionPill {arribo} {ahora} compacta />
  <BotonAvisar {arribo} />
</div>

<style>
  .fila {
    /* El ancho de la fila decide si la Situación entra con texto o solo con ícono. */
    container: fila / inline-size;
    display: grid;
    grid-template-columns: 4.2rem minmax(0, 1fr) auto auto;
    align-items: center;
    gap: 6px 10px;
    padding: 9px 6px 9px 14px;
    position: relative;
  }
  /* Barra con el color de la Situación: se lee de un vistazo al recorrer la lista. */
  .fila::before {
    content: "";
    position: absolute;
    left: 4px;
    top: 9px;
    bottom: 9px;
    width: 3px;
    border-radius: 3px;
    background: var(--sit);
    transition: background-color var(--mov-ui);
  }
  .cuenta {
    font-size: var(--t-cuerpo);
    font-weight: 700;
    font-variant-numeric: tabular-nums;
  }
  .sit-arribando .cuenta,
  .sit-en_anden .cuenta {
    color: var(--sit-texto);
  }
  .detalle {
    display: grid;
    min-width: 0;
  }
  .destino {
    font-weight: 600;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  /* Hora, demora y andén pasan de línea enteros: nunca se parten ni se montan
     sobre la Situación. */
  .meta {
    display: flex;
    flex-wrap: wrap;
    gap: 0 6px;
    white-space: nowrap;
    font-size: var(--t-meta);
    color: var(--texto-suave);
    font-variant-numeric: tabular-nums;
  }
  .estimada {
    font-weight: 600;
    color: var(--sit-texto);
  }
  s {
    color: var(--texto-tenue);
  }
  /* En el celular (o un panel angosto) la Situación queda en ícono y color. */
  @container fila (max-width: 340px) {
    .fila :global(.pill .texto) {
      display: none;
    }
    .fila :global(.pill) {
      padding: 4px;
    }
  }
  .sit-cancelado .cuenta,
  .sit-cancelado .destino,
  .sit-cancelado .meta {
    text-decoration: line-through;
    color: var(--texto-tenue);
  }
</style>

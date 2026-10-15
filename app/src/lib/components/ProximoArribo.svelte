<script lang="ts">
  import type { Arribo } from "$lib/api";
  import { claseSituacion, clave, segundosRestantes } from "$lib/arribos";
  import { detalle } from "$lib/detalle.svelte";
  import { hora, partesCuenta } from "$lib/formato";
  import SituacionPill from "./SituacionPill.svelte";
  import BotonAvisar from "./BotonAvisar.svelte";

  let { arribo, ahora }: { arribo: Arribo; ahora: number } = $props();

  const cuenta = $derived(partesCuenta(segundosRestantes(arribo, ahora)));
  const clase = $derived(claseSituacion(arribo, ahora));
  const demorado = $derived(arribo.demora_minutos !== null && arribo.demora_minutos > 0);
</script>

<article class="proximo sit-{clase}" aria-label="Próximo tren">
  <div class="encabezado">
    <span class="rotulo">Próximo tren</span>
    <div class="lado">
      <SituacionPill {arribo} {ahora} latir />
      <BotonAvisar {arribo} />
    </div>
  </div>
  <!-- El cuerpo de la tarjeta abre el detalle del Servicio. -->
  <button class="abrir" onclick={() => (detalle.clave = clave(arribo))} aria-haspopup="dialog">
  <span class="cuerpo">
    <span class="cuenta">
      <span class="valor">{cuenta.valor}</span>
      {#if cuenta.unidad}<span class="unidad">{cuenta.unidad}</span>{/if}
    </span>
    <span class="destino">
      <strong>a {arribo.destino}</strong>
      <span>{arribo.ramal.nombre}</span>
    </span>
  </span>
  <span class="datos">
    <span class="dato">
      <span class="dt">Horario</span>
      <span class="dd">
        <span class:estimada={demorado}>{hora(arribo.momento)}</span>
        {#if demorado}<s aria-label="programado {hora(arribo.programada)}">{hora(arribo.programada)}</s>{/if}
      </span>
    </span>
    <span class="dato">
      <span class="dt">Andén</span>
      <span class="dd anden" class:sin-asignar={!arribo.anden}>{arribo.anden ?? "Sin asignar"}</span>
    </span>
    <span class="dato">
      <span class="dt">Servicio</span>
      <span class="dd">#{arribo.numero}</span>
    </span>
  </span>
  </button>
</article>

<style>
  .proximo {
    border-radius: var(--radio-l);
    padding: 14px 16px 12px;
    background: color-mix(in srgb, var(--sit) 8%, var(--superficie));
    border: 1px solid color-mix(in srgb, var(--sit) 38%, transparent);
    transition:
      background-color var(--mov-ui),
      border-color var(--mov-ui);
  }
  .encabezado {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }
  .lado {
    display: flex;
    align-items: center;
    gap: 4px;
  }
  .rotulo {
    font-family: var(--fuente-chica);
    font-size: var(--t-micro);
    font-weight: 600;
    letter-spacing: 0.04em;
    text-transform: uppercase;
    color: var(--texto-suave);
  }
  .abrir {
    display: block;
    width: calc(100% + 16px);
    margin: 2px -8px -4px;
    padding: 4px 8px;
    border: none;
    border-radius: var(--radio-m);
    background: none;
    color: inherit;
    font: inherit;
    text-align: left;
    cursor: pointer;
    transition: background-color var(--mov-ui);
  }
  @media (hover: hover) {
    .abrir:hover {
      background: color-mix(in srgb, var(--sit) 7%, transparent);
    }
  }
  .cuerpo {
    display: flex;
    align-items: flex-end;
    gap: 14px;
    margin-top: 6px;
  }
  .cuenta {
    display: flex;
    align-items: baseline;
    gap: 4px;
    font-family: var(--fuente-display);
    color: var(--sit-texto);
    font-variant-numeric: tabular-nums;
    transition: color var(--mov-ui);
  }
  .valor {
    font-size: var(--t-cuenta);
    font-weight: 650;
    line-height: 1;
    letter-spacing: -0.02em;
  }
  .unidad {
    font-size: var(--t-cuerpo);
    font-weight: 600;
  }
  .destino {
    display: grid;
    gap: 2px;
    padding-bottom: 4px;
    min-width: 0;
  }
  .destino strong {
    font-family: var(--fuente-display);
    font-size: var(--t-titulo);
    font-weight: 600;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .destino span {
    font-size: var(--t-meta);
    color: var(--texto-suave);
  }
  .dato {
    display: grid;
  }
  .datos {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    margin: 12px 0 0;
    padding-top: 10px;
    border-top: 1px solid var(--borde);
  }
  .dt {
    font-family: var(--fuente-chica);
    font-size: var(--t-micro);
    color: var(--texto-tenue);
  }
  .dd {
    display: flex;
    flex-wrap: wrap;
    align-items: baseline;
    gap: 0 6px;
    margin: 0;
    font-weight: 600;
    font-variant-numeric: tabular-nums;
  }
  .estimada {
    color: var(--sit-texto);
  }
  s {
    font-size: var(--t-meta);
    font-weight: 400;
    color: var(--texto-tenue);
  }
  /* El andén es lo que se busca al llegar a la estación: un punto más grande. */
  .anden {
    font-family: var(--fuente-display);
    font-size: var(--t-titulo);
    line-height: 1.1;
  }
  .anden.sin-asignar {
    font-family: var(--fuente);
    font-size: var(--t-cuerpo);
    font-weight: 500;
    color: var(--texto-tenue);
  }
  .sit-cancelado .valor {
    text-decoration: line-through;
  }
</style>

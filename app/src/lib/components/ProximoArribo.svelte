<script lang="ts">
  import type { Arribo } from "$lib/api";
  import { claseSituacion, segundosRestantes } from "$lib/arribos";
  import { hora, partesCuenta } from "$lib/formato";
  import SituacionPill from "./SituacionPill.svelte";
  import BotonAvisar from "./BotonAvisar.svelte";

  let { arribo, ahora }: { arribo: Arribo; ahora: number } = $props();

  const cuenta = $derived(partesCuenta(segundosRestantes(arribo, ahora)));
  const clase = $derived(claseSituacion(arribo, ahora));
</script>

<article class="proximo sit-{clase}" aria-label="Próximo tren">
  <div class="encabezado">
    <span class="rotulo">Próximo tren</span>
    <div class="lado">
      <SituacionPill {arribo} {ahora} />
      <BotonAvisar {arribo} />
    </div>
  </div>
  <div class="cuerpo">
    <div class="cuenta">
      <span class="valor">{cuenta.valor}</span>
      {#if cuenta.unidad}<span class="unidad">{cuenta.unidad}</span>{/if}
    </div>
    <div class="destino">
      <strong>a {arribo.destino}</strong>
      <span>{arribo.ramal.nombre}</span>
    </div>
  </div>
  <dl class="datos">
    <div>
      <dt>Horario</dt>
      <dd>
        {hora(arribo.programada)}
        {#if arribo.demora_minutos !== null}<em>+{arribo.demora_minutos} min</em>{/if}
      </dd>
    </div>
    <div>
      <dt>Andén</dt>
      <dd>{arribo.anden ?? "Sin asignar"}</dd>
    </div>
    <div>
      <dt>Servicio</dt>
      <dd>#{arribo.numero}</dd>
    </div>
  </dl>
</article>

<style>
  .proximo {
    border-radius: 14px;
    padding: 14px 16px 12px;
    background: color-mix(in srgb, var(--sit) 9%, var(--superficie));
    border: 1px solid color-mix(in srgb, var(--sit) 40%, transparent);
    transition:
      background-color 0.5s ease,
      border-color 0.5s ease;
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
    font-size: 0.8rem;
    font-weight: 600;
    color: var(--texto-suave);
  }
  .cuerpo {
    display: flex;
    align-items: flex-end;
    gap: 14px;
    margin-top: 8px;
  }
  .cuenta {
    display: flex;
    align-items: baseline;
    gap: 4px;
    color: var(--sit);
    font-variant-numeric: tabular-nums;
    transition: color 0.5s ease;
  }
  .valor {
    font-size: 3rem;
    font-weight: 700;
    line-height: 1;
    letter-spacing: -0.02em;
  }
  .unidad {
    font-size: 1rem;
    font-weight: 600;
  }
  .destino {
    display: grid;
    gap: 2px;
    padding-bottom: 4px;
    min-width: 0;
  }
  .destino strong {
    font-size: 1.15rem;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .destino span {
    font-size: 0.8rem;
    color: var(--texto-suave);
  }
  .datos {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    margin: 12px 0 0;
    padding-top: 10px;
    border-top: 1px solid color-mix(in srgb, var(--sit) 25%, transparent);
  }
  dt {
    font-size: 0.7rem;
    color: var(--texto-suave);
  }
  dd {
    margin: 0;
    font-weight: 600;
    font-variant-numeric: tabular-nums;
  }
  em {
    font-style: normal;
    color: var(--demorado);
  }
  .sit-cancelado .valor {
    text-decoration: line-through;
  }
</style>

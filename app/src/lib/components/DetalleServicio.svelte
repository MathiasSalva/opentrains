<script lang="ts">
  import type { Arribo } from "$lib/api";
  import { claseSituacion } from "$lib/arribos";
  import { cuentaRegresiva, hora } from "$lib/formato";
  import SituacionPill from "./SituacionPill.svelte";
  import BotonAvisar from "./BotonAvisar.svelte";

  // Lo que se sabe de un Servicio en la Estación abierta. Acá va a sumarse el recorrido
  // parada por parada cuando exista.
  let { arribo, ahora }: { arribo: Arribo; ahora: number } = $props();

  const clase = $derived(claseSituacion(arribo, ahora));
  const demorado = $derived(arribo.demora_minutos !== null && arribo.demora_minutos > 0);
</script>

<div class="detalle sit-{clase}">
  <div class="resumen">
    <div>
      <p class="cuenta">{cuentaRegresiva(arribo.momento, ahora)}</p>
      <p class="ramal">{arribo.ramal.nombre}</p>
    </div>
    <div class="lado">
      <SituacionPill {arribo} {ahora} />
      <BotonAvisar {arribo} />
    </div>
  </div>
  <dl>
    <dt>Servicio</dt>
    <dd>#{arribo.numero}</dd>
    <dt>{demorado ? "Estimado" : "Horario"}</dt>
    <dd>
      {hora(arribo.momento)}
      {#if demorado}<span class="tenue">programado {hora(arribo.programada)}, +{arribo.demora_minutos} min</span>{/if}
    </dd>
    <dt>Andén</dt>
    <dd>{arribo.anden ?? "Sin asignar"}</dd>
    <dt>Formación</dt>
    <dd>{arribo.formacion ?? "Sin informar"}</dd>
    <dt>Estado</dt>
    <dd>{arribo.estado}</dd>
  </dl>
</div>

<style>
  .resumen {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 8px;
    padding-bottom: 12px;
    border-bottom: 1px solid var(--borde);
  }
  .cuenta {
    margin: 0;
    font-family: var(--fuente-display);
    font-size: 28px;
    font-weight: 650;
    color: var(--sit-texto);
    font-variant-numeric: tabular-nums;
  }
  .ramal {
    margin: 0;
    font-size: var(--t-meta);
    color: var(--texto-suave);
  }
  .lado {
    display: flex;
    align-items: center;
    gap: 4px;
  }
  dl {
    display: grid;
    grid-template-columns: auto 1fr;
    gap: 8px 16px;
    margin: 12px 0 0;
  }
  dt {
    font-family: var(--fuente-chica);
    font-size: var(--t-micro);
    color: var(--texto-tenue);
    align-self: center;
  }
  dd {
    display: flex;
    flex-wrap: wrap;
    gap: 0 8px;
    align-items: baseline;
    margin: 0;
    font-weight: 600;
    font-variant-numeric: tabular-nums;
  }
  .tenue {
    font-size: var(--t-meta);
    font-weight: 400;
    color: var(--texto-tenue);
  }
</style>

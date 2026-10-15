<script lang="ts">
  import TrainSimpleIcon from "phosphor-svelte/lib/TrainSimpleIcon";
  import ClockCountdownIcon from "phosphor-svelte/lib/ClockCountdownIcon";
  import NavigationArrowIcon from "phosphor-svelte/lib/NavigationArrowIcon";
  import XCircleIcon from "phosphor-svelte/lib/XCircleIcon";
  import ClockIcon from "phosphor-svelte/lib/ClockIcon";
  import InfoIcon from "phosphor-svelte/lib/InfoIcon";
  import type { Arribo } from "$lib/api";
  import { claseSituacion, etiquetaSituacion } from "$lib/arribos";

  let { arribo, ahora }: { arribo: Arribo; ahora: number } = $props();

  const ICONOS = {
    arribando: TrainSimpleIcon,
    en_anden: TrainSimpleIcon,
    demorado: ClockCountdownIcon,
    en_camino: NavigationArrowIcon,
    cancelado: XCircleIcon,
    programado: ClockIcon,
    desconocido: InfoIcon,
  };

  const clase = $derived(claseSituacion(arribo, ahora));
  const Icono = $derived(ICONOS[clase]);
  const etiqueta = $derived(etiquetaSituacion(arribo, ahora));
</script>

<!-- En filas angostas el texto se oculta (ver FilaArribo): el nombre accesible y el
     tooltip lo conservan. -->
<span class="pill sit-{clase}" role="img" aria-label={etiqueta} title={etiqueta}>
  <Icono size={14} weight="bold" />
  <span class="texto">{etiqueta}</span>
</span>

<style>
  .pill {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    padding: 3px 9px 3px 7px;
    border-radius: 999px;
    font-size: 0.75rem;
    font-weight: 600;
    white-space: nowrap;
    color: var(--sit);
    background: color-mix(in srgb, var(--sit) 16%, transparent);
    border: 1px solid color-mix(in srgb, var(--sit) 35%, transparent);
    transition:
      color 0.4s ease,
      background-color 0.4s ease,
      border-color 0.4s ease;
  }
  .sit-desconocido {
    border-style: dashed;
  }
  /* Lo programado es lo normal: se muestra sin relleno para que resalte lo que no lo es. */
  .sit-programado {
    background: none;
    border-color: transparent;
    font-weight: 500;
  }
  @media (prefers-reduced-motion: no-preference) {
    /* Lo que está por pasar late: es la información más urgente de la pantalla. */
    .sit-arribando,
    .sit-en_anden {
      animation: latido 1.6s ease-in-out infinite;
    }
  }
  @keyframes latido {
    50% {
      background: color-mix(in srgb, var(--sit) 32%, transparent);
    }
  }
</style>

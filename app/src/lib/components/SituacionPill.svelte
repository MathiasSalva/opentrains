<script lang="ts">
  import TrainSimpleIcon from "phosphor-svelte/lib/TrainSimpleIcon";
  import ClockCountdownIcon from "phosphor-svelte/lib/ClockCountdownIcon";
  import NavigationArrowIcon from "phosphor-svelte/lib/NavigationArrowIcon";
  import XCircleIcon from "phosphor-svelte/lib/XCircleIcon";
  import ClockIcon from "phosphor-svelte/lib/ClockIcon";
  import InfoIcon from "phosphor-svelte/lib/InfoIcon";
  import type { Arribo } from "$lib/api";
  import { claseSituacion, etiquetaSituacion } from "$lib/arribos";

  let {
    arribo,
    ahora,
    compacta = false,
    latir = false,
  }: {
    arribo: Arribo;
    ahora: number;
    /** Sin la demora en el texto: en las filas la hora ya la dice. */
    compacta?: boolean;
    /** Solo el tren destacado late: si laten todos, ninguno parece urgente. */
    latir?: boolean;
  } = $props();

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
  const etiqueta = $derived(etiquetaSituacion(arribo, ahora, compacta));
</script>

<!-- En filas angostas el texto se oculta (ver FilaArribo): el nombre accesible y el
     tooltip lo conservan. -->
<span
  class="pill sit-{clase}"
  class:latir
  role="img"
  aria-label={etiquetaSituacion(arribo, ahora)}
  title={etiquetaSituacion(arribo, ahora)}
>
  <Icono size={14} weight="bold" />
  <span class="texto">{etiqueta}</span>
</span>

<style>
  .pill {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    padding: 3px 9px 3px 7px;
    border-radius: var(--radio-pill);
    font-family: var(--fuente-chica);
    font-size: var(--t-micro);
    font-weight: 600;
    white-space: nowrap;
    color: var(--sit-texto);
    background: color-mix(in srgb, var(--sit) 15%, transparent);
    border: 1px solid color-mix(in srgb, var(--sit) 32%, transparent);
    transition:
      color var(--mov-ui),
      background-color var(--mov-ui),
      border-color var(--mov-ui);
  }
  .sit-desconocido {
    border-style: dashed;
  }
  /* Lo programado es lo normal: se muestra sin relleno para que resalte lo que no lo es. */
  .sit-programado {
    background: none;
    border-color: transparent;
    font-weight: 500;
    color: var(--texto-suave);
  }
  /* Lo que está por pasar late: es la información más urgente de la pantalla. */
  .latir.sit-arribando,
  .latir.sit-en_anden {
    animation: latido 1.6s ease-in-out infinite;
  }
  @keyframes latido {
    50% {
      background: color-mix(in srgb, var(--sit) 30%, transparent);
    }
  }
</style>

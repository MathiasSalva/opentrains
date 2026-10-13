<!-- Esqueleto con la forma de la vista final, mientras llega la primera respuesta. -->
<div class="esqueleto" aria-busy="true" aria-label="Cargando próximos trenes">
  <div class="bloque grande"></div>
  <div class="bloque"></div>
  <div class="bloque"></div>
  <div class="bloque"></div>
</div>

<style>
  .esqueleto {
    display: grid;
    gap: 8px;
  }
  .bloque {
    height: 48px;
    border-radius: 14px;
    background: var(--borde);
    opacity: 0.7;
  }
  .grande {
    height: 132px;
  }
  @media (prefers-reduced-motion: no-preference) {
    .bloque {
      animation: respirar 1.4s ease-in-out infinite;
    }
  }
  @keyframes respirar {
    50% {
      opacity: 0.35;
    }
  }
</style>

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
    border-radius: var(--radio-l);
    background: var(--superficie-hover);
  }
  .grande {
    height: 132px;
  }
  .bloque {
    animation: respirar 1.4s ease-in-out infinite;
  }
  @keyframes respirar {
    50% {
      opacity: 0.5;
    }
  }
</style>

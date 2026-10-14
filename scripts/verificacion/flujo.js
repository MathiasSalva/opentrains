(async () => {
  const esperar = (ms) => new Promise((r) => setTimeout(r, ms));
  const elegir = async (i, texto) => {
    const s = document.querySelectorAll("select")[i];
    const op = [...s.options].find((o) => o.text.startsWith(texto));
    s.value = op.value; s.selectedIndex = op.index;
    s.dispatchEvent(new Event("change", { bubbles: true }));
    await esperar(2500);
  };
  await elegir(0, "San Martín");
  await elegir(1, "Retiro - Junín");
  await elegir(2, "Retiro (LSM)");
  await esperar(2000);
  document.querySelector('button[title="Agregar a favoritos"]')?.click();
  await esperar(800);
  [...document.querySelectorAll("button.link")].find((b) => b.textContent.includes("Instalaciones"))?.click();
  await esperar(3000);
  return document.body.innerText;
})()

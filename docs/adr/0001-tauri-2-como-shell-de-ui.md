# Tauri 2 como shell de UI (frontend Svelte)

La app de escritorio usa Tauri 2 con un frontend Svelte + TypeScript, en vez de un toolkit nativo de Rust (Slint, egui, Iced). Slint era más liviano y nativo, pero un mapa con la traza de los Ramales y la posición en vivo de los Servicios va a llegar en algún momento: en Tauri sale casi gratis con Leaflet y los tiles de OSM de SOFSE, y en Slint habría que renderizar tiles a mano. Además, Tauri 2 compila para Android de forma oficial. El núcleo hexagonal (`crates/core`) no depende de Tauri, así que el shell se puede reemplazar sin tocar el dominio.

// Evita una consola extra en Windows en release. NO QUITAR.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    opentrains_app_lib::run()
}

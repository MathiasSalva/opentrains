//! Adaptador de salida contra la API no oficial de SOFSE (ver ADR-0002).

mod cliente;
pub mod credencial;
pub mod mapeo;

pub use cliente::SofseClient;

use std::time::Duration;

use chrono::{NaiveDate, Utc};
use opentrains_core::dominio::{
    Arribo, Estacion, EstacionId, Instalaciones, Linea, LineaId, Ramal, RamalId,
};
use opentrains_core::puertos::{ArribosSource, CatalogoSource, InstalacionesSource, SourceError};
use reqwest::StatusCode;
use serde::Deserialize;
use tokio::sync::Mutex;

use crate::{credencial, mapeo};

const API: &str = "https://api-servicios.sofse.gob.ar/v1/";
const INFOESTACIONES: &str = "https://infoestaciones.sofse.gob.ar/public/app-estaciones/";
const TIMEOUT: Duration = Duration::from_secs(8);

/// Cliente de la API de SOFSE. Obtiene y cachea el token del día, y lo renueva una
/// vez ante un 401 antes de dar la fuente por cambiada.
pub struct SofseClient {
    http: reqwest::Client,
    api: String,
    infoestaciones: String,
    token: Mutex<Option<Token>>,
}

struct Token {
    valor: String,
    fecha: NaiveDate,
}

#[derive(Deserialize)]
struct RespuestaToken {
    token: String,
}

impl Default for SofseClient {
    fn default() -> Self {
        Self::con_urls(API.into(), INFOESTACIONES.into())
    }
}

impl SofseClient {
    pub fn con_urls(api: String, infoestaciones: String) -> Self {
        let http = reqwest::Client::builder()
            .timeout(TIMEOUT)
            .build()
            .expect("cliente HTTP válido");
        Self {
            http,
            api,
            infoestaciones,
            token: Mutex::new(None),
        }
    }

    async fn get_autenticado(&self, ruta: &str) -> Result<String, SourceError> {
        let url = format!("{}{ruta}", self.api);
        let token = self.token(false).await?;
        match self.get(&url, Some(&token)).await {
            Err(Rejection::NoAutorizado) => {
                let token = self.token(true).await?;
                self.get(&url, Some(&token))
                    .await
                    .map_err(Rejection::en_error)
            }
            otro => otro.map_err(Rejection::en_error),
        }
    }

    async fn get(&self, url: &str, token: Option<&str>) -> Result<String, Rejection> {
        let mut pedido = self.http.get(url);
        if let Some(token) = token {
            pedido = pedido.header("Authorization", token);
        }
        let respuesta = pedido.send().await.map_err(sin_conexion)?;
        match respuesta.status() {
            StatusCode::UNAUTHORIZED => Err(Rejection::NoAutorizado),
            s if s.is_success() => respuesta.text().await.map_err(|e| sin_conexion(e).into()),
            s => Err(Rejection::Error(SourceError::RespuestaInvalida(format!(
                "HTTP {s} en {url}"
            )))),
        }
    }

    /// Token vigente para hoy; `renovar` fuerza un login nuevo.
    async fn token(&self, renovar: bool) -> Result<String, SourceError> {
        let hoy = credencial::fecha_argentina(Utc::now());
        let mut cache = self.token.lock().await;
        if let Some(token) = cache.as_ref().filter(|t| !renovar && t.fecha == hoy) {
            return Ok(token.valor.clone());
        }
        let (username, password) = credencial::para_fecha(hoy);
        let respuesta = self
            .http
            .post(format!("{}auth/authorize", self.api))
            .json(&serde_json::json!({ "username": username, "password": password }))
            .send()
            .await
            .map_err(sin_conexion)?;
        if respuesta.status() == StatusCode::UNAUTHORIZED {
            return Err(SourceError::FuenteCambiada);
        }
        if !respuesta.status().is_success() {
            return Err(SourceError::RespuestaInvalida(format!(
                "HTTP {} en login",
                respuesta.status()
            )));
        }
        let RespuestaToken { token } = respuesta
            .json()
            .await
            .map_err(|e| SourceError::RespuestaInvalida(e.to_string()))?;
        *cache = Some(Token {
            valor: token.clone(),
            fecha: hoy,
        });
        Ok(token)
    }
}

enum Rejection {
    NoAutorizado,
    Error(SourceError),
}

impl From<SourceError> for Rejection {
    fn from(error: SourceError) -> Self {
        Rejection::Error(error)
    }
}

impl Rejection {
    fn en_error(self) -> SourceError {
        match self {
            Rejection::NoAutorizado => SourceError::FuenteCambiada,
            Rejection::Error(e) => e,
        }
    }
}

fn sin_conexion(error: reqwest::Error) -> SourceError {
    SourceError::SinConexion(error.to_string())
}

impl CatalogoSource for SofseClient {
    async fn lineas(&self) -> Result<Vec<Linea>, SourceError> {
        mapeo::lineas(
            &self
                .get_autenticado("infraestructura/gerencias?idEmpresa=1")
                .await?,
        )
    }

    async fn ramales(&self, linea: LineaId) -> Result<Vec<Ramal>, SourceError> {
        let ruta = format!("infraestructura/ramales?idGerencia={}", linea.0);
        mapeo::ramales(&self.get_autenticado(&ruta).await?)
    }

    async fn estaciones(&self, ramal: RamalId) -> Result<Vec<Estacion>, SourceError> {
        let ruta = format!("infraestructura/estaciones?idRamal={}", ramal.0);
        mapeo::estaciones(&self.get_autenticado(&ruta).await?)
    }
}

impl ArribosSource for SofseClient {
    async fn arribos(
        &self,
        estacion: EstacionId,
        ramal: Option<RamalId>,
    ) -> Result<Vec<Arribo>, SourceError> {
        let mut ruta = format!("arribos/estacion/{}", estacion.0);
        if let Some(ramal) = ramal {
            ruta.push_str(&format!("?ramal={}", ramal.0));
        }
        let json = self.get_autenticado(&ruta).await?;
        mapeo::arribos(&json, Utc::now())
    }
}

impl InstalacionesSource for SofseClient {
    async fn instalaciones(
        &self,
        estacion: EstacionId,
    ) -> Result<Option<Instalaciones>, SourceError> {
        // Endpoint público: no lleva token.
        let url = format!("{}{}", self.infoestaciones, estacion.0);
        let json = self.get(&url, None).await.map_err(Rejection::en_error)?;
        mapeo::instalaciones(&json)
    }
}

use opentrains_core::dominio::{EstacionId, LineaId, RamalId};
use opentrains_core::puertos::{ArribosSource, CatalogoSource, InstalacionesSource, SourceError};
use opentrains_sofse::SofseClient;
use wiremock::matchers::{header, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn fixture(nombre: &str) -> String {
    std::fs::read_to_string(format!(
        "{}/tests/fixtures/{nombre}",
        env!("CARGO_MANIFEST_DIR")
    ))
    .unwrap()
}

async fn servidor_con_token(token: &str) -> MockServer {
    let servidor = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/auth/authorize"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(serde_json::json!({ "token": token })),
        )
        .mount(&servidor)
        .await;
    servidor
}

fn cliente(servidor: &MockServer) -> SofseClient {
    SofseClient::con_urls(
        format!("{}/v1/", servidor.uri()),
        format!("{}/public/app-estaciones/", servidor.uri()),
    )
}

#[tokio::test]
async fn las_consultas_envian_el_token_crudo_en_authorization() {
    let servidor = servidor_con_token("tok-1").await;
    Mock::given(method("GET"))
        .and(path("/v1/infraestructura/ramales"))
        .and(query_param("idGerencia", "31"))
        .and(header("Authorization", "tok-1"))
        .respond_with(ResponseTemplate::new(200).set_body_string(fixture("ramales.json")))
        .mount(&servidor)
        .await;

    let ramales = cliente(&servidor).ramales(LineaId(31)).await.unwrap();

    assert_eq!(ramales.len(), 2);
}

#[tokio::test]
async fn el_token_se_reutiliza_entre_consultas() {
    let servidor = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/auth/authorize"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(serde_json::json!({ "token": "tok" })),
        )
        .expect(1)
        .mount(&servidor)
        .await;
    Mock::given(method("GET"))
        .and(path("/v1/infraestructura/gerencias"))
        .respond_with(ResponseTemplate::new(200).set_body_string(fixture("gerencias.json")))
        .mount(&servidor)
        .await;
    let cliente = cliente(&servidor);

    cliente.lineas().await.unwrap();
    cliente.lineas().await.unwrap();
    // `expect(1)` se verifica al soltar el servidor.
}

#[tokio::test]
async fn ante_un_401_se_renueva_el_token_y_se_reintenta_una_vez() {
    let servidor = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/auth/authorize"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(serde_json::json!({ "token": "viejo" })),
        )
        .up_to_n_times(1)
        .mount(&servidor)
        .await;
    Mock::given(method("POST"))
        .and(path("/v1/auth/authorize"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(serde_json::json!({ "token": "nuevo" })),
        )
        .mount(&servidor)
        .await;
    Mock::given(method("GET"))
        .and(path("/v1/arribos/estacion/463"))
        .and(header("Authorization", "viejo"))
        .respond_with(ResponseTemplate::new(401))
        .mount(&servidor)
        .await;
    Mock::given(method("GET"))
        .and(path("/v1/arribos/estacion/463"))
        .and(query_param("ramal", "65"))
        .and(header("Authorization", "nuevo"))
        .respond_with(ResponseTemplate::new(200).set_body_string(fixture("arribos.json")))
        .mount(&servidor)
        .await;

    let arribos = cliente(&servidor)
        .arribos(EstacionId(463), Some(RamalId(65)))
        .await
        .unwrap();

    assert_eq!(arribos.len(), 3);
}

#[tokio::test]
async fn un_401_persistente_significa_que_la_fuente_cambio() {
    let servidor = servidor_con_token("tok").await;
    Mock::given(method("GET"))
        .and(path("/v1/infraestructura/gerencias"))
        .respond_with(ResponseTemplate::new(401))
        .mount(&servidor)
        .await;

    assert_eq!(
        cliente(&servidor).lineas().await,
        Err(SourceError::FuenteCambiada)
    );
}

#[tokio::test]
async fn si_el_login_es_rechazado_la_fuente_cambio() {
    let servidor = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/auth/authorize"))
        .respond_with(
            ResponseTemplate::new(401).set_body_json(
                serde_json::json!({ "code": 401, "message": "Contraseña incorrecta" }),
            ),
        )
        .mount(&servidor)
        .await;

    assert_eq!(
        cliente(&servidor).lineas().await,
        Err(SourceError::FuenteCambiada)
    );
}

#[tokio::test]
async fn sin_servidor_es_sin_conexion() {
    let cliente = SofseClient::con_urls(
        "http://127.0.0.1:9/v1/".into(),
        "http://127.0.0.1:9/".into(),
    );

    assert!(matches!(
        cliente.lineas().await,
        Err(SourceError::SinConexion(_))
    ));
}

#[tokio::test]
async fn las_instalaciones_se_consultan_sin_token() {
    let servidor = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/public/app-estaciones/463"))
        .respond_with(ResponseTemplate::new(200).set_body_string(fixture("instalaciones.json")))
        .mount(&servidor)
        .await;

    let instalaciones = cliente(&servidor)
        .instalaciones(EstacionId(463))
        .await
        .unwrap();

    assert!(instalaciones.unwrap().rampa);
    // Ningún login: el único mock montado es el de instalaciones.
}

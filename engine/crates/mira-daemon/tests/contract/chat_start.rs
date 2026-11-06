//! Moteur avec les six fournisseurs d'IA et un faux serveur d'IA sur la boucle locale, connexion prête.

use mira_assembly::api::{AiRequest, Request, SecretInput};
use mira_assembly::mira_vocabulary::{AiConnectionId, AiProviderId};
use mira_testkit::ai::FakeAiServer;
use serde_json::{Value, json};

use super::chat_fixture::ai_config;
use super::driver::Driver;
use super::mail_start::send;

/// Moteur ouvert, connexion prête et faux serveur d'IA.
pub struct ChatFixture<D> {
    pub driver: D,
    pub server: FakeAiServer,
    pub connection: String,
    pub home: tempfile::TempDir,
}

/// Exécute une requête du domaine IA par ce chemin et rend la donnée de la réponse.
pub async fn ask_ai<D: Driver>(driver: &D, request: AiRequest) -> Value {
    let request = serde_json::to_value(Request::Ai(request)).unwrap();
    let reply = send(driver, &request).await;
    assert_eq!(reply["ok"], true, "{reply}");
    reply["data"]["data"]["data"].clone()
}

/// Ouvre le moteur et crée une connexion prête vers le faux serveur.
pub async fn start<D: Driver>() -> ChatFixture<D> {
    let server = FakeAiServer::start().await.unwrap();
    let home = tempfile::tempdir().unwrap();
    let driver = D::start(Some(ai_config(home.path()))).await;
    let connection = ready_connection(&driver, &server.base_url()).await;
    ChatFixture {
        driver,
        server,
        connection,
        home,
    }
}

/// Crée une connexion générique vers cette adresse, avec clé et modèle par défaut ; rend son identifiant.
async fn ready_connection<D: Driver>(driver: &D, base_url: &str) -> String {
    let label = "Local".to_string();
    let add = AiRequest::AddAiConnection {
        provider: AiProviderId("generic".to_string()),
        label: label.clone(),
        base_url: Some(base_url.to_string()),
    };
    let id = ask_ai(driver, add).await["id"]
        .as_str()
        .unwrap()
        .to_string();
    let connection = AiConnectionId::from_raw(id.clone());
    let key = SecretInput::new("sk-BRIDGE-SECRET-KEY");
    ask_ai(
        driver,
        AiRequest::SetAiApiKey {
            connection: connection.clone(),
            key,
        },
    )
    .await;
    let update = AiRequest::UpdateAiConnection {
        connection,
        label,
        base_url: Some(base_url.to_string()),
        default_model: Some("model-a".to_string()),
    };
    ask_ai(driver, update).await;
    id
}

/// Statut JSON d'une opération, lu par le contrat.
pub async fn operation_status<D: Driver>(driver: &D, operation: &str) -> Value {
    let request =
        json!({"domain": "operation", "request": {"op": "get_operation", "operation": operation}});
    send(driver, &request).await["data"]["data"]["status"].clone()
}

/// Lance `pump_chat`, note chaque ligne reçue et rend toutes les lignes en JSON.
pub async fn collect_lines<D: Driver>(driver: &D, request_json: &str) -> Vec<Value> {
    let mut lines = Vec::new();
    driver
        .pump_chat(request_json, |line| {
            lines.push(serde_json::from_str(&line).unwrap());
            true
        })
        .await;
    lines
}

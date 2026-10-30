#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, dead_code)]
//! Moteur ouvert avec les six fournisseurs d'IA et un faux serveur d'IA sur la boucle locale.

use mira_assembly::api::{AiRequest, ChatStart, Request, SecretInput};
use mira_assembly::build_ai_registry;
use mira_assembly::mira_vocabulary::{
    AiConnectionId, AiProviderId, ChatMessage, ChatRequest, ChatRole,
};
use mira_bridge::runtime;
use mira_testkit::ai::FakeAiServer;
use serde_json::{Value, json};
use tokio::sync::{Mutex, MutexGuard};

use super::{config, send};

pub static SESSION: Mutex<()> = Mutex::const_new(());

/// Moteur ouvert (un seul à la fois dans le processus), connexion prête et faux serveur.
pub struct ChatFixture {
    pub server: FakeAiServer,
    pub connection: String,
    pub home: tempfile::TempDir,
    pub guard: MutexGuard<'static, ()>,
}

/// Exécute une requête du domaine Ai par le pont et rend la donnée de la réponse.
pub async fn ask_ai(request: AiRequest) -> Value {
    let request = serde_json::to_value(Request::Ai(request)).unwrap();
    let reply = send(&request).await;
    assert_eq!(reply["ok"], true, "{reply}");
    reply["data"]["data"]["data"].clone()
}

/// Ouvre le moteur et crée une connexion prête vers le faux serveur.
pub async fn start() -> ChatFixture {
    let guard = SESSION.lock().await;
    let server = FakeAiServer::start().await.unwrap();
    let home = tempfile::tempdir().unwrap();
    let mut config = config(home.path(), 1, 1);
    config.ai_registry = build_ai_registry();
    assert!(runtime::open(config).contains("\"ok\":true"));
    let connection = ready_connection(&server.base_url()).await;
    ChatFixture {
        server,
        connection,
        home,
        guard,
    }
}

/// Crée une connexion générique vers cette adresse, avec clé et modèle par défaut ; rend son identifiant.
async fn ready_connection(base_url: &str) -> String {
    let provider = AiProviderId("generic".to_string());
    let base_url = Some(base_url.to_string());
    let label = "Local".to_string();
    let add = AiRequest::AddAiConnection {
        provider,
        label: label.clone(),
        base_url: base_url.clone(),
    };
    let id = ask_ai(add).await["id"].as_str().unwrap().to_string();
    let connection = AiConnectionId::from_raw(id.clone());
    let key = SecretInput::new("sk-BRIDGE-SECRET-KEY");
    ask_ai(AiRequest::SetAiApiKey {
        connection: connection.clone(),
        key,
    })
    .await;
    let default_model = Some("model-a".to_string());
    let update = AiRequest::UpdateAiConnection {
        connection,
        label,
        base_url,
        default_model,
    };
    ask_ai(update).await;
    id
}

/// Demande JSON d'une discussion d'un message sur cette connexion (`None` : connexion par défaut).
pub fn chat_json(connection: Option<&str>, text: &str) -> String {
    let message = ChatMessage::plain(ChatRole::User, text.to_string());
    let request = ChatRequest {
        model: None,
        system: None,
        messages: vec![message],
        max_output_tokens: None,
        temperature: None,
        tools: Vec::new(),
        context: Vec::new(),
    };
    let connection = connection.map(AiConnectionId::from_raw);
    serde_json::to_string(&ChatStart {
        connection,
        request,
    })
    .unwrap()
}

/// Statut JSON d'une opération, lu par le contrat.
pub async fn operation_status(operation: &str) -> Value {
    let request =
        json!({"domain": "operation", "request": {"op": "get_operation", "operation": operation}});
    let reply = send(&request).await;
    reply["data"]["data"]["status"].clone()
}

/// Lance `pump_chat`, note chaque ligne reçue et rend toutes les lignes en JSON.
pub async fn collect_lines(request_json: &str) -> Vec<Value> {
    let mut lines = Vec::new();
    runtime::pump_chat(request_json, |line| {
        lines.push(serde_json::from_str(&line).unwrap());
        true
    })
    .await;
    lines
}

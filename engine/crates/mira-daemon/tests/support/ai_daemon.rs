//! Démon muni des fournisseurs d'IA et d'un faux serveur d'IA, avec une connexion prête.

use mira_assembly::api::{AiRequest, Request, SecretInput};
use mira_assembly::mira_vocabulary::{AiConnectionId, AiProviderId};
use mira_testkit::ai::FakeAiServer;
use serde_json::Value;

use super::chat_fixture::ai_config;
use super::{TestClient, TestDaemon};

/// Démon d'IA : le faux serveur, la connexion prête et le dossier jetable.
pub struct AiDaemon {
    pub daemon: TestDaemon,
    pub server: FakeAiServer,
    pub connection: String,
    pub home: tempfile::TempDir,
    pub setup_bytes: Vec<u8>,
}

impl AiDaemon {
    /// Démarre le faux serveur et le démon, puis crée une connexion générique avec clé et modèle.
    pub async fn start() -> Self {
        let server = FakeAiServer::start().await.unwrap();
        let home = tempfile::tempdir().unwrap();
        let daemon = TestDaemon::start(ai_config(home.path())).await;
        let mut client = TestClient::connect(&daemon).await;
        client.hello_ok(daemon.token()).await;
        let connection = ready_connection(&mut client, &server.base_url()).await;
        let setup_bytes = client.received_bytes().to_vec();
        Self {
            daemon,
            server,
            connection,
            home,
            setup_bytes,
        }
    }

    /// Nouveau client attaché au démon.
    pub async fn attach(&self) -> TestClient {
        let mut client = TestClient::connect(&self.daemon).await;
        client.hello_ok(self.daemon.token()).await;
        client
    }
}

/// Exécute une requête du domaine IA et rend la donnée de la réponse.
pub async fn ask_ai(client: &mut TestClient, request: AiRequest) -> Value {
    let reply = client
        .call(serde_json::to_value(Request::Ai(request)).unwrap())
        .await;
    assert_eq!(reply["ok"], true, "{reply}");
    reply["data"]["data"]["data"].clone()
}

/// Crée une connexion générique vers cette adresse, avec clé et modèle par défaut ; rend son identifiant.
async fn ready_connection(client: &mut TestClient, base_url: &str) -> String {
    let label = "Local".to_string();
    let add = AiRequest::AddAiConnection {
        provider: AiProviderId("generic".to_string()),
        label: label.clone(),
        base_url: Some(base_url.to_string()),
    };
    let id = ask_ai(client, add).await["id"]
        .as_str()
        .unwrap()
        .to_string();
    let connection = AiConnectionId::from_raw(id.clone());
    let key = SecretInput::new("sk-BRIDGE-SECRET-KEY");
    ask_ai(
        client,
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
    ask_ai(client, update).await;
    id
}

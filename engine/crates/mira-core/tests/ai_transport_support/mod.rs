//! Descripteur, clé, limites réduites et lecture complète partagés par les tests du transport.
#![allow(dead_code)]

use std::time::Duration;

use mira_core::ai::transport::{AiStreamBody, TransportLimits};
use mira_core::ai::wire::HttpCall;
use mira_core::ports::SecretValue;
use mira_testkit::ai::FakeAiServer;
use mira_vocabulary::{
    AiError, AiProviderDescriptor, AiProviderId, AiQuirks, AuthScheme, WireFormat,
};

pub const KEY: &str = "sk-test-SECRET-KEY-0123456789";

/// Descripteur générique présentant la clé selon le schéma donné.
pub fn descriptor(auth: AuthScheme) -> AiProviderDescriptor {
    AiProviderDescriptor {
        id: AiProviderId("test-provider".to_string()),
        display_name: "Test".to_string(),
        base_url: "http://127.0.0.1:9".to_string(),
        auth,
        format: WireFormat::OpenAiChat,
        models_path: "/models".to_string(),
        model_exclusions: Vec::new(),
        static_headers: vec![("x-test".to_string(), "1".to_string())],
        quirks: AiQuirks::default(),
        base_url_editable: false,
    }
}

/// Clé d'essai enveloppée comme un secret.
pub fn key() -> SecretValue {
    SecretValue::new(KEY)
}

/// Limites réduites à quelques centaines de millisecondes pour des tests rapides.
pub fn fast_limits() -> TransportLimits {
    TransportLimits {
        connect: Duration::from_secs(2),
        first_byte: Duration::from_millis(200),
        idle: Duration::from_millis(200),
        max_body: 2 * 1024 * 1024,
        max_stream: 8 * 1024 * 1024,
    }
}

/// Appel de discussion tel qu'un format l'aurait encodé.
pub fn call() -> HttpCall {
    HttpCall {
        path: "/chat/completions".to_string(),
        headers: vec![
            ("content-type".to_string(), "application/json".to_string()),
            ("accept".to_string(), "text/event-stream".to_string()),
            ("x-test".to_string(), "1".to_string()),
        ],
        body: r#"{"model":"m1","stream":true}"#.to_string(),
    }
}

/// Lit le flux jusqu'au bout et concatène les morceaux reçus.
pub async fn read_all(body: &mut AiStreamBody) -> Result<Vec<u8>, AiError> {
    let mut all = Vec::new();
    while let Some(chunk) = body.next_chunk().await? {
        all.extend_from_slice(&chunk);
    }
    Ok(all)
}

/// Attend, dans une limite de 10 s, que le serveur compte une fermeture par le client.
pub async fn wait_closed_by_client(server: &FakeAiServer) -> usize {
    for _ in 0..200 {
        let closed = server.connections_closed_by_client();
        if closed > 0 {
            return closed;
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    server.connections_closed_by_client()
}

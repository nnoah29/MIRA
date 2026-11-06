//! Échanges usuels du client de test : poignée de main valide et appel apparié par identifiant.

use mira_assembly::mira_vocabulary::{CONTRACT_VERSION, ContractVersion};
use mira_protocol::{Auth, Frame, PROTOCOL_VERSION, ProtocolVersion};
use serde_json::Value;

use super::TestClient;

/// Trame `hello` valide avec ce jeton.
pub fn valid_hello(token: &str) -> Frame {
    hello_with(token, PROTOCOL_VERSION, CONTRACT_VERSION)
}

/// Trame `hello` avec ce jeton et ces versions.
pub fn hello_with(token: &str, protocol: ProtocolVersion, contract: ContractVersion) -> Frame {
    Frame::Hello {
        protocol_version: protocol,
        contract_version: contract,
        client_id: "test-client".into(),
        client_name: "tests".into(),
        auth: Auth::LocalToken {
            token: token.into(),
        },
    }
}

impl TestClient {
    /// Fait la poignée de main avec le vrai jeton et exige `hello_ok`.
    pub async fn hello_ok(&mut self, token: &str) -> Frame {
        self.send(valid_hello(token)).await;
        let reply = self.recv().await.expect("hello_ok attendu");
        assert!(matches!(reply, Frame::HelloOk { .. }), "{reply:?}");
        reply
    }

    /// Envoie un appel avec un identifiant neuf et rend l'identifiant.
    pub async fn send_call(&mut self, request: Value) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        self.send(Frame::Call { id, request }).await;
        id
    }

    /// Envoie un appel et rend l'enveloppe de sa réponse.
    pub async fn call(&mut self, request: Value) -> Value {
        let id = self.send_call(request).await;
        match self.recv().await {
            Some(Frame::Result { id: got, envelope }) if got == id => envelope,
            other => panic!("réponse inattendue : {other:?}"),
        }
    }
}

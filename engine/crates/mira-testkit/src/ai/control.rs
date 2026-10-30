//! Adresses du faux serveur d'IA et pilotage par les tests : comportements, requêtes, déconnexions.

use std::net::SocketAddr;

use super::behavior::Behavior;
use super::recorded::RecordedRequest;
use super::server::FakeAiServer;

impl FakeAiServer {
    /// Adresse d'écoute, toujours sur la boucle locale
    pub fn local_addr(&self) -> SocketAddr {
        self.addr
    }

    /// URL de base `http://127.0.0.1:<port>`
    pub fn base_url(&self) -> String {
        format!("http://127.0.0.1:{}", self.addr.port())
    }

    /// Met un comportement en file ; consommés dans l'ordre, le dernier se répète
    pub fn queue(&self, behavior: Behavior) {
        self.shared.with(|state| state.enqueue(behavior));
    }

    /// Requêtes reçues, dans l'ordre d'arrivée
    pub fn requests(&self) -> Vec<RecordedRequest> {
        self.shared.with(|state| state.requests.clone())
    }

    /// Connexions que le client a fermées ou fait échouer avant la fin de la réponse
    pub fn connections_closed_by_client(&self) -> usize {
        self.shared.with(|state| state.closed_by_client)
    }
}

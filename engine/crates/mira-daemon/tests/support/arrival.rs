//! Arrivée ordonnée des connexions : le test attend que le démon ait inscrit la précédente avant d'ouvrir la suivante.

use std::time::Duration;

use super::{TestClient, TestDaemon};

const ARRIVAL_DEADLINE: Duration = Duration::from_secs(5);

impl TestDaemon {
    /// Ouvre une connexion et rend la main quand le démon l'a inscrite en attente de son `hello` ; l'ordre d'arrivée est alors celui des appels.
    pub async fn connect_registered(&self) -> TestClient {
        let before = self.context.pending.registered();
        let client = TestClient::connect(self).await;
        let waiting = async {
            while self.context.pending.registered() == before {
                tokio::task::yield_now().await;
            }
        };
        let arrived = tokio::time::timeout(ARRIVAL_DEADLINE, waiting).await;
        assert!(arrived.is_ok(), "connexion jamais inscrite par le démon");
        client
    }
}

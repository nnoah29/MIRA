//! Parcours complet d'autorisation contre le faux serveur OAuth, navigateur simulé compris.

use std::time::Duration;

use mira_core::auth::{PendingFlow, TokenSet, complete_flow, start_flow, wait_for_redirect};
use mira_core::ports::ManualClock;
use mira_testkit::oauth::FakeOAuthServer;
use mira_vocabulary::Timestamp;

use super::{CLIENT_ID, USER, oauth_settings, seeded_rng};

/// Horloge de test arrêtée à 1 000 secondes après l'époque
pub fn clock() -> ManualClock {
    ManualClock::new(Timestamp::from_millis(1_000_000))
}

/// Déroule le flux complet avec approbation simulée et renvoie le flux et les jetons obtenus
pub async fn approved_flow(server: &FakeOAuthServer, seed: u64) -> (PendingFlow, TokenSet) {
    let mut rng = seeded_rng(seed);
    let mut flow = start_flow(&oauth_settings(server), CLIENT_ID, USER, &mut rng)
        .await
        .unwrap();
    let url = flow.authorize_url().to_string();
    let (approved, code) = tokio::join!(
        server.approve(&url, USER),
        wait_for_redirect(&mut flow, Duration::from_secs(5))
    );
    approved.unwrap();
    let tokens = complete_flow(&mut flow, &code.unwrap(), &clock())
        .await
        .unwrap();
    (flow, tokens)
}

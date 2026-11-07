#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Issue de la ligne de journal selon le statut HTTP refusé par le service : cause lisible, sans contenu.

mod ai_chat_support;
mod ai_models_support;
mod support_api;

use ai_chat_support::{collect, journal_rows, loopback_stream};
use ai_models_support::{ModelsFixture, TestProvider};
use mira_testkit::ai::Behavior;

/// Réponse d'échec de ce statut
fn refusal(code: u16) -> Behavior {
    Behavior::Status {
        code,
        headers: Vec::new(),
        body: r#"{"error":{"message":"refus"}}"#.to_string(),
    }
}

/// Issue gardée par le journal après un refus de ce statut
async fn outcome_after(code: u16) -> String {
    let fixture = ModelsFixture::start(TestProvider::new("test-provider")).await;
    let mut stream = loopback_stream(&fixture, refusal(code), "Salut").await;
    collect(&mut stream).await;
    journal_rows(&fixture)[0].outcome.clone()
}

/// Un statut sans variante propre garde le code de réponse inattendue et le statut
#[tokio::test]
async fn unexpected_status_is_recorded_with_its_number() {
    assert_eq!(outcome_after(405).await, "server.ai_bad_response:http_405");
}

/// Un 402 dit que le crédit ou le quota est épuisé, un 408 que le service est indisponible
#[tokio::test]
async fn payment_required_and_timeout_have_their_own_outcome() {
    assert_eq!(outcome_after(402).await, "server.ai_quota_exhausted");
    assert_eq!(outcome_after(408).await, "server.ai_unavailable");
}

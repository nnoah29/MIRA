#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Contrat 1.4 : le contenu d'une conversation n'apparaît ni dans le journal, ni dans les événements, ni dans les erreurs.

mod support_ai_conversations;
mod support_api;

use mira_core::api::{AiRequest, Request};
use support_ai_conversations::history::set_enabled;
use support_ai_conversations::{list, save, start};
use support_api::leaks::{events_text, failure_text};

const SECRET_PAYLOAD: &str = "SECRET-PAYLOAD-123";
const SECRET_TITLE: &str = "SECRET-TITLE-456";

/// Requête d'enregistrement portant les deux secrets
fn save_request(title: &str, payload: &str) -> Request {
    Request::Ai(AiRequest::SaveAiConversation {
        id: "c-1".into(),
        title: title.into(),
        payload_version: 1,
        payload: payload.into(),
    })
}

/// Une conversation enregistrée ne laisse rien dans le journal des envois ni dans les événements
#[tokio::test]
async fn saved_content_never_reaches_journal_or_events() {
    let f = start();
    let payload = format!("{{\"text\":\"{SECRET_PAYLOAD}\"}}");
    save(&f.engine, "c-1", SECRET_TITLE, &payload)
        .await
        .unwrap();
    let journal = Request::Ai(AiRequest::ListAiJournal {
        limit: 50,
        before: None,
    });
    let journal_text = format!("{:?}", f.engine.execute(journal).await.unwrap());
    let events = events_text(&f.engine).await;
    for text in [journal_text, events] {
        assert!(!text.contains(SECRET_PAYLOAD), "{text}");
        assert!(!text.contains(SECRET_TITLE), "{text}");
    }
    assert_eq!(list(&f.engine, 50, None).await.unwrap().len(), 1);
}

/// Les refus (contenu invalide, trop gros, historique coupé) ne répètent ni le titre ni le contenu
#[tokio::test]
async fn refusals_do_not_echo_title_or_payload() {
    let f = start();
    let huge = format!(
        "{{\"k\":\"{SECRET_PAYLOAD}{}\"}}",
        "a".repeat(2 * 1024 * 1024)
    );
    let not_object = format!("[\"{SECRET_PAYLOAD}\"]");
    let long_title = SECRET_TITLE.repeat(10);
    let cases = [
        ("T".to_string(), huge),
        ("T".to_string(), not_object),
        (long_title, "{}".to_string()),
    ];
    let mut texts = Vec::new();
    for (title, payload) in &cases {
        texts.push(failure_text(&f.engine, save_request(title, payload)).await);
    }
    set_enabled(&f.engine, false).await.unwrap();
    let body = format!("{{\"k\":\"{SECRET_PAYLOAD}\"}}");
    texts.push(failure_text(&f.engine, save_request(SECRET_TITLE, &body)).await);
    for text in texts {
        assert!(!text.contains(SECRET_PAYLOAD), "{text}");
        assert!(!text.contains(SECRET_TITLE), "{text}");
    }
}

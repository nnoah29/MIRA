#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Masquage d'une génération : le serveur ne voit jamais la valeur en clair, la réponse la retrouve.

mod ai_chat_support;
mod ai_models_support;
#[allow(dead_code)]
mod support_ai_context;
mod support_api;
mod support_masking;

use ai_chat_support::{
    allow_kinds, answer_of, chat_start_with, connection_with, journal_rows, run_chat,
};
use ai_models_support::{ModelsFixture, TestProvider};
use mira_testkit::ai::openai_stream;
use mira_vocabulary::{ChatDelta, ContentKind, MaskingMode, MaskingSettings};
use support_ai_context::part;
use support_masking::mask_on;

const IBAN: &str = "FR7630006000011234567890189";

/// Le faux serveur ne reçoit jamais l'IBAN en clair, seulement son repère
#[tokio::test]
async fn server_never_receives_the_iban_in_clear() {
    let fixture = ModelsFixture::start(TestProvider::new("test-provider")).await;
    let connection = connection_with(&fixture, &MaskingSettings::remote_default());
    fixture.server.queue(openai_stream(&["ok"], None));
    let start = chat_start_with(&connection, &format!("Mon IBAN {IBAN}, vérifie."), vec![]);
    run_chat(&fixture, start).await;
    let bodies: Vec<String> = fixture
        .server
        .requests()
        .into_iter()
        .map(|r| r.body)
        .collect();
    assert_eq!(bodies.len(), 1);
    assert!(!bodies[0].contains(IBAN));
    assert!(bodies[0].contains("Mon IBAN [IBAN_1], vérifie."));
}

/// Un repère renvoyé par le modèle, même coupé entre morceaux, revient restitué ; un repère inventé reste tel quel
#[tokio::test]
async fn answer_echoing_the_placeholder_comes_back_restored() {
    let fixture = ModelsFixture::start(TestProvider::new("test-provider")).await;
    let connection = connection_with(&fixture, &MaskingSettings::remote_default());
    let chunks = [
        "Votre IBAN ",
        "[IBA",
        "N_1] est ",
        "bon, pas [IBAN_9] ni [",
        "fin",
    ];
    fixture.server.queue(openai_stream(&chunks, None));
    let start = chat_start_with(&connection, &format!("IBAN {IBAN}"), vec![]);
    let deltas = run_chat(&fixture, start).await;
    let expected = format!("Votre IBAN {IBAN} est bon, pas [IBAN_9] ni [fin");
    assert_eq!(answer_of(&deltas), expected);
    assert!(!answer_of(&deltas).contains("[IBAN_1]"));
}

/// L'annonce du masquage vient une fois, avant le premier texte, et la fin reste unique
#[tokio::test]
async fn masked_delta_precedes_the_first_text() {
    let fixture = ModelsFixture::start(TestProvider::new("test-provider")).await;
    let connection = connection_with(&fixture, &MaskingSettings::remote_default());
    fixture.server.queue(openai_stream(&["a", "b"], None));
    let text = format!("{IBAN} et 4111 1111 1111 1111");
    let deltas = run_chat(&fixture, chat_start_with(&connection, &text, vec![])).await;
    let first_text = deltas
        .iter()
        .position(|d| matches!(d, ChatDelta::Text { .. }));
    let masked = deltas
        .iter()
        .position(|d| matches!(d, ChatDelta::Masked { .. }));
    assert_eq!(masked, Some(0));
    assert_eq!(deltas[0], ChatDelta::Masked { count: 2 });
    assert_eq!(first_text, Some(1));
    let done = deltas
        .iter()
        .filter(|d| matches!(d, ChatDelta::Done { .. }))
        .count();
    assert_eq!(done, 1);
}

/// La ligne du journal compte les valeurs masquées, sans en garder le texte
#[tokio::test]
async fn journal_row_counts_the_masked_values() {
    let fixture = ModelsFixture::start(TestProvider::new("test-provider")).await;
    let connection = connection_with(&fixture, &MaskingSettings::remote_default());
    fixture.server.queue(openai_stream(&["ok"], Some((3, 1))));
    let text = format!("{IBAN} et 4111 1111 1111 1111 et rien d'autre");
    run_chat(&fixture, chat_start_with(&connection, &text, vec![])).await;
    let rows = journal_rows(&fixture);
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].masked, 2);
    let stored = serde_json::to_string(&rows).unwrap();
    assert!(!stored.contains(IBAN) && !stored.contains("4111"));
}

/// Avec le mode coupé, le texte part tel quel et aucune annonce n'est faite
#[tokio::test]
async fn off_mode_sends_the_text_as_is() {
    let fixture = ModelsFixture::start(TestProvider::new("test-provider")).await;
    let off = MaskingSettings {
        mode: MaskingMode::Off,
        ..MaskingSettings::remote_default()
    };
    let connection = connection_with(&fixture, &off);
    fixture.server.queue(openai_stream(&["[IBAN_1]"], None));
    let deltas = run_chat(&fixture, chat_start_with(&connection, IBAN, vec![])).await;
    assert!(fixture.server.requests()[0].body.contains(IBAN));
    assert!(!deltas.iter().any(|d| matches!(d, ChatDelta::Masked { .. })));
    assert_eq!(answer_of(&deltas), "[IBAN_1]");
    assert_eq!(journal_rows(&fixture)[0].masked, 0);
}

/// Le texte d'une partie de contexte est masqué lui aussi ; la consigne système ne l'est pas
#[tokio::test]
async fn context_part_text_is_masked_too() {
    let fixture = ModelsFixture::start(TestProvider::new("test-provider")).await;
    let connection = connection_with(&fixture, &mask_on(&support_masking::ALL));
    allow_kinds(&fixture, &connection, &[ContentKind::MailMessage]);
    fixture.server.queue(openai_stream(&["ok"], None));
    let parts = vec![part(
        ContentKind::MailMessage,
        "Sujet",
        &format!("RIB : {IBAN}"),
    )];
    let mut start = chat_start_with(&connection, "Résume", parts);
    start.request.system = Some(format!("Consigne avec {IBAN}"));
    let deltas = run_chat(&fixture, start).await;
    let body = fixture.server.requests()[0].body.clone();
    assert_eq!(body.matches(IBAN).count(), 1);
    assert!(body.contains("RIB : [IBAN_1]"));
    assert!(body.contains("Consigne avec FR76"));
    assert_eq!(deltas[0], ChatDelta::Masked { count: 1 });
}

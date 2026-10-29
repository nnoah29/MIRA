#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Pont `chat_stream` et masquage : la ligne `masked` précède le texte, la réponse arrive restituée.

mod support;

use mira_assembly::api::AiRequest;
use mira_assembly::mira_vocabulary::{
    AiConnectionId, MaskingMode, MaskingSettings, SensitiveCategory,
};
use mira_bridge::runtime;
use mira_testkit::ai::openai_stream;
use support::chat_fixture::{ask_ai, chat_json, collect_lines, start};

const IBAN: &str = "FR7630006000011234567890189";

/// Avec le masquage actif, la ligne `masked` vient juste après `started`, avant le premier texte, et le texte est restitué.
#[tokio::test(flavor = "multi_thread")]
async fn masked_line_is_emitted_before_text() {
    let f = start().await;
    let masking = MaskingSettings {
        mode: MaskingMode::Mask,
        categories: vec![SensitiveCategory::Iban],
    };
    let connection = AiConnectionId::from_raw(f.connection.clone());
    ask_ai(AiRequest::SetAiMasking {
        connection,
        masking,
    })
    .await;
    f.server.queue(openai_stream(
        &["Votre IBAN [IBAN_1] ", "est noté."],
        Some((7, 3)),
    ));
    let text = format!("Vérifie {IBAN} s'il te plaît");
    let lines = collect_lines(&chat_json(Some(&f.connection), &text)).await;
    let kinds: Vec<&str> = lines.iter().map(|l| l["type"].as_str().unwrap()).collect();
    assert_eq!(kinds[..3], ["started", "masked", "text"], "{lines:?}");
    assert_eq!(kinds.last(), Some(&"done"));
    assert_eq!(lines[1]["count"], 1);
    let answer: String = lines
        .iter()
        .filter(|l| l["type"] == "text")
        .map(|l| l["text"].as_str().unwrap())
        .collect();
    assert_eq!(answer, format!("Votre IBAN {IBAN} est noté."));
    assert!(!f.server.requests()[0].body.contains(IBAN));
    assert!(runtime::close().await.contains("\"ok\":true"));
}

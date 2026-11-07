//! Masquage : la ligne `masked` précède le texte, la réponse arrive restituée.

use mira_assembly::api::AiRequest;
use mira_assembly::mira_vocabulary::{
    AiConnectionId, MaskingMode, MaskingSettings, SensitiveCategory,
};
use mira_testkit::ai::openai_stream;

use super::chat_fixture::chat_json;
use super::chat_start::{ask_ai, collect_lines, start};
use super::driver::Driver;

const IBAN: &str = "FR7630006000011234567890189";

/// Texte de la réponse recomposé à partir des lignes `text`.
fn restored_answer(lines: &[serde_json::Value]) -> String {
    lines
        .iter()
        .filter(|l| l["type"] == "text")
        .map(|l| l["text"].as_str().unwrap())
        .collect()
}

/// Active le masquage des IBAN sur cette connexion.
async fn enable_iban_masking<D: Driver>(driver: &D, connection: &str) {
    let masking = MaskingSettings {
        mode: MaskingMode::Mask,
        categories: vec![SensitiveCategory::Iban],
    };
    let connection = AiConnectionId::from_raw(connection.to_string());
    ask_ai(
        driver,
        AiRequest::SetAiMasking {
            connection,
            masking,
        },
    )
    .await;
}

/// Avec le masquage actif, la ligne `masked` vient juste après `started`, avant le premier texte, et le texte est restitué.
pub async fn masked_line_is_emitted_before_text<D: Driver>() {
    let f = start::<D>().await;
    enable_iban_masking(&f.driver, &f.connection).await;
    f.server.queue(openai_stream(
        &["Votre IBAN [IBAN_1] ", "est noté."],
        Some((7, 3)),
    ));
    let text = format!("Vérifie {IBAN} s'il te plaît");
    let lines = collect_lines(&f.driver, &chat_json(Some(&f.connection), &text)).await;
    let kinds: Vec<&str> = lines.iter().map(|l| l["type"].as_str().unwrap()).collect();
    assert_eq!(kinds[..3], ["started", "masked", "text"], "{lines:?}");
    assert_eq!(kinds.last(), Some(&"done"));
    assert_eq!(lines[1]["count"], 1);
    assert_eq!(
        restored_answer(&lines),
        format!("Votre IBAN {IBAN} est noté.")
    );
    assert!(!f.server.requests()[0].body.contains(IBAN));
    assert!(f.driver.close().await.contains("\"ok\":true"));
}

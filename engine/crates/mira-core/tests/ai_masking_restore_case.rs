#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Restitution insensible à la casse, et attente vidée avant un `Done` d'échec.

mod ai_chat_support;
mod ai_models_support;
mod support_api;
mod support_masking;

use ai_chat_support::{answer_of, chat_start_with, connection_with, run_chat};
use ai_models_support::{ModelsFixture, TestProvider};
use mira_testkit::ai::Behavior;
use mira_vocabulary::{ChatDelta, MaskingSettings, StopReason};
use support_masking::{feed, restorer_for};

const IBAN: &str = "FR7630006000011234567890189";

/// Un repère écrit avec une autre casse revient avec la valeur d'origine
#[test]
fn placeholder_with_other_case_is_restored() {
    let (_, mut restorer) = restorer_for(&format!("{IBAN} 4111111111111111"));
    let out = restorer.push("a [iban_1] b [Carte_1] c [IBAN_1]");
    assert_eq!(out, format!("a {IBAN} b 4111111111111111 c {IBAN}"));
    assert_eq!(restorer.finish(), "");
}

/// La restitution à casse différente donne le même résultat quelle que soit la coupure
#[test]
fn case_insensitive_restore_split_at_every_position() {
    let reply = "é [iban_1] et [CaRtE_1], [Iban_2] [IbAn_ [[carte_1] ✓ [iba";
    let expected =
        format!("é {IBAN} et 4111111111111111, [Iban_2] [IbAn_ [4111111111111111 ✓ [iba");
    for cut in 1..=reply.chars().count() {
        let (_, mut restorer) = restorer_for(&format!("{IBAN} 4111111111111111"));
        assert_eq!(feed(&mut restorer, reply, cut), expected, "taille {cut}");
    }
}

/// Un repère d'une catégorie qui n'a pas été masquée passe tel quel, quelle que soit sa casse
#[test]
fn unknown_category_in_other_case_passes_through() {
    let (_, mut restorer) = restorer_for(IBAN);
    let reply = "[secret_1] [Secret_1] [email_1] [iban_2]";
    assert_eq!(feed(&mut restorer, reply, 2), reply);
}

/// Un échec en cours de flux vide d'abord l'attente de la restitution, puis envoie l'unique `Done`
#[tokio::test]
async fn pending_text_is_flushed_before_a_failed_done() {
    let fixture = ModelsFixture::start(TestProvider::new("test-provider")).await;
    let connection = connection_with(&fixture, &MaskingSettings::remote_default());
    let event = |text: &str| {
        let line = format!(r#"data: {{"choices":[{{"delta":{{"content":"{text}"}}}}]}}"#);
        format!("{line}\n\n").into_bytes()
    };
    let chunks = vec![event("voici "), event("[IBAN_")];
    fixture.server.queue(Behavior::DropAfter { chunks });
    let start = chat_start_with(&connection, &format!("IBAN {IBAN}"), vec![]);
    let deltas = run_chat(&fixture, start).await;
    let text = answer_of(&deltas);
    assert_eq!(text, "voici [IBAN_");
    let done = deltas
        .iter()
        .filter(|d| matches!(d, ChatDelta::Done { .. }))
        .count();
    assert_eq!(done, 1);
    assert!(matches!(
        deltas.last(),
        Some(ChatDelta::Done {
            reason: StopReason::Failed { .. }
        })
    ));
}

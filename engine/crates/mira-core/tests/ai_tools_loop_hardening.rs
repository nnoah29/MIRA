#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Boucle d'outils, durcissement : contenu hostile borné par la taille d'un résultat, narration séparée du texte suivant.

mod ai_loop_support;
mod ai_models_support;
mod support;
mod support_api;
mod support_mirror;
mod support_read;

use ai_loop_support::fixture::LoopFixture;
use ai_loop_support::turns::{FORMATS, collect, text_turn, tool_results, tool_turn};
use mira_core::ai::tools::render::mail_line;
use mira_vocabulary::{AddressEntry, ChatDelta, Timestamp};
use support_mirror::fixtures::{appearance, sample_message};

/// Ajoute à la boîte un message dont le sujet est énorme.
fn add_hostile_message(fixture: &LoopFixture) {
    let mut message = sample_message("m9", "acc-1");
    message.subject = "S".repeat(50_000);
    fixture
        .db
        .write(|tx| {
            tx.insert_message(&message, Timestamp::from_millis(9_000))?;
            tx.add_appearance(&appearance("m9", "box-1", 9))
        })
        .unwrap();
}

/// Un sujet de 50 000 caractères ne peut pas faire dépasser les 8 000 caractères d'un résultat
#[tokio::test]
async fn hostile_subject_cannot_blow_the_result_budget() {
    for format in FORMATS {
        let fixture = LoopFixture::start(format).await;
        add_hostile_message(&fixture);
        fixture.queue(tool_turn(
            format,
            "",
            &[("c1", "list_messages", "{}")],
            None,
        ));
        fixture.queue(text_turn(format, "Fini.", None));
        collect(&mut fixture.ask("Liste").await).await;
        let result = &tool_results(&fixture.bodies()[1], format)[0];
        assert!(
            result.chars().count() <= 8_100,
            "{format:?} {}",
            result.chars().count()
        );
    }
}

/// La ligne d'un message coupe le sujet et la liste des expéditeurs
#[test]
fn mail_line_cuts_a_huge_subject_and_sender_list() {
    let mut message = sample_message("m1", "acc-1");
    message.subject = "S".repeat(50_000);
    message.from = (0..500)
        .map(|n| AddressEntry {
            name: Some("N".repeat(300)),
            address: format!("a{n}@example.org"),
        })
        .collect();
    assert!(mail_line(&message).chars().count() <= 2_500);
}

/// Le texte de narration d'un tour d'outils est séparé du texte du tour suivant
#[tokio::test]
async fn narration_is_separated_from_the_final_text() {
    for format in FORMATS {
        let fixture = LoopFixture::start(format).await;
        fixture.queue(tool_turn(
            format,
            "Je cherche.",
            &[("c1", "list_mailboxes", "{}")],
            None,
        ));
        fixture.queue(text_turn(format, "Voici.", None));
        let deltas = collect(&mut fixture.ask("Liste").await).await;
        let text: String = deltas
            .iter()
            .filter_map(|d| match d {
                ChatDelta::Text { text } => Some(text.as_str()),
                _ => None,
            })
            .collect();
        assert_eq!(text, "Je cherche.\n\nVoici.", "{format:?}");
        let history = fixture.bodies()[1].to_string();
        assert!(history.contains("Je cherche.") && !history.contains("Je cherche.\\n\\nVoici"));
    }
}

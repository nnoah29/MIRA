#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Outil `show_mails` : les cartes arrivent entre l'annonce et la fin de l'appel, rien n'est lu ni modifié.

mod ai_loop_support;
mod ai_models_support;
mod support;
mod support_api;
mod support_mirror;
mod support_read;

use ai_loop_support::fixture::LoopFixture;
use ai_loop_support::turns::{FORMATS, collect, shape, text_turn, tool_results, tool_turn};
use mira_vocabulary::ChatDelta;

const TWO: &str = r#"{"title":"Mails importants","mails":[{"id":"m1","reason":"échéance vendredi"},{"id":"m2"}]}"#;

/// Les cartes arrivent entre l'annonce et la fin de l'appel ; le modèle apprend que l'utilisateur les voit
#[tokio::test]
async fn cards_arrive_between_called_and_finished() {
    for format in FORMATS {
        let fixture = LoopFixture::start(format).await;
        fixture.queue(tool_turn(format, "", &[("c1", "show_mails", TWO)], None));
        fixture.queue(text_turn(format, "Voici les plus importants.", None));
        let mut stream = fixture.ask("mes mails importants ?").await;
        let deltas = collect(&mut stream).await;
        let expected = [
            "called:show_mails",
            "cards",
            "finished:ok",
            "text",
            "done:end_turn",
        ];
        assert_eq!(shape(&deltas), expected, "{format:?}");
        let ChatDelta::MailCardsShown { id, cards } = &deltas[1] else {
            panic!("cartes attendues");
        };
        assert_eq!(id, "c1");
        assert_eq!(cards.title.as_deref(), Some("Mails importants"));
        assert_eq!(cards.cards.len(), 2);
        assert_eq!(cards.cards[0].id, "m1");
        assert_eq!(cards.cards[0].reason.as_deref(), Some("échéance vendredi"));
        assert_eq!(cards.cards[1].reason, None);
        let results = tool_results(&fixture.bodies()[1], format);
        assert!(results[0].contains("cartes"), "{results:?}");
    }
}

/// Une liste vide, un identifiant vide, trop de cartes ou une raison sur plusieurs lignes : erreur, aucune carte
#[tokio::test]
async fn invalid_cards_are_refused_without_any_card() {
    let format = FORMATS[1];
    let too_many: Vec<String> = (0..51).map(|n| format!(r#"{{"id":"m{n}"}}"#)).collect();
    let bad = [
        r#"{"mails":[]}"#.to_string(),
        r#"{"mails":[{"id":"  "}]}"#.to_string(),
        format!(r#"{{"mails":[{}]}}"#, too_many.join(",")),
        r#"{"mails":[{"id":"m1","reason":"a\nb"}]}"#.to_string(),
    ];
    for arguments in bad {
        let fixture = LoopFixture::start(format).await;
        fixture.queue(tool_turn(
            format,
            "",
            &[("c1", "show_mails", &arguments)],
            None,
        ));
        fixture.queue(text_turn(format, "Désolé.", None));
        let mut stream = fixture.ask("mes mails importants ?").await;
        let deltas = collect(&mut stream).await;
        let expected = [
            "called:show_mails",
            "finished:error",
            "text",
            "done:end_turn",
        ];
        assert_eq!(shape(&deltas), expected, "{arguments}");
    }
}

/// Un même identifiant cité deux fois ne donne qu'une carte, la première raison est gardée
#[tokio::test]
async fn a_repeated_id_makes_a_single_card() {
    let format = FORMATS[0];
    let fixture = LoopFixture::start(format).await;
    let arguments = r#"{"mails":[{"id":"m1","reason":"premier"},{"id":"m1","reason":"second"}]}"#;
    fixture.queue(tool_turn(
        format,
        "",
        &[("c1", "show_mails", arguments)],
        None,
    ));
    fixture.queue(text_turn(format, "Voilà.", None));
    let mut stream = fixture.ask("montre").await;
    let deltas = collect(&mut stream).await;
    let ChatDelta::MailCardsShown { cards, .. } = &deltas[1] else {
        panic!("cartes attendues");
    };
    assert_eq!(cards.cards.len(), 1);
    assert_eq!(cards.cards[0].reason.as_deref(), Some("premier"));
}

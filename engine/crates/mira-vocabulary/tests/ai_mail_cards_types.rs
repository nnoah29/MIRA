#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Cartes de messages que l'IA montre dans le chat : carte, lot et delta qui le porte.

use mira_vocabulary::{ChatDelta, MailCard, MailCards};
use serde_json::json;

/// Un lot de cartes survit à un aller-retour JSON, la raison est facultative
#[test]
fn mail_cards_round_trip() {
    let cards = MailCards {
        title: Some("Mails importants".into()),
        cards: vec![
            MailCard {
                id: "m1".into(),
                reason: Some("échéance vendredi".into()),
            },
            MailCard {
                id: "m2".into(),
                reason: None,
            },
        ],
    };
    let value = serde_json::to_value(&cards).unwrap();
    let back: MailCards = serde_json::from_value(value).unwrap();
    assert_eq!(back, cards);
}

/// Le delta s'écrit avec le type `mail_cards_shown`, l'identifiant de l'appel et les cartes
#[test]
fn delta_uses_the_mail_cards_shown_type() {
    let delta = ChatDelta::MailCardsShown {
        id: "c1".into(),
        cards: MailCards {
            title: None,
            cards: vec![MailCard {
                id: "m1".into(),
                reason: None,
            }],
        },
    };
    let value = serde_json::to_value(&delta).unwrap();
    assert_eq!(value["type"], json!("mail_cards_shown"));
    assert_eq!(value["id"], json!("c1"));
    assert_eq!(value["cards"]["cards"][0]["id"], json!("m1"));
    let back: ChatDelta = serde_json::from_value(value).unwrap();
    assert_eq!(back, delta);
}

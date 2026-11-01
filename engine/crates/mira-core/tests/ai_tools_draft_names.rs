#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Noms affichés d'une proposition de brouillon : jamais de quoi cacher ou imiter une autre adresse.

mod ai_tools_draft_support;
mod ai_tools_support;
mod support;
mod support_api;
mod support_mirror;
mod support_read;

use ai_tools_draft_support::compose;
use mira_vocabulary::ToolState;
use serde_json::json;

/// Un nom qui contient une adresse, des chevrons, un arobase, un caractère de contrôle ou d'inversion de sens est refusé
#[test]
fn display_names_cannot_impersonate_another_address() {
    let bad = [
        "Alice <ceo@bank.com>".to_string(),
        "ceo@bank.com".to_string(),
        "A\u{202E}B".to_string(),
        "A\u{2066}B".to_string(),
        "ligne\nsuivante".to_string(),
        "A\u{0007}B".to_string(),
        "x".repeat(81),
    ];
    for name in bad {
        let run = compose(json!({"to": [format!("{name} <evil@x.com>")], "body": "x"}));
        assert_eq!(run.state, ToolState::Error, "{name:?}");
        assert!(run.proposal.is_none(), "{name:?}");
    }
}

/// Les noms ordinaires, accents, apostrophes et guillemets compris, passent
#[test]
fn ordinary_names_are_kept() {
    let run = compose(json!({
        "to": ["Marie-Hélène O'Brien <mh@ex.org>", "\"Dupont, Jean\" <j@d.fr>", "Support Banque <s@b.fr>"],
        "body": "x"
    }));
    assert_eq!(run.state, ToolState::Ok, "{}", run.text);
    let to = run.proposal.unwrap().to.unwrap();
    assert_eq!(to[0].name.as_deref(), Some("Marie-Hélène O'Brien"));
    assert_eq!(to[1].name.as_deref(), Some("Dupont, Jean"));
    assert_eq!(to[2].address, "s@b.fr");
}

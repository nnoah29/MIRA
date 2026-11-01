#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Outil `compose_draft` : proposition d'un nouveau message, validation des arguments, trous entre crochets.

mod ai_tools_draft_support;
mod ai_tools_support;
mod support;
mod support_api;
mod support_mirror;
mod support_read;

use ai_tools_draft_support::compose;
use mira_vocabulary::{DraftProposalKind, ToolState};
use serde_json::json;

/// Un appel complet donne une proposition « nouveau message » aux adresses normalisées
#[test]
fn valid_compose_gives_a_new_proposal() {
    let run = compose(json!({
        "to": ["Ma boss <Boss@Example.ORG>", "c@d.org"],
        "cc": ["x@y.fr"],
        "subject": "Fin de stage",
        "body": "Bonjour,\n\nMon stage se termine mercredi.",
        "reply_to": "m-1",
        "identity": "id-1"
    }));
    assert_eq!(run.state, ToolState::Ok);
    let proposal = run.proposal.unwrap();
    assert_eq!(proposal.kind, DraftProposalKind::New);
    let to = proposal.to.unwrap();
    assert_eq!(to[0].address, "Boss@example.org");
    assert_eq!(to[0].name.as_deref(), Some("Ma boss"));
    assert_eq!(to[1].address, "c@d.org");
    assert_eq!(proposal.cc.unwrap()[0].address, "x@y.fr");
    assert_eq!(proposal.subject.as_deref(), Some("Fin de stage"));
    assert_eq!(proposal.reply_to.as_deref(), Some("m-1"));
    assert_eq!(proposal.identity.as_deref(), Some("id-1"));
    assert!(proposal.selection.is_none());
    assert!(run.text.contains("relira"));
}

/// Une adresse invalide est refusée, nommée au modèle, et rien n'est proposé
#[test]
fn bad_address_is_refused_and_named() {
    let run = compose(json!({"to": ["pas-une-adresse"], "body": "x"}));
    assert_eq!(run.state, ToolState::Error);
    assert!(run.proposal.is_none());
    assert!(run.text.contains("pas-une-adresse"));
}

/// Le corps est obligatoire et limité à 50 000 caractères
#[test]
fn body_is_required_and_bounded() {
    assert_eq!(compose(json!({"subject": "x"})).state, ToolState::Error);
    assert_eq!(compose(json!({"body": "  "})).state, ToolState::Error);
    let at_limit = compose(json!({"body": "é".repeat(50_000)}));
    assert_eq!(at_limit.state, ToolState::Ok);
    let over = compose(json!({"body": "é".repeat(50_001)}));
    assert_eq!(over.state, ToolState::Error);
    assert!(over.proposal.is_none());
}

/// L'objet tient sur une ligne et fait 300 caractères au plus
#[test]
fn subject_is_one_short_line() {
    let ok = compose(json!({"subject": "a".repeat(300), "body": "x"}));
    assert_eq!(ok.state, ToolState::Ok);
    for subject in ["a".repeat(301), "ligne 1\nligne 2".to_string()] {
        let run = compose(json!({"subject": subject, "body": "x"}));
        assert_eq!(run.state, ToolState::Error);
        assert!(run.proposal.is_none());
    }
}

/// Vingt destinataires au plus par liste
#[test]
fn at_most_twenty_recipients_per_list() {
    let many = |n: usize| -> Vec<String> { (0..n).map(|i| format!("u{i}@ex.org")).collect() };
    assert_eq!(
        compose(json!({"to": many(20), "body": "x"})).state,
        ToolState::Ok
    );
    let run = compose(json!({"to": many(21), "body": "x"}));
    assert_eq!(run.state, ToolState::Error);
    assert_eq!(
        compose(json!({"cc": many(21), "body": "x"})).state,
        ToolState::Error
    );
}

/// Des passages entre crochets sont signalés au modèle, la proposition passe quand même
#[test]
fn brackets_produce_a_warning_but_still_propose() {
    let run = compose(json!({"body": "Bonjour [Prénom/Nom], voir [date]."}));
    assert_eq!(run.state, ToolState::Ok);
    assert!(run.proposal.is_some());
    assert!(run.text.contains("crochets"));
    assert!(run.text.contains("[Prénom/Nom]"));
    let clean = compose(json!({"body": "Bonjour Marie."}));
    assert!(!clean.text.contains("crochets"));
}

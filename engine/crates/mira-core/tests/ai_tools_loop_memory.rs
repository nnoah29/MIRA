#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Outils de mémoire dans la boucle du chat : propositions sans écriture, rappel par le sens, discrétion si désactivée.

mod ai_loop_support;
mod ai_models_support;
mod support;
mod support_ai_memory_loop;
mod support_api;
mod support_mirror;
mod support_read;

use ai_loop_support::fixture::LoopFixture;
use ai_loop_support::turns::{FORMATS, shape, text_turn, tool_results, tool_turn};
use mira_testkit::ai::Behavior;
use mira_vocabulary::{ChatDelta, MemoryProposalAction};
use support_ai_memory_loop::{enable_with_one_memory, one_vector, run_turns};

const REMEMBER_ARGS: &str = r#"{"text":"Je signe « Noah »","kind":"preference"}"#;

/// Retenir : la proposition arrive entre l'annonce et la fin, et rien n'est écrit
#[tokio::test]
async fn remember_proposal_arrives_in_order_and_writes_nothing() {
    for format in FORMATS {
        let f = LoopFixture::start(format).await;
        enable_with_one_memory(&f);
        let turns = [
            one_vector(),
            tool_turn(format, "", &[("c1", "remember", REMEMBER_ARGS)], None),
            text_turn(format, "Je te propose de le retenir.", None),
        ];
        let deltas = run_turns(&f, turns, "retiens ma signature").await;
        let expected = [
            "called:remember",
            "memory",
            "finished:ok",
            "text",
            "done:end_turn",
        ];
        assert_eq!(shape(&deltas), expected, "{format:?}");
        let ChatDelta::MemoryProposed { id, proposal } = &deltas[1] else {
            panic!("proposition attendue")
        };
        assert_eq!(
            (id.as_str(), proposal.action),
            ("c1", MemoryProposalAction::Remember)
        );
        assert_eq!(
            f.db.read(|tx| tx.list_ai_memories(10, None)).unwrap().len(),
            1
        );
    }
}

/// Rappeler : le texte et l'identifiant du souvenir proche reviennent au modèle
#[tokio::test]
async fn recall_returns_the_stored_text_and_its_id() {
    let format = FORMATS[1];
    let f = LoopFixture::start(format).await;
    enable_with_one_memory(&f);
    let call = [("c1", "recall_memory", r#"{"query":"mon chat"}"#)];
    let turns = [
        one_vector(),
        tool_turn(format, "", &call, None),
        one_vector(),
        text_turn(format, "Il s'appelle Mochi.", None),
    ];
    let deltas = run_turns(&f, turns, "comment s'appelle mon chat ?").await;
    assert_eq!(
        shape(&deltas),
        [
            "called:recall_memory",
            "finished:ok",
            "text",
            "done:end_turn"
        ]
    );
    let results = tool_results(f.bodies().last().unwrap(), format);
    assert!(
        results[0].contains("mem-1") && results[0].contains("Mochi"),
        "{results:?}"
    );
}

/// Oublier : la proposition porte l'identifiant et rien n'est supprimé
#[tokio::test]
async fn forget_proposal_carries_the_id_and_deletes_nothing() {
    let format = FORMATS[0];
    let f = LoopFixture::start(format).await;
    enable_with_one_memory(&f);
    let args = r#"{"id":"mem-1","reason":"il est parti"}"#;
    let turns = [
        one_vector(),
        tool_turn(format, "", &[("c1", "forget_memory", args)], None),
        text_turn(format, "D'accord.", None),
    ];
    let deltas = run_turns(&f, turns, "oublie le nom de mon chat").await;
    let ChatDelta::MemoryProposed { proposal, .. } = &deltas[1] else {
        panic!("proposition attendue")
    };
    assert_eq!(
        (proposal.action, proposal.id.as_deref()),
        (MemoryProposalAction::Forget, Some("mem-1"))
    );
    assert_eq!(
        f.db.read(|tx| tx.list_ai_memories(10, None)).unwrap().len(),
        1
    );
}

/// Mémoire désactivée : aucun outil de mémoire n'est annoncé au modèle
#[tokio::test]
async fn memory_tools_are_not_announced_when_disabled() {
    let format = FORMATS[1];
    let f = LoopFixture::start(format).await;
    run_turns(&f, [text_turn(format, "Bonjour.", None)], "bonjour").await;
    let body = f.bodies()[0].to_string();
    for name in ["recall_memory", "remember", "forget_memory"] {
        assert!(!body.contains(name), "{name}");
    }
}

/// Un serveur d'embeddings en panne donne une erreur d'outil sobre et la discussion continue
#[tokio::test]
async fn recall_fails_softly_when_the_embedder_is_down() {
    let format = FORMATS[1];
    let f = LoopFixture::start(format).await;
    enable_with_one_memory(&f);
    let down = Behavior::Status {
        code: 500,
        headers: vec![],
        body: "sk-secret".into(),
    };
    let call = [("c1", "recall_memory", r#"{"query":"chat"}"#)];
    let turns = [
        one_vector(),
        tool_turn(format, "", &call, None),
        down,
        text_turn(format, "Je ne retrouve pas.", None),
    ];
    let deltas = run_turns(&f, turns, "mon chat ?").await;
    assert_eq!(
        shape(&deltas),
        [
            "called:recall_memory",
            "finished:error",
            "text",
            "done:end_turn"
        ]
    );
    let results = tool_results(f.bodies().last().unwrap(), format);
    assert!(!results[0].contains("sk-secret"), "{results:?}");
}

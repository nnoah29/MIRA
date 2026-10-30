#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Troncature de l'entrée : plus anciens messages d'abord, dernier message coupé sur un caractère entier.

use mira_core::ai::chat::truncate_request;
use mira_vocabulary::{ChatMessage, ChatRequest, ChatRole};

/// Requête de ces messages alternés, avec une consigne facultative
fn request(system: Option<&str>, texts: &[&str]) -> ChatRequest {
    let messages = texts.iter().enumerate().map(|(index, text)| {
        ChatMessage::plain(
            if index % 2 == 0 {
                ChatRole::User
            } else {
                ChatRole::Assistant
            },
            (*text).to_string(),
        )
    });
    ChatRequest {
        model: Some("m".to_string()),
        system: system.map(str::to_string),
        messages: messages.collect(),
        max_output_tokens: Some(100),
        temperature: None,
        tools: Vec::new(),
        context: Vec::new(),
    }
}

/// Les plus anciens messages partent d'abord ; consigne et derniers messages restent
#[test]
fn truncate_drops_oldest_messages_first() {
    let original = request(Some("ss"), &["aaaa", "bbbb", "cccc"]);
    let truncated = truncate_request(original.clone(), 10);
    assert_eq!(truncated.system.as_deref(), Some("ss"));
    let texts: Vec<&str> = truncated.messages.iter().map(|m| m.text.as_str()).collect();
    assert_eq!(texts, vec!["bbbb", "cccc"]);
    assert_eq!(truncated.messages[0].role, original.messages[1].role);
    assert_eq!(truncated.model, original.model);
}

/// Un dernier message trop long est coupé sur une frontière de caractère, marqué, et tient dans le plafond
#[test]
fn truncate_cuts_on_char_boundary() {
    let long = "é😀à".repeat(100);
    let original = request(Some("abc"), &["vieux", &long]);
    let truncated = truncate_request(original, 50);
    assert_eq!(truncated.messages.len(), 1);
    let text = &truncated.messages[0].text;
    assert!(text.ends_with("…[tronqué]"));
    let kept = text.trim_end_matches("…[tronqué]");
    assert!(long.starts_with(kept) && !kept.is_empty());
    assert!(3 + text.chars().count() <= 50);
    for budget in 0..20 {
        let cut = truncate_request(request(None, &[&long]), budget);
        assert!(cut.messages[0].text.chars().count() <= budget.max("…[tronqué]".chars().count()));
    }
}

/// Une requête qui tient dans le plafond est rendue telle quelle
#[test]
fn truncate_leaves_short_requests_untouched() {
    let original = request(Some("consigne"), &["un", "deux", "trois"]);
    assert_eq!(truncate_request(original.clone(), 120_000), original);
    assert_eq!(truncate_request(original.clone(), 19), original);
}

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, dead_code)]
//! Identité et derniers messages de test pour les règles pures des libellés système.

use mira_core::system_labels::classify::LastMessage;
use mira_core::system_labels::me::Me;

pub const NOW: i64 = 1_800_000_000_000;

/// Identité de test : deux adresses, un nom complet et son prénom
pub fn me() -> Me {
    Me {
        addresses: vec!["noah@mira.dev".into(), "alias@mira.dev".into()],
        names: vec!["noah toffa".into(), "noah".into()],
    }
}

/// Dernier message de moi, envoyé il y a `age` ms à un humain
pub fn mine(age: i64) -> LastMessage {
    LastMessage {
        from_own: true,
        sender_automated: false,
        awaits_human: true,
        date: NOW - age,
        text: String::new(),
    }
}

/// Dernier message reçu il y a `age` ms, de texte donné
pub fn received(age: i64, text: &str) -> LastMessage {
    LastMessage {
        from_own: false,
        sender_automated: false,
        awaits_human: true,
        date: NOW - age,
        text: text.to_string(),
    }
}

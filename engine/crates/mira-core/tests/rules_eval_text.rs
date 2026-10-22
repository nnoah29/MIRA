#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, dead_code)]
//! Comparaison de texte : casse Unicode, regex, listes et valeurs absentes.

mod support_rules;

#[path = "rules_eval_text/absent.rs"]
mod absent;
#[path = "rules_eval_text/operators.rs"]
mod operators;

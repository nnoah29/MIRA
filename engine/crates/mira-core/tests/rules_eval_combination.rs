#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, dead_code)]
//! Combinaison des conditions, portée, règles désactivées et arrêt du parcours.

mod support_rules;

#[path = "rules_eval_combination/combinations.rs"]
mod combinations;
#[path = "rules_eval_combination/ordering.rs"]
mod ordering;
#[path = "rules_eval_combination/scope.rs"]
mod scope;

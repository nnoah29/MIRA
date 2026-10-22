#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, dead_code)]
//! Évaluation des conditions élémentaires, une variante de `ConditionTest` à la fois.

mod support_rules;

#[path = "rules_eval_conditions/addresses.rs"]
mod addresses;
#[path = "rules_eval_conditions/content.rs"]
mod content;
#[path = "rules_eval_conditions/counts_and_dates.rs"]
mod counts_and_dates;
#[path = "rules_eval_conditions/flags.rs"]
mod flags;
#[path = "rules_eval_conditions/membership.rs"]
mod membership;

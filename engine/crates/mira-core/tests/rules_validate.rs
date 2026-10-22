#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, dead_code)]
//! Validation des règles à l'enregistrement et statut calculé à la lecture.

mod support_rules;

#[path = "rules_validate/condition_targets.rs"]
mod condition_targets;
#[path = "rules_validate/conflicts.rs"]
mod conflicts;
#[path = "rules_validate/limits.rs"]
mod limits;
#[path = "rules_validate/mailboxes.rs"]
mod mailboxes;
#[path = "rules_validate/regexes.rs"]
mod regexes;
#[path = "rules_validate/scope.rs"]
mod scope;
#[path = "rules_validate/shape.rs"]
mod shape;
#[path = "rules_validate/status.rs"]
mod status;
#[path = "rules_validate/structure.rs"]
mod structure;
#[path = "rules_validate/values.rs"]
mod values;
#[path = "rules_validate/world.rs"]
mod world;

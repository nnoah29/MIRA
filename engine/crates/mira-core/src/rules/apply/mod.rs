//! Exécution des règles sur un message arrivé : chargement, évaluation, soumission des actions, reprise.

mod context_loader;
mod eligibility;
mod replay_pending;
mod submit_actions;

pub use eligibility::rules_apply_in;
pub use replay_pending::replay_pending_rules;
pub use submit_actions::apply_rules_with;

//! Exécution des règles sur un message arrivé : chargement, évaluation, soumission des actions, reprise.

mod context_loader;
mod eligibility;
mod label_changes;
mod replay_pending;
mod reply_clock;
mod reply_drafts;
mod reply_eligibility;
mod submit_actions;

pub use eligibility::rules_apply_in;
pub(crate) use label_changes::apply_label_changes;
pub use replay_pending::replay_pending_rules;
pub use submit_actions::apply_rules_with;

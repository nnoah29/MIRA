//! Validation d'une règle à l'enregistrement et calcul de son statut à la lecture.

mod conflicts;
mod limits;
mod mailboxes;
mod matchers;
mod regexes;
mod status;
mod structure;
mod world;

use mira_vocabulary::{MiraError, Rule};

pub use limits::{
    MAX_ACTIONS, MAX_ALTERNATIVES, MAX_CONDITIONS_PER_GROUP, MAX_NAME_LEN, MAX_REGEX_LEN,
    MAX_VALUE_LEN, MAX_VALUES_PER_MATCHER,
};
pub(crate) use mailboxes::{depends_on_account_mailbox, named_mailbox};
pub use status::{RuleStatus, rule_status};
pub use world::RuleWorld;

/// Refuse une règle incorrecte avec une erreur explicite, sans rien enregistrer.
pub fn validate_rule(rule: &Rule, world: &dyn RuleWorld) -> Result<(), MiraError> {
    structure::check_structure(rule)?;
    validate_shape(rule)?;
    mailboxes::check_world(rule, world)?;
    conflicts::check_conflicts(rule)
}

/// Contrôles de forme seuls (plafonds, regex), sans le monde ni l'exigence d'une action : sert aussi à l'aperçu.
pub fn validate_shape(rule: &Rule) -> Result<(), MiraError> {
    limits::check_limits(rule)?;
    regexes::check_regexes(rule)
}

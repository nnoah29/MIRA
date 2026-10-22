//! Lecture, en une seule passe sur la base, des règles valides et du contexte d'un message.

use mira_vocabulary::{MessageId, MiraError, Rule, Timestamp};

use crate::rule_context::{StoredRuleWorld, message_context, own_addresses};
use crate::rules::context::MessageContext;
use crate::rules::validate::{RuleStatus, rule_status};
use crate::storage::Database;

/// Règles à évaluer et contexte du message.
pub(super) struct Loaded {
    pub rules: Vec<Rule>,
    pub context: MessageContext,
}

/// Charge les règles valides (ordre de position) et le contexte du message, ou rien si le message n'existe plus
pub(super) fn load_for_message(
    db: &Database,
    message: &MessageId,
    now: Timestamp,
) -> Result<Option<Loaded>, MiraError> {
    db.read(|tx| {
        let Some(summary) = tx.get_message(message)? else {
            return Ok(None);
        };
        let world = StoredRuleWorld::load(tx)?;
        let rules = tx
            .list_rules()?
            .into_iter()
            .filter(|rule| rule_status(rule, &world) == RuleStatus::Valid)
            .collect();
        let own = own_addresses(tx, &summary.account)?;
        let context = message_context(tx, summary, own, now)?;
        Ok(Some(Loaded { rules, context }))
    })
}

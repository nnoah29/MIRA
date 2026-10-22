//! Interrogation du monde (comptes, boîtes) par la validation, sans accès au stockage.

use mira_vocabulary::{AccountId, MailboxId};

/// Ce que la validation doit savoir des comptes et des boîtes existants.
pub trait RuleWorld {
    /// Vrai si le compte existe encore.
    fn account_exists(&self, account: &AccountId) -> bool;

    /// Compte propriétaire de la boîte, ou rien si la boîte n'existe plus.
    fn mailbox_account(&self, mailbox: &MailboxId) -> Option<AccountId>;
}

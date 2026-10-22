//! Monde des règles photographié depuis le stockage : comptes, propriétaire de chaque boîte et libellés locaux.

use std::collections::{HashMap, HashSet};

use mira_vocabulary::{AccountId, LocalLabelId, MailboxId, MiraError};

use crate::rules::validate::RuleWorld;
use crate::storage::Tx;

/// Comptes, boîtes et libellés locaux du profil au moment de la lecture.
#[derive(Debug, Clone, Default)]
pub struct StoredRuleWorld {
    accounts: HashSet<AccountId>,
    mailboxes: HashMap<MailboxId, AccountId>,
    labels: HashSet<LocalLabelId>,
}

impl StoredRuleWorld {
    /// Lit tous les comptes du profil, les boîtes de chacun et les libellés locaux
    pub fn load(tx: &Tx<'_>) -> Result<Self, MiraError> {
        let mut world = Self::default();
        for account in tx.list_accounts()? {
            for mailbox in tx.list_mailboxes(&account.id)? {
                world.mailboxes.insert(mailbox.id, account.id.clone());
            }
            world.accounts.insert(account.id);
        }
        for label in tx.list_local_labels()? {
            world.labels.insert(label.id);
        }
        Ok(world)
    }
}

impl RuleWorld for StoredRuleWorld {
    /// Vrai si le compte existait au moment de la lecture
    fn account_exists(&self, account: &AccountId) -> bool {
        self.accounts.contains(account)
    }

    /// Compte propriétaire de la boîte au moment de la lecture
    fn mailbox_account(&self, mailbox: &MailboxId) -> Option<AccountId> {
        self.mailboxes.get(mailbox).cloned()
    }

    /// Vrai si le libellé local existait au moment de la lecture
    fn local_label_exists(&self, label: &LocalLabelId) -> bool {
        self.labels.contains(label)
    }
}

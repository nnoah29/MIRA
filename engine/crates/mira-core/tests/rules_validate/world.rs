//! Monde de test : comptes et boîtes connus, et règle valide de référence.

#[allow(unused_imports)]
pub use crate::support_rules::valid_rule::{valid_rule, with_action};
use mira_core::rules::validate::RuleWorld;
use mira_vocabulary::{AccountId, LocalLabelId, MailboxId, TemplateId};

/// Comptes et boîtes (boîte, compte) existants
pub struct FakeWorld {
    pub accounts: Vec<&'static str>,
    pub mailboxes: Vec<(&'static str, &'static str)>,
    pub labels: Vec<&'static str>,
    pub templates: Vec<&'static str>,
}

impl RuleWorld for FakeWorld {
    /// Vrai si le compte figure dans la liste
    fn account_exists(&self, account: &AccountId) -> bool {
        self.accounts.contains(&account.as_str())
    }

    /// Compte de la boîte, s'il la connaît
    fn mailbox_account(&self, mailbox: &MailboxId) -> Option<AccountId> {
        self.mailboxes
            .iter()
            .find(|(id, _)| *id == mailbox.as_str())
            .map(|(_, account)| AccountId::from_raw(*account))
    }

    /// Vrai si le libellé local figure dans la liste
    fn local_label_exists(&self, label: &LocalLabelId) -> bool {
        self.labels.contains(&label.as_str())
    }

    /// Vrai si le modèle figure dans la liste
    fn template_exists(&self, template: &TemplateId) -> bool {
        self.templates.contains(&template.as_str())
    }
}

/// Monde de référence : acc-1 et acc-2, boîte mb-1 à acc-1, boîte mb-2 à acc-2
pub fn world() -> FakeWorld {
    FakeWorld {
        accounts: vec!["acc-1", "acc-2"],
        mailboxes: vec![("mb-1", "acc-1"), ("mb-2", "acc-2")],
        labels: vec!["lab-1"],
        templates: vec!["tpl-1"],
    }
}

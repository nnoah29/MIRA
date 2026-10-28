#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, dead_code)]
//! Comptes et boîtes factices : compte `a1` (`moi@x.fr`), boîte `m1` (`Archives/Factures`).

use mira_core::portability::rules::{AccountRefs, MailboxRefs};
use mira_vocabulary::{AccountId, MailboxId};

/// Table de références factice ; `without_label` fait disparaître le libellé `l1`
#[derive(Debug, Default)]
pub struct FakeRefs {
    pub without_label: bool,
}

impl AccountRefs for FakeRefs {
    /// Adresse du compte `a1`
    fn account_address(&self, id: &AccountId) -> Option<String> {
        (id.as_str() == "a1").then(|| "moi@x.fr".to_string())
    }
    /// Compte `a1` pour son adresse
    fn account_by_address(&self, address: &str) -> Option<AccountId> {
        (address == "moi@x.fr").then(|| AccountId::from_raw("a1"))
    }
}

impl MailboxRefs for FakeRefs {
    /// Référence de la boîte `m1`
    fn mailbox_ref(&self, id: &MailboxId) -> Option<(String, String)> {
        (id.as_str() == "m1").then(|| ("moi@x.fr".to_string(), "Archives/Factures".to_string()))
    }
    /// Boîte `m1` pour sa référence
    fn mailbox_by_ref(&self, address: &str, name: &str) -> Option<MailboxId> {
        (address == "moi@x.fr" && name == "Archives/Factures").then(|| MailboxId::from_raw("m1"))
    }
}

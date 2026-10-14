//! Constructeurs de requêtes sur les boîtes et d'actions de déplacement.

use mira_core::api::{ActionRequest, Request};
use mira_vocabulary::{AccountId, ActionKind, MailboxId};

use super::builders::key;

/// Action de déplacement vers la boîte désignée par son identifiant
pub fn move_to(mailbox: &str) -> ActionKind {
    ActionKind::Move {
        to: MailboxId::from_raw(mailbox),
    }
}

/// Création d'une boîte à la racine du compte
pub fn create_mailbox(account: &AccountId, name: &str, key_text: &str) -> Request {
    Request::Action(ActionRequest::CreateMailbox {
        account: account.clone(),
        name: name.to_string(),
        parent: None,
        idempotency_key: key(key_text),
    })
}

/// Renommage d'une boîte
pub fn rename_mailbox(mailbox: &str, name: &str, key_text: &str) -> Request {
    Request::Action(ActionRequest::RenameMailbox {
        mailbox: MailboxId::from_raw(mailbox),
        name: name.to_string(),
        idempotency_key: key(key_text),
    })
}

/// Suppression d'une boîte
pub fn delete_mailbox(mailbox: &str, key_text: &str) -> Request {
    Request::Action(ActionRequest::DeleteMailbox {
        mailbox: MailboxId::from_raw(mailbox),
        idempotency_key: key(key_text),
    })
}

/// Vidage d'une boîte
pub fn empty_mailbox(mailbox: &str, key_text: &str) -> Request {
    Request::Action(ActionRequest::EmptyMailbox {
        mailbox: MailboxId::from_raw(mailbox),
        idempotency_key: key(key_text),
    })
}

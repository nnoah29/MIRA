//! Validation et application des modifications d'un compte.

use mira_vocabulary::error::codes::invalid_input;
use mira_vocabulary::{Account, ErrorCategory, MailboxId, MiraError};

use crate::api::contract::AccountChanges;
use crate::storage::Tx;

/// Applique chaque champ présent après l'avoir validé ; rien n'est écrit en base ici
pub(super) fn apply_changes(
    tx: &Tx<'_>,
    account: &mut Account,
    changes: AccountChanges,
) -> Result<(), MiraError> {
    if let Some(name) = changes.display_name {
        account.display_name = non_blank(name)?;
    }
    if let Some(days) = changes.window_days {
        account.sync.window_days = positive_days(days)?;
    }
    if let Some(mailboxes) = changes.synced_mailboxes {
        require_own_mailboxes(tx, account, &mailboxes)?;
        account.sync.synced_mailboxes = Some(mailboxes);
    }
    if let Some(servers) = changes.servers {
        account.servers = Some(servers);
    }
    Ok(())
}

/// Refuse un nom vide
fn non_blank(name: String) -> Result<String, MiraError> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err(invalid("le nom affiché ne peut pas être vide"));
    }
    Ok(trimmed.to_string())
}

/// Refuse une durée de synchronisation de zéro jour
fn positive_days(days: u32) -> Result<u32, MiraError> {
    if days == 0 {
        return Err(invalid(
            "la durée de synchronisation doit durer au moins un jour",
        ));
    }
    Ok(days)
}

/// Refuse une boîte inconnue ou qui appartient à un autre compte
fn require_own_mailboxes(
    tx: &Tx<'_>,
    account: &Account,
    mailboxes: &[MailboxId],
) -> Result<(), MiraError> {
    for id in mailboxes {
        let owned = tx
            .get_mailbox(id)?
            .is_some_and(|mailbox| mailbox.account == account.id);
        if !owned {
            return Err(invalid("boîte inconnue pour ce compte"));
        }
    }
    Ok(())
}

/// Erreur de paramètre invalide
fn invalid(message: &str) -> MiraError {
    MiraError::new(
        ErrorCategory::InvalidInput,
        invalid_input::PARAMETER,
        message,
    )
}

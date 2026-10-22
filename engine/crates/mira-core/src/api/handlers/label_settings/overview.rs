//! Vue d'ensemble des libellés : serveur (boîtes « autre ») et locaux, avec couleur effective et réglages.

use std::collections::HashMap;

use mira_vocabulary::error::codes::not_found;
use mira_vocabulary::{ErrorCategory, LabelKind, LabelRef, LabelSettings, MailboxRole, MiraError};

use crate::api::contract::LabelOverview;
use crate::storage::Tx;

/// Erreur « libellé inconnu »
pub(super) fn label_not_found() -> MiraError {
    MiraError::new(ErrorCategory::NotFound, not_found::LABEL, "libellé inconnu")
}

/// Couleurs ARGB attribuées par défaut aux libellés qui n'en ont pas choisi
const PALETTE: [u32; 6] = [
    0xFFE8_542B,
    0xFF6A_A8D9,
    0xFF8B_C48A,
    0xFFD9_B46A,
    0xFFC9_8BB9,
    0xFF9A_8BE0,
];

/// Couleur par défaut d'un libellé, stable pour un identifiant donné
fn default_color(id: &str) -> u32 {
    let sum: usize = id.bytes().map(usize::from).sum();
    PALETTE[sum % PALETTE.len()]
}

/// Assemble la vue d'un libellé à partir de ses réglages (ou des réglages par défaut)
fn overview(
    label: LabelRef,
    account: Option<mira_vocabulary::AccountId>,
    name: String,
    messages: u64,
    settings: LabelSettings,
) -> LabelOverview {
    let color = settings.color.unwrap_or_else(|| default_color(&label.id));
    LabelOverview {
        label,
        account,
        name,
        color,
        messages,
        unused: messages == 0,
        settings,
    }
}

/// Libellés du serveur : les boîtes de rôle « autre » de chaque compte
fn server_overviews(
    tx: &Tx<'_>,
    stored: &HashMap<LabelRef, LabelSettings>,
) -> Result<Vec<LabelOverview>, MiraError> {
    let mut found = Vec::new();
    for account in tx.list_accounts()? {
        for mailbox in tx.list_mailboxes(&account.id)? {
            if mailbox.role != MailboxRole::Other {
                continue;
            }
            let label = LabelRef {
                kind: LabelKind::Server,
                id: mailbox.id.as_str().to_string(),
            };
            let settings = stored.get(&label).cloned().unwrap_or_default();
            let name = mailbox.display_name.clone();
            found.push(overview(
                label,
                Some(account.id.clone()),
                name,
                mailbox.total,
                settings,
            ));
        }
    }
    Ok(found)
}

/// Libellés locaux
fn local_overviews(
    tx: &Tx<'_>,
    stored: &HashMap<LabelRef, LabelSettings>,
) -> Result<Vec<LabelOverview>, MiraError> {
    let mut found = Vec::new();
    for local in tx.list_local_labels()? {
        let label = LabelRef {
            kind: LabelKind::Local,
            id: local.id.as_str().to_string(),
        };
        let settings = stored.get(&label).cloned().unwrap_or_default();
        let count = tx.local_label_message_count(&local.id)?;
        let mut shown = overview(label, None, local.name, count, settings);
        shown.color = shown.settings.color.unwrap_or(local.color);
        found.push(shown);
    }
    Ok(found)
}

/// Tous les libellés : locaux d'abord, puis par compte ; dans chaque groupe, `position` puis nom
pub(super) fn load_overviews(tx: &Tx<'_>) -> Result<Vec<LabelOverview>, MiraError> {
    let stored: HashMap<LabelRef, LabelSettings> = tx.list_label_settings()?.into_iter().collect();
    let mut all = local_overviews(tx, &stored)?;
    all.extend(server_overviews(tx, &stored)?);
    all.sort_by_key(|item| {
        let group = (item.label.kind == LabelKind::Server, item.account.clone());
        (
            group,
            item.settings.position.unwrap_or(u32::MAX),
            item.name.to_lowercase(),
        )
    });
    Ok(all)
}

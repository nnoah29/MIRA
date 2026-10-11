//! Préparation de la liste d'éléments FETCH.

use super::modifiers::FetchModifiers;
use crate::imap::capabilities::Capability;
use crate::imap::parser::command::Command;
use crate::imap::parser::fetch_items::FetchItem;
use crate::imap::response::error::ImapError;
use crate::imap::session::state::SessionState;

/// Indique si un élément marque le message comme lu (BODY[] sans PEEK)
pub(crate) fn has_unpeeked_body(items: &[FetchItem]) -> bool {
    items
        .iter()
        .any(|item| matches!(item, FetchItem::Body(request) if !request.peek))
}

/// Vérifie les extensions requises par les éléments et modificateurs
fn check_extensions(
    state: &mut SessionState,
    modifiers: &FetchModifiers,
    items: &[FetchItem],
) -> Result<(), ImapError> {
    if items.contains(&FetchItem::ModSeq) || modifiers.changed_since.is_some() {
        state.require_cap(Capability::Condstore)?;
        state.enabled.condstore = true;
    }
    if modifiers.vanished && !state.enabled.qresync {
        return Err(ImapError::bad("QRESYNC is not enabled"));
    }
    let gmail = |item: &FetchItem| {
        matches!(
            item,
            FetchItem::GmMsgId | FetchItem::GmThrId | FetchItem::GmLabels
        )
    };
    if items.iter().any(gmail) {
        state.require_cap(Capability::Gmail)?;
    }
    Ok(())
}

/// Vérifie les extensions puis complète la liste (UID, MODSEQ, FLAGS)
pub(crate) fn prepare_items(
    state: &mut SessionState,
    command: &Command,
    modifiers: &FetchModifiers,
    items: &mut Vec<FetchItem>,
) -> Result<(), ImapError> {
    check_extensions(state, modifiers, items)?;
    if modifiers.changed_since.is_some() && !items.contains(&FetchItem::ModSeq) {
        items.push(FetchItem::ModSeq);
    }
    if command.uid && !items.contains(&FetchItem::Uid) {
        items.insert(0, FetchItem::Uid);
    }
    if has_unpeeked_body(items) && !items.contains(&FetchItem::Flags) {
        items.push(FetchItem::Flags);
    }
    Ok(())
}

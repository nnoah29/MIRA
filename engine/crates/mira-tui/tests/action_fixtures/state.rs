//! États de test prêts pour agir sur des messages, avec les boîtes de deux comptes.
#![allow(dead_code)]

use crate::message_fixtures::summary;
use crate::reader_fixtures::list_state;
use crate::sidebar_fixtures::mailbox;
use mira_assembly::mira_vocabulary::{MailboxRole, MessageId};
use mira_tui::state::AppState;

/// État sur la liste avec `m1` et `m2`, `m1` sélectionné, et quatre boîtes dont une d'un autre compte.
pub fn state_with_mailboxes() -> AppState {
    let mut state = list_state(vec![
        summary("m1", "Bonjour", "Bob"),
        summary("m2", "Salut", "Ann"),
    ]);
    state.mailboxes.replace_items(vec![
        mailbox("a1", "INBOX", MailboxRole::Inbox, 2),
        mailbox("a1", "Archive", MailboxRole::Archive, 0),
        mailbox("a1", "Projets", MailboxRole::Other, 0),
        mailbox("a2", "Ailleurs", MailboxRole::Other, 0),
    ]);
    state
}

/// Identifiants de messages à partir de leurs noms bruts.
pub fn ids(raw: &[&str]) -> Vec<MessageId> {
    raw.iter().map(|id| MessageId::from_raw(*id)).collect()
}

/// Même état que `state_with_mailboxes` mais d'une autre session, donc d'un autre nonce.
pub fn state_with_nonce(nonce: u64) -> AppState {
    let base = state_with_mailboxes();
    let mut state = AppState::new(base.width, base.height, nonce);
    state.accounts = base.accounts.clone();
    state.mailboxes = base.mailboxes.clone();
    state.messages = base.messages.clone();
    state.focus = base.focus;
    state
}

//! Modificateurs de FETCH : CHANGEDSINCE et VANISHED.

use crate::imap::parser::token::Token;
use crate::imap::parser::uidset_format::format_uid_set;
use crate::imap::response::error::ImapError;
use crate::imap::response::reply::Reply;
use crate::imap::session::state::SessionState;

/// Modificateurs lus dans la liste finale de FETCH.
#[derive(Debug, Default)]
pub(crate) struct FetchModifiers {
    pub changed_since: Option<u64>,
    pub vanished: bool,
}

/// Lit `(CHANGEDSINCE m [VANISHED])`
pub(crate) fn parse_modifiers(token: Option<&Token>) -> Result<FetchModifiers, ImapError> {
    let mut modifiers = FetchModifiers::default();
    let mut items = token.and_then(Token::list).unwrap_or(&[]).iter();
    while let Some(item) = items.next() {
        match item.keyword().as_deref() {
            Some("CHANGEDSINCE") => {
                let value = items
                    .next()
                    .and_then(Token::text)
                    .and_then(|t| t.parse().ok());
                modifiers.changed_since =
                    Some(value.ok_or_else(|| ImapError::bad("Invalid CHANGEDSINCE"))?);
            }
            Some("VANISHED") => modifiers.vanished = true,
            _ => {}
        }
    }
    Ok(modifiers)
}

/// Annonce les UID disparus depuis le MODSEQ demandé lorsque VANISHED est présent
pub(crate) fn announce_vanished(
    state: &SessionState,
    modifiers: &FetchModifiers,
    reply: &mut Reply,
) -> Result<(), ImapError> {
    let (true, Some(since)) = (modifiers.vanished, modifiers.changed_since) else {
        return Ok(());
    };
    let (user, mailbox) = state.target()?;
    let view = state
        .ctx
        .store
        .read(|data| data.view(&user, &mailbox, since));
    let mut gone = view.map(|view| view.vanished).unwrap_or_default();
    gone.sort_unstable();
    if !gone.is_empty() {
        reply.untagged(&format!("VANISHED (EARLIER) {}", format_uid_set(&gone)));
    }
    Ok(())
}

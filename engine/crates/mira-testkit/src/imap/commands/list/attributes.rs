//! Attributs d'une boîte dans une réponse LIST.

use crate::imap::store::data::{Mailbox, UserData};

/// Attributs de hiérarchie et, si demandé, SPECIAL-USE
pub(crate) fn attributes(owner: &UserData, name: &str, mailbox: &Mailbox, special: bool) -> String {
    let prefix = format!("{name}/");
    let has_children = owner
        .mailboxes
        .keys()
        .any(|other| other.starts_with(&prefix));
    let mut attrs = vec![if has_children {
        "\\HasChildren"
    } else {
        "\\HasNoChildren"
    }];
    if let (true, Some(usage)) = (special, mailbox.special_use.as_deref()) {
        attrs.push(usage);
    }
    attrs.join(" ")
}

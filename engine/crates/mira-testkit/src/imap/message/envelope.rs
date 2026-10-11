//! Structure ENVELOPE d'un message brut.

use super::address::address_list;
use super::fields::header_value;
use crate::imap::response::quoting::nstring;

/// ENVELOPE (RFC 3501 section 7.4.2) construite à partir des en-têtes
pub(crate) fn envelope(raw: &[u8]) -> String {
    let field = |name: &str| header_value(raw, name);
    let from = field("From");
    let sender = field("Sender").or_else(|| from.clone());
    let reply_to = field("Reply-To").or_else(|| from.clone());
    format!(
        "({} {} {} {} {} {} {} {} {} {})",
        nstring(field("Date").as_deref()),
        nstring(field("Subject").as_deref()),
        address_list(from.as_deref()),
        address_list(sender.as_deref()),
        address_list(reply_to.as_deref()),
        address_list(field("To").as_deref()),
        address_list(field("Cc").as_deref()),
        address_list(field("Bcc").as_deref()),
        nstring(field("In-Reply-To").as_deref()),
        nstring(field("Message-ID").as_deref()),
    )
}

//! Écriture d'un élément FETCH.

use super::body::section_bytes;
use crate::imap::date::format::format_internal_date;
use crate::imap::message::envelope::envelope;
use crate::imap::parser::fetch_items::{BodyRequest, FetchItem};
use crate::imap::response::flags::flag_list;
use crate::imap::response::literal::literal;
use crate::imap::response::quoting::astring;
use crate::imap::store::rows::MessageRow;

/// Écrit l'élément demandé pour ce message
pub(crate) fn render_item(item: &FetchItem, row: &MessageRow) -> Vec<u8> {
    let text = match item {
        FetchItem::Uid => format!("UID {}", row.uid),
        FetchItem::Flags => format!("FLAGS {}", flag_list(&row.flags)),
        FetchItem::InternalDate => {
            format!(
                "INTERNALDATE \"{}\"",
                format_internal_date(row.internal_date)
            )
        }
        FetchItem::Size => format!("RFC822.SIZE {}", row.message.raw.len()),
        FetchItem::Envelope => format!("ENVELOPE {}", envelope(&row.message.raw)),
        FetchItem::ModSeq => format!("MODSEQ ({})", row.modseq),
        FetchItem::GmMsgId => format!("X-GM-MSGID {}", row.message.gm_msgid),
        FetchItem::GmThrId => format!("X-GM-THRID {}", row.message.gm_thrid),
        FetchItem::GmLabels => {
            let labels: Vec<String> = row.labels.iter().map(|label| astring(label)).collect();
            format!("X-GM-LABELS ({})", labels.join(" "))
        }
        FetchItem::Body(request) => return render_body(request, row),
    };
    text.into_bytes()
}

/// Élément de contenu : étiquette puis littéral
fn render_body(request: &BodyRequest, row: &MessageRow) -> Vec<u8> {
    let mut out = format!("{} ", request.label).into_bytes();
    out.extend(literal(&section_bytes(&row.message.raw, request)));
    out
}

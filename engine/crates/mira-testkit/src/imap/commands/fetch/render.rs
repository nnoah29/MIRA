//! Écriture des réponses FETCH.

use super::item::render_item;
use crate::imap::parser::fetch_items::FetchItem;
use crate::imap::response::reply::Reply;
use crate::imap::store::rows::MessageRow;

/// Écrit une réponse `* n FETCH (...)` par message
pub(crate) fn render_rows(reply: &mut Reply, rows: &[MessageRow], items: &[FetchItem]) {
    for row in rows {
        reply.raw(&render_row(row, items));
    }
}

/// Une réponse FETCH complète
fn render_row(row: &MessageRow, items: &[FetchItem]) -> Vec<u8> {
    let mut out = format!("* {} FETCH (", row.seq).into_bytes();
    for (index, item) in items.iter().enumerate() {
        if index > 0 {
            out.push(b' ');
        }
        out.extend(render_item(item, row));
    }
    out.extend_from_slice(b")\r\n");
    out
}

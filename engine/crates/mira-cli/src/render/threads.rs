//! Rendu humain des fils de conversation.

use mira_assembly::api::contract::ThreadView;
use mira_assembly::mira_vocabulary::{Page, Thread};

use super::addresses::first_party;
use super::instant::format_optional_instant;
use super::messages::messages_page_text;
use super::pagination::with_next_page;
use super::table::render_table;

/// Cellules d'une ligne de fil.
fn thread_row(thread: &Thread) -> Vec<String> {
    vec![
        thread.id.to_string(),
        format_optional_instant(thread.last_date),
        first_party(&thread.participants),
        thread.subject.clone(),
        format!("{}/{}", thread.unread_count, thread.message_count),
    ]
}

/// Tableau des fils d'une page (non lus/total), suivi de la commande de la page suivante.
pub fn threads_page_text(page: &Page<Thread>) -> String {
    let rows: Vec<Vec<String>> = page.items.iter().map(thread_row).collect();
    let table = render_table(&["ID", "DERNIER", "AVEC", "SUJET", "NON LUS/TOTAL"], &rows);
    with_next_page(table, page.next_cursor.as_ref())
}

/// Titre du fil puis ses messages, du plus ancien au plus récent.
pub fn thread_view_text(view: &ThreadView) -> String {
    let page = Page {
        items: view.messages.clone(),
        next_cursor: None,
    };
    format!(
        "Fil {} : {} ({} messages)\n{}",
        view.thread.id,
        view.thread.subject,
        view.thread.message_count,
        messages_page_text(&page)
    )
}

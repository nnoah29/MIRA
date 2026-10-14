//! Lignes de l'arbre : un compte puis, sous le compte sélectionné, ses boîtes.

use mira_assembly::mira_vocabulary::Mailbox;
use ratatui::text::{Line, Span};
use ratatui::widgets::ListItem;

use super::role_label::label;
use crate::state::AppState;
use crate::theme;

/// Ligne d'une boîte : libellé suivi du nombre de non lus s'il y en a.
fn mailbox_line(mailbox: &Mailbox) -> Line<'static> {
    let mut spans = vec![Span::raw(format!("  {}", label(mailbox)))];
    if mailbox.unread > 0 {
        spans.push(Span::styled(
            format!(" {}", mailbox.unread),
            theme::unread(),
        ));
    }
    Line::from(spans)
}

/// Toutes les lignes de l'arbre, boîtes comprises sous le compte sélectionné.
pub fn rows(state: &AppState) -> Vec<ListItem<'static>> {
    let current = state.accounts.selection.selected;
    let mut rows = Vec::new();
    for (index, account) in state.accounts.items().iter().enumerate() {
        rows.push(ListItem::new(account.address.as_str().to_string()));
        if Some(index) == current {
            let lines = state.mailboxes.items().iter().map(mailbox_line);
            rows.extend(lines.map(ListItem::new));
        }
    }
    rows
}

/// Numéro de la ligne à mettre en valeur : la boîte sélectionnée, ou le compte à défaut.
pub fn selected_row(state: &AppState) -> Option<usize> {
    let account = state.accounts.selection.selected?;
    match state.mailboxes.selection.selected {
        Some(mailbox) => Some(account + 1 + mailbox),
        None => Some(account),
    }
}

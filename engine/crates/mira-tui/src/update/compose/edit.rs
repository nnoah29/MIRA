//! Frappe : la touche va au gestionnaire de la famille du champ qui a le focus.

use super::intent::EditKey;
use super::{address_field, body_field, subject_field};
use crate::commands::Command;
use crate::state::AppState;
use crate::state::compose::FieldKind;
use crate::state::compose::viewport::body_rows;

/// Applique la touche au champ qui a le focus, puis fait suivre la fenêtre au curseur.
pub fn on_edit(state: &mut AppState, key: EditKey) -> Vec<Command> {
    let rows = body_rows(state.height);
    let Some(compose) = state.compose.open_mut() else {
        return Vec::new();
    };
    match compose.focus.kind() {
        FieldKind::Address => address_field::apply(compose, key),
        FieldKind::Subject => subject_field::apply(compose, key),
        FieldKind::Body => body_field::apply(compose, key),
    }
    compose.follow_cursor(rows);
    Vec::new()
}

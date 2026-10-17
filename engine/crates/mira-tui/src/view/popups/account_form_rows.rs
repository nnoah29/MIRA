//! Lignes des champs du formulaire d'ajout de compte : nom, valeur fenêtrée ou masquée, et colonne du curseur.

use mira_assembly::mira_vocabulary::{AuthMethodKind, Security};
use ratatui::text::{Line, Span};

use crate::state::account_form::{AccountForm, FormField};
use crate::theme;
use crate::view::compose::line_window::{cursor_offset, left_edge, visible_slice};
use crate::view::masked_input::mask;
use crate::view::safe_text::display_safe;

pub const LABEL_WIDTH: usize = 16;

/// Une ligne de champ et la colonne du curseur dans la ligne, s'il y en a un.
#[derive(Debug)]
pub struct FieldRow {
    pub line: Line<'static>,
    pub cursor: Option<usize>,
}

/// Nom de la méthode d'authentification, tel qu'affiché.
pub fn method_label(kind: AuthMethodKind) -> &'static str {
    match kind {
        AuthMethodKind::Password => "mot de passe",
        AuthMethodKind::AppPassword => "mot de passe d'application",
        AuthMethodKind::OAuth2 => "OAuth (non disponible dans cette version)",
    }
}

/// Nom du chiffrement d'un serveur.
pub fn security_label(security: Security) -> &'static str {
    match security {
        Security::Tls => "TLS",
        Security::StartTls => "STARTTLS",
        Security::Plaintext => "aucun (non chiffré)",
    }
}

/// Ligne du champ, la valeur tenant dans `width` colonnes ; le curseur n'est indiqué que sur le champ qui a le focus.
pub fn field_row(form: &AccountForm, field: FormField, width: usize) -> FieldRow {
    let focused = form.focus == field;
    let style = if focused {
        theme::unread()
    } else {
        theme::dim()
    };
    let label = Span::styled(
        format!("{:<LABEL_WIDTH$}", format!("{} :", field.label())),
        style,
    );
    let room = width.saturating_sub(LABEL_WIDTH);
    let (value, column) = match field {
        FormField::Password => password_value(form, room),
        FormField::Method => (method_value(form), 0),
        text => text_value(form, text, room),
    };
    FieldRow {
        line: Line::from(vec![label, Span::raw(value)]),
        cursor: focused.then_some(LABEL_WIDTH + column),
    }
}

/// Pastilles du mot de passe et colonne du curseur, juste après la dernière.
fn password_value(form: &AccountForm, room: usize) -> (String, usize) {
    let bullets = mask(form.password.char_count(), room.saturating_sub(1));
    let column = bullets.chars().count();
    (bullets, column)
}

/// Méthode choisie entre chevrons.
fn method_value(form: &AccountForm) -> String {
    match form.method_kind() {
        Some(kind) => format!("‹ {} ›", method_label(kind)),
        None => String::new(),
    }
}

/// Texte libre fenêtré autour du curseur et neutralisé, avec la colonne du curseur.
fn text_value(form: &AccountForm, field: FormField, room: usize) -> (String, usize) {
    let Some(buffer) = form.buffer(field) else {
        return (String::new(), 0);
    };
    let text = buffer.text();
    let cursor = buffer.cursor().1;
    let start = left_edge(&text, cursor, room);
    let shown = display_safe(&visible_slice(&text, start, room));
    (shown, cursor_offset(&text, start, cursor))
}

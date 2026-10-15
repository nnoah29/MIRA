//! Frappe dans le champ qui a le focus : texte libre, mot de passe masqué ou choix de la méthode.

use crate::commands::Command;
use crate::state::AppState;
use crate::state::account_form::{AccountForm, FormField, Phase};
use crate::state::compose::TextBuffer;
use crate::update::compose::EditKey;

/// Applique une touche au champ qui a le focus ; ignorée pendant une requête.
pub fn apply_edit(state: &mut AppState, key: EditKey) -> Vec<Command> {
    let Some(form) = state.account_form.form_mut() else {
        return Vec::new();
    };
    if !matches!(form.phase, Phase::Typing | Phase::Ready) {
        return Vec::new();
    }
    form.error = None;
    match form.focus {
        FormField::Password => edit_password(form, key),
        FormField::Method => choose_method(form, key),
        field => edit_text(form, field, key),
    }
    Vec::new()
}

/// Mot de passe : caractères ajoutés ou retirés en fin ; aucun déplacement, rien n'est jamais relu.
fn edit_password(form: &mut AccountForm, key: EditKey) {
    match key {
        EditKey::Char(c) => form.password.push(c),
        EditKey::Backspace => form.password.pop(),
        EditKey::Newline
        | EditKey::Delete
        | EditKey::Left
        | EditKey::Right
        | EditKey::Up
        | EditKey::Down
        | EditKey::Home
        | EditKey::End => {}
    }
}

/// Méthode : droite ou espace passe à la suivante, gauche à la précédente.
fn choose_method(form: &mut AccountForm, key: EditKey) {
    let count = form
        .prepared
        .as_ref()
        .map_or(0, |prepared| prepared.methods.len());
    if count == 0 {
        return;
    }
    match key {
        EditKey::Right | EditKey::Char(' ') => form.method = (form.method + 1) % count,
        EditKey::Left => form.method = (form.method + count - 1) % count,
        EditKey::Char(_)
        | EditKey::Newline
        | EditKey::Backspace
        | EditKey::Delete
        | EditKey::Up
        | EditKey::Down
        | EditKey::Home
        | EditKey::End => {}
    }
}

/// Texte libre ; modifier l'adresse déjà préparée annule la préparation et efface le mot de passe.
fn edit_text(form: &mut AccountForm, field: FormField, key: EditKey) {
    let Some(buffer) = form.buffer_mut(field) else {
        return;
    };
    let before = buffer.text();
    type_into(buffer, key);
    let changed = buffer.text() != before;
    if changed && field == FormField::Address && form.prepared.is_some() {
        form.reset_preparation();
    }
}

/// Une touche de texte sur un tampon d'une ligne.
fn type_into(buffer: &mut TextBuffer, key: EditKey) {
    match key {
        EditKey::Char(c) => buffer.insert_char(c),
        EditKey::Backspace => buffer.backspace(),
        EditKey::Delete => buffer.delete(),
        EditKey::Left => buffer.left(),
        EditKey::Right => buffer.right(),
        EditKey::Home => buffer.home(),
        EditKey::End => buffer.end(),
        EditKey::Newline | EditKey::Up | EditKey::Down => {}
    }
}

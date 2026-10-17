//! Touches du formulaire d'ajout de compte : tout caractère imprimable est du texte, jamais une action globale.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use super::context::KeyContext;
use super::keys::{is_ctrl, is_plain, lookup};
use crate::state::AppState;
use crate::update::{AccountFormIntent, EditKey, Intent};

const FORM_KEYS: [(KeyCode, Intent); 12] = [
    (KeyCode::Esc, Intent::AccountForm(AccountFormIntent::Close)),
    (
        KeyCode::Enter,
        Intent::AccountForm(AccountFormIntent::Enter),
    ),
    (
        KeyCode::Tab,
        Intent::AccountForm(AccountFormIntent::NextField),
    ),
    (
        KeyCode::Down,
        Intent::AccountForm(AccountFormIntent::NextField),
    ),
    (
        KeyCode::BackTab,
        Intent::AccountForm(AccountFormIntent::PrevField),
    ),
    (
        KeyCode::Up,
        Intent::AccountForm(AccountFormIntent::PrevField),
    ),
    (KeyCode::Backspace, edit(EditKey::Backspace)),
    (KeyCode::Delete, edit(EditKey::Delete)),
    (KeyCode::Left, edit(EditKey::Left)),
    (KeyCode::Right, edit(EditKey::Right)),
    (KeyCode::Home, edit(EditKey::Home)),
    (KeyCode::End, edit(EditKey::End)),
];

const CONFIRM_KEYS: [(KeyCode, Intent); 4] = [
    (
        KeyCode::Enter,
        Intent::AccountForm(AccountFormIntent::ConfirmClose),
    ),
    (
        KeyCode::Char('y'),
        Intent::AccountForm(AccountFormIntent::ConfirmClose),
    ),
    (
        KeyCode::Esc,
        Intent::AccountForm(AccountFormIntent::KeepEditing),
    ),
    (
        KeyCode::Char('n'),
        Intent::AccountForm(AccountFormIntent::KeepEditing),
    ),
];

/// Touche d'édition d'un champ.
const fn edit(key: EditKey) -> Intent {
    Intent::AccountForm(AccountFormIntent::Edit(key))
}

/// Contexte du formulaire ouvert, ou rien s'il est fermé ; la confirmation de fermeture a ses propres touches.
pub fn form_context(state: &AppState) -> Option<KeyContext> {
    let form = state.account_form.form()?;
    Some(if form.confirm_close {
        KeyContext::AccountFormConfirm
    } else {
        KeyContext::AccountForm
    })
}

/// Vrai pour Ctrl+Alt ensemble, que certains terminaux envoient pour AltGr.
fn is_alt_gr(key: KeyEvent) -> bool {
    key.modifiers
        .contains(KeyModifiers::CONTROL | KeyModifiers::ALT)
}

/// Intention d'une touche dans le formulaire : Ctrl+t change le chiffrement, le reste imprimable est du texte.
pub fn intent(key: KeyEvent) -> Option<Intent> {
    if is_ctrl(key, 't') && !is_alt_gr(key) {
        return Some(Intent::AccountForm(AccountFormIntent::CycleSecurity));
    }
    if let KeyCode::Char(c) = key.code {
        let typed = is_plain(key) || is_alt_gr(key);
        return typed.then_some(edit(EditKey::Char(c)));
    }
    if key.code == KeyCode::Tab && key.modifiers.contains(KeyModifiers::SHIFT) {
        return Some(Intent::AccountForm(AccountFormIntent::PrevField));
    }
    lookup(&FORM_KEYS, key)
}

/// Touches de la confirmation de fermeture d'un formulaire commencé ; toute autre touche est ignorée.
pub fn confirm_intent(key: KeyEvent) -> Option<Intent> {
    lookup(&CONFIRM_KEYS, key)
}

/// Intention d'ouverture du formulaire : `n` dans la barre latérale, ou partout tant qu'aucun compte n'existe.
pub fn open_intent(state: &AppState, key: KeyEvent) -> Option<Intent> {
    let on_sidebar = state.focus == crate::state::Pane::Sidebar || state.accounts.is_empty();
    let is_n = key.code == KeyCode::Char('n') && is_plain(key);
    (is_n && on_sidebar).then_some(Intent::AccountForm(AccountFormIntent::Open))
}

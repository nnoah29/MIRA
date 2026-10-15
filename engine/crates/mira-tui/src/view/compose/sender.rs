//! Expéditeur du brouillon ouvert : adresse du compte qui envoie, neutralisée.

use crate::state::AppState;
use crate::view::safe_text::display_safe;

/// Adresse du compte du brouillon ouvert (son identifiant si le compte n'est pas listé) ; vide sans brouillon.
pub fn sender_label(state: &AppState) -> String {
    let Some(compose) = state.compose.open() else {
        return String::new();
    };
    let account = compose.account();
    let found = state.accounts.items().iter().find(|a| &a.id == account);
    match found {
        Some(account) => display_safe(account.address.as_str()),
        None => display_safe(account.as_str()),
    }
}

//! Brouillon de compte construit depuis le formulaire, avec le message exact de ce qui manque.

use mira_assembly::api::{AccountDraft, CredentialInput, SecretInput};
use mira_assembly::mira_vocabulary::AuthMethodKind;

use super::servers::manual_servers;
use crate::state::account_form::{AccountForm, Phase};

/// Valide le formulaire sans rien consommer et rend le brouillon de compte à envoyer.
pub fn build_draft(form: &AccountForm) -> Result<AccountDraft, String> {
    let Some(prepared) = form
        .prepared
        .as_ref()
        .filter(|_| form.phase == Phase::Ready)
    else {
        return Err("Validez d'abord l'adresse (Entrée)".to_string());
    };
    let Some(auth) = form.method_kind() else {
        return Err(unavailable_message(form));
    };
    if form.password.char_count() == 0 {
        return Err("Saisissez le mot de passe".to_string());
    }
    let address = form.address.text().trim().to_string();
    let typed_name = form.name.text().trim().to_string();
    let display_name = if typed_name.is_empty() {
        address.clone()
    } else {
        typed_name
    };
    Ok(AccountDraft {
        address,
        display_name,
        provider: Some(prepared.provider.clone()),
        servers: manual_servers(form)?,
        auth,
    })
}

/// Pourquoi aucune méthode ne peut être utilisée pour ce fournisseur.
pub fn unavailable_message(form: &AccountForm) -> String {
    let oauth_only = form
        .prepared
        .as_ref()
        .is_some_and(|prepared| prepared.unavailable.contains(&AuthMethodKind::OAuth2));
    if oauth_only {
        return "OAuth : non disponible dans cette version".to_string();
    }
    "Aucune méthode d'authentification disponible pour ce fournisseur".to_string()
}

/// Identifiant à envoyer pour cette méthode, en vidant le mot de passe du formulaire ; rien si la méthode n'est pas saisissable ici.
pub fn take_credential(form: &mut AccountForm, auth: AuthMethodKind) -> Option<CredentialInput> {
    let wrap: fn(SecretInput) -> CredentialInput = match auth {
        AuthMethodKind::Password => CredentialInput::Password,
        AuthMethodKind::AppPassword => CredentialInput::AppPassword,
        AuthMethodKind::OAuth2 => return None,
    };
    Some(wrap(form.password.take_input()))
}

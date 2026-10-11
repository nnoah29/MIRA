//! Construction d'un nouveau compte et de son identité par défaut (invariant 11).

use mira_vocabulary::{
    Account, AccountId, AccountState, AuthMethodKind, EmailAddress, Identity, IdentityId,
    ProfileId, ProviderId, ServerSettings, SyncSettings, Timestamp,
};

/// Éléments d'un compte à créer, tous déjà validés.
#[derive(Debug, Clone)]
pub struct NewAccountParts {
    pub id: AccountId,
    pub identity_id: IdentityId,
    pub profile: ProfileId,
    pub address: EmailAddress,
    pub display_name: String,
    pub provider: ProviderId,
    pub auth: AuthMethodKind,
    pub servers: Option<ServerSettings>,
    pub now: Timestamp,
}

/// Construit le compte, connecté puisque le test de connexion vient de réussir
pub fn build_account(parts: &NewAccountParts) -> Account {
    Account {
        id: parts.id.clone(),
        profile: parts.profile.clone(),
        address: parts.address.clone(),
        display_name: shown_name(parts),
        provider: parts.provider.clone(),
        auth: parts.auth,
        servers: parts.servers.clone(),
        sync: SyncSettings::default(),
        state: AccountState::Connected,
        created_at: parts.now,
    }
}

/// Construit l'identité par défaut : l'adresse et le nom du compte
pub fn build_default_identity(parts: &NewAccountParts) -> Identity {
    Identity {
        id: parts.identity_id.clone(),
        account: parts.id.clone(),
        address: parts.address.clone(),
        display_name: shown_name(parts),
        signature: None,
        is_default: true,
    }
}

/// Nom affiché : celui saisi, ou l'adresse s'il est vide
fn shown_name(parts: &NewAccountParts) -> String {
    let name = parts.display_name.trim();
    if name.is_empty() {
        return parts.address.as_str().to_string();
    }
    name.to_string()
}

//! Conversion entre une ligne de la table `accounts` et un `Account`.

use mira_vocabulary::error::codes::{local_storage, not_found};
use mira_vocabulary::{
    Account, AccountId, AuthMethodKind, EmailAddress, ErrorCategory, ErrorContext, MiraError,
    ProfileId, ProviderId, Timestamp,
};

use crate::storage::json_columns::{from_json, to_json};

pub(super) const COLUMNS: &str = "id, profile_id, address, display_name, provider_id, auth_kind, servers_json, sync_json, state_json, created_at";

/// Ligne brute de la table `accounts`, dans l'ordre de `COLUMNS`.
pub(super) struct RawAccount {
    id: String,
    profile: String,
    address: String,
    display_name: String,
    provider: String,
    auth_kind: String,
    servers_json: Option<String>,
    sync_json: String,
    state_json: String,
    created_at: i64,
}

impl RawAccount {
    /// Lit une ligne sélectionnée avec `COLUMNS`
    pub(super) fn from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Self> {
        Ok(Self {
            id: row.get(0)?,
            profile: row.get(1)?,
            address: row.get(2)?,
            display_name: row.get(3)?,
            provider: row.get(4)?,
            auth_kind: row.get(5)?,
            servers_json: row.get(6)?,
            sync_json: row.get(7)?,
            state_json: row.get(8)?,
            created_at: row.get(9)?,
        })
    }

    /// Reconstruit le compte du domaine depuis la ligne
    pub(super) fn into_account(self) -> Result<Account, MiraError> {
        Ok(Account {
            id: AccountId::from_raw(self.id),
            profile: ProfileId::from_raw(self.profile),
            address: EmailAddress::parse(&self.address)?,
            display_name: self.display_name,
            provider: ProviderId::new(&self.provider),
            auth: auth_kind_from_name(&self.auth_kind)?,
            servers: self.servers_json.as_deref().map(from_json).transpose()?,
            sync: from_json(&self.sync_json)?,
            state: from_json(&self.state_json)?,
            created_at: Timestamp::from_millis(self.created_at),
        })
    }
}

/// Colonnes JSON d'un compte, sérialisées avant l'écriture.
pub(super) struct AccountJson {
    pub(super) servers: Option<String>,
    pub(super) sync: String,
    pub(super) state: String,
}

impl AccountJson {
    /// Sérialise les valeurs composées du compte
    pub(super) fn of(account: &Account) -> Result<Self, MiraError> {
        Ok(Self {
            servers: account.servers.as_ref().map(to_json).transpose()?,
            sync: to_json(&account.sync)?,
            state: to_json(&account.state)?,
        })
    }
}

/// Donne le nom stocké d'une méthode d'authentification
pub(super) fn auth_kind_name(kind: AuthMethodKind) -> &'static str {
    match kind {
        AuthMethodKind::OAuth2 => "oauth2",
        AuthMethodKind::AppPassword => "app_password",
        AuthMethodKind::Password => "password",
    }
}

/// Relit une méthode d'authentification depuis son nom stocké
fn auth_kind_from_name(name: &str) -> Result<AuthMethodKind, MiraError> {
    match name {
        "oauth2" => Ok(AuthMethodKind::OAuth2),
        "app_password" => Ok(AuthMethodKind::AppPassword),
        "password" => Ok(AuthMethodKind::Password),
        _ => Err(MiraError::new(
            ErrorCategory::LocalStorage,
            local_storage::CORRUPTED,
            "méthode d'authentification inconnue en base",
        )),
    }
}

/// Erreur « compte inconnu » portant l'identifiant visé
pub(super) fn account_not_found(id: &AccountId) -> MiraError {
    MiraError::new(
        ErrorCategory::NotFound,
        not_found::ACCOUNT,
        "compte inconnu",
    )
    .with_context(ErrorContext {
        account: Some(id.clone()),
        ..ErrorContext::default()
    })
}

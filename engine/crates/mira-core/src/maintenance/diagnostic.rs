//! Diagnostic exportable : versions, volumes, intégrité et état des comptes, sans secret ni contenu.

use mira_vocabulary::{
    AccountId, AccountState, AuthMethodKind, CONTRACT_VERSION, ContractVersion, MiraError,
    ProfileId, ProviderId,
};
use serde::Serialize;

use crate::storage::repositories::maintenance::StorageStats;
use crate::storage::{Database, schema_version};

/// État d'un compte, sans adresse ni identifiant de connexion.
#[derive(Debug, Serialize)]
struct AccountDiagnostic {
    id: AccountId,
    provider: ProviderId,
    auth: AuthMethodKind,
    state: AccountState,
    pending_actions: u64,
}

/// Contenu du fichier de diagnostic.
#[derive(Debug, Serialize)]
pub struct DiagnosticReport {
    contract_version: ContractVersion,
    schema_version: usize,
    profile: ProfileId,
    storage: StorageStats,
    integrity_ok: bool,
    accounts: Vec<AccountDiagnostic>,
}

/// Rassemble le diagnostic du profil depuis la base
pub fn build_diagnostic(
    database: &Database,
    profile: &ProfileId,
) -> Result<DiagnosticReport, MiraError> {
    database.read(|tx| {
        let mut accounts = Vec::new();
        for account in tx.list_accounts()? {
            accounts.push(AccountDiagnostic {
                pending_actions: tx.pending_action_count(&account.id)?,
                id: account.id,
                provider: account.provider,
                auth: account.auth,
                state: account.state,
            });
        }
        Ok(DiagnosticReport {
            contract_version: CONTRACT_VERSION,
            schema_version: schema_version(),
            profile: profile.clone(),
            storage: tx.storage_stats()?,
            integrity_ok: tx.integrity_check()?,
            accounts,
        })
    })
}

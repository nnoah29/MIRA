//! Export du profil (D-29) : configuration et intentions en JSON, sans secret ni miroir.

use mira_vocabulary::{
    Account, CONTRACT_VERSION, ContractVersion, Draft, Identity, MiraError, PendingAction,
    ProfileId,
};
use serde::Serialize;

use crate::storage::Database;
use crate::storage::repositories::allowances::{RemoteContentAllowance, RemoteContentScope};

const FORMAT_VERSION: u32 = 1;

/// Autorisation de contenu distant, avec sa portée en clair.
#[derive(Debug, Serialize)]
struct AllowanceExport {
    scope: &'static str,
    value: String,
}

/// Contenu du fichier d'export : réglages des comptes et intentions de l'utilisateur.
#[derive(Debug, Serialize)]
pub struct ProfileExport {
    format_version: u32,
    contract_version: ContractVersion,
    profile: ProfileId,
    accounts: Vec<Account>,
    identities: Vec<Identity>,
    remote_content_allowances: Vec<AllowanceExport>,
    drafts: Vec<Draft>,
    actions: Vec<PendingAction>,
}

/// Convertit une autorisation en forme exportable
fn allowance_export(allowance: RemoteContentAllowance) -> AllowanceExport {
    let scope = match allowance.scope {
        RemoteContentScope::Message => "message",
        RemoteContentScope::Sender => "sender",
    };
    AllowanceExport {
        scope,
        value: allowance.value,
    }
}

/// Rassemble la configuration et les intentions du profil depuis la base
pub fn build_profile_export(
    database: &Database,
    profile: &ProfileId,
) -> Result<ProfileExport, MiraError> {
    database.read(|tx| {
        let accounts = tx.list_accounts()?;
        let mut export = ProfileExport {
            format_version: FORMAT_VERSION,
            contract_version: CONTRACT_VERSION,
            profile: profile.clone(),
            identities: Vec::new(),
            remote_content_allowances: tx
                .list_remote_content_allowances()?
                .into_iter()
                .map(allowance_export)
                .collect(),
            drafts: Vec::new(),
            actions: Vec::new(),
            accounts: Vec::new(),
        };
        for account in accounts {
            export.identities.extend(tx.list_identities(&account.id)?);
            export.drafts.extend(tx.list_drafts(&account.id)?);
            export.actions.extend(tx.list_actions(&account.id, &[])?);
            export.accounts.push(account);
        }
        Ok(export)
    })
}

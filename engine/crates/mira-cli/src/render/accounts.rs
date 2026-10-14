//! Rendu humain des comptes.

use mira_assembly::mira_vocabulary::Account;

use super::label::json_label;
use super::table::render_table;

/// Cellules d'une ligne de compte.
fn account_row(account: &Account) -> Vec<String> {
    vec![
        account.id.to_string(),
        account.address.as_str().to_string(),
        account.provider.as_str().to_string(),
        json_label(&account.auth),
        json_label(&account.state),
    ]
}

/// Tableau des comptes : identifiant, adresse, provider, authentification, état.
pub fn accounts_table(accounts: &[Account]) -> String {
    let rows: Vec<Vec<String>> = accounts.iter().map(account_row).collect();
    render_table(&["ID", "ADRESSE", "PROVIDER", "AUTH", "ÉTAT"], &rows)
}

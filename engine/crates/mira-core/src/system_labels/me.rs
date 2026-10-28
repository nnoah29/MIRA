//! Identité de l'utilisateur pour un compte : adresses et noms sous lesquels on peut lui écrire ou le citer.

use mira_vocabulary::{AccountId, MiraError};

use crate::storage::Tx;

/// Adresses et noms en minuscules ; un prénom de trois lettres et plus compte aussi.
#[derive(Debug)]
pub struct Me {
    pub addresses: Vec<String>,
    pub names: Vec<String>,
}

impl Me {
    /// Cette adresse est-elle l'une des miennes ?
    pub fn owns(&self, address: &str) -> bool {
        let lower = address.to_lowercase();
        self.addresses.contains(&lower)
    }
}

/// Charge l'identité d'un compte : adresse et nom du compte, puis ceux de ses identités d'envoi
pub fn load_me(tx: &Tx<'_>, account: &AccountId) -> Result<Me, MiraError> {
    let mut addresses = Vec::new();
    let mut shown = Vec::new();
    if let Some(found) = tx.get_account(account)? {
        addresses.push(found.address.as_str().to_lowercase());
        shown.push(found.display_name);
    }
    for identity in tx.list_identities(account)? {
        addresses.push(identity.address.as_str().to_lowercase());
        shown.push(identity.display_name);
    }
    addresses.sort();
    addresses.dedup();
    Ok(Me {
        addresses,
        names: names_with_first_names(&shown),
    })
}

/// Noms d'affichage en minuscules, sans doublon, chacun suivi de son prénom s'il compte trois lettres ou plus
fn names_with_first_names(shown: &[String]) -> Vec<String> {
    let mut names: Vec<String> = Vec::new();
    for name in shown.iter().map(|n| n.trim().to_lowercase()) {
        let first = name.split_whitespace().next().unwrap_or("").to_string();
        for candidate in [name, first] {
            if candidate.chars().count() > 2 && !names.contains(&candidate) {
                names.push(candidate);
            }
        }
    }
    names
}

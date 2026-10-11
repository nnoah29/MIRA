//! Adresse brute lue dans un en-tête reçu (jamais rejetée, même mal formée).

use serde::{Deserialize, Serialize};

/// Adresse d'un en-tête tel que reçu : nom d'affichage facultatif et adresse non validée.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AddressEntry {
    pub name: Option<String>,
    pub address: String,
}

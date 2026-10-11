//! Pagination par curseur (invariant 29 : toute liste est bornée).

use serde::{Deserialize, Serialize};

use crate::ids::keys::Cursor;

/// Demande de page : curseur de reprise et taille souhaitée.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PageRequest {
    pub cursor: Option<Cursor>,
    pub limit: u32,
}

impl Default for PageRequest {
    /// Donne la première page de 50 éléments.
    fn default() -> Self {
        Self {
            cursor: None,
            limit: 50,
        }
    }
}

impl PageRequest {
    /// Donne la taille de page effective, bornée entre 1 et 500.
    pub fn bounded_limit(&self) -> u32 {
        self.limit.clamp(1, 500)
    }
}

/// Page de résultats et curseur de la suivante.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Page<T> {
    pub items: Vec<T>,
    pub next_cursor: Option<Cursor>,
}

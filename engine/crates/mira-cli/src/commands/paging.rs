//! Options de pagination par curseur communes aux listes : `--limit` et `--cursor`.

use clap::Args;
use mira_assembly::mira_vocabulary::{Cursor, PageRequest};

/// Taille de page et curseur de reprise d'une liste.
#[derive(Debug, Args)]
pub struct PageArgs {
    /// Nombre maximal d'éléments rendus (1 à 500).
    #[arg(long, default_value_t = 50)]
    pub limit: u32,
    /// Curseur rendu par la page précédente pour obtenir la suivante.
    #[arg(long)]
    pub cursor: Option<String>,
}

impl PageArgs {
    /// Traduit les options en demande de page du contrat.
    pub fn page_request(&self) -> PageRequest {
        PageRequest {
            cursor: self.cursor.clone().map(Cursor),
            limit: self.limit,
        }
    }
}

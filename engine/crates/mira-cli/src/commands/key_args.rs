//! Option `--key` des commandes d'écriture : clé d'idempotence fournie ou fabriquée.

use clap::Args;
use mira_assembly::mira_vocabulary::IdempotencyKey;

use crate::argument_context::ArgumentContext;

/// Clé d'idempotence d'une écriture.
#[derive(Debug, Args)]
pub struct KeyArgs {
    /// Clé d'idempotence (générée si absente) ; la rejouer ne crée qu'une seule action.
    #[arg(long)]
    pub key: Option<String>,
}

impl KeyArgs {
    /// Rend la clé fournie, sinon une clé fabriquée à partir du générateur d'identifiants.
    pub fn idempotency_key(&self, context: &ArgumentContext) -> IdempotencyKey {
        IdempotencyKey(self.key.clone().unwrap_or_else(|| context.generated_key()))
    }
}

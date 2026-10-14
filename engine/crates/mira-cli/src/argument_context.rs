//! Ressources dont l'analyse d'une commande a besoin : entrée standard (secrets) et identifiants.

use std::io::Read;

use mira_assembly::api::SecretInput;
use mira_assembly::mira_vocabulary::MiraError;
use mira_assembly::ports::IdGenerator;

use crate::argument_error::invalid_argument;

const MAX_SECRET_BYTES: usize = 4096;

/// Entrée standard et générateur d'identifiants prêtés à la construction d'une requête.
pub struct ArgumentContext<'a> {
    pub stdin: &'a mut dyn Read,
    pub ids: &'a dyn IdGenerator,
}

impl ArgumentContext<'_> {
    /// Lit la première ligne de l'entrée standard comme secret (jamais d'argument de ligne de commande).
    pub fn read_secret(&mut self) -> Result<SecretInput, MiraError> {
        let mut bytes = Vec::new();
        let mut byte = [0u8; 1];
        while bytes.len() < MAX_SECRET_BYTES {
            let count = self.stdin.read(&mut byte).unwrap_or(0);
            if count == 0 || byte[0] == b'\n' {
                break;
            }
            bytes.push(byte[0]);
        }
        let text = String::from_utf8_lossy(&bytes);
        let line = text.trim_end_matches('\r');
        if line.is_empty() {
            return Err(invalid_argument("aucun secret reçu sur l'entrée standard"));
        }
        Ok(SecretInput::new(line))
    }

    /// Fabrique une clé d'idempotence quand l'utilisateur n'en fournit pas.
    pub fn generated_key(&self) -> String {
        format!("cli-{}", self.ids.next_id())
    }
}

impl std::fmt::Debug for ArgumentContext<'_> {
    /// Affiche le type sans lire l'entrée standard.
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ArgumentContext")
            .finish_non_exhaustive()
    }
}

//! Ressources dont l'analyse d'une commande a besoin : entrée standard (secrets) et identifiants.

use std::io::{ErrorKind, Read};

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
        let bytes = self.read_first_line()?;
        let mut line = String::from_utf8(bytes).map_err(|_| {
            invalid_argument("le secret reçu sur l'entrée standard n'est pas de l'UTF-8")
        })?;
        while line.ends_with('\r') {
            line.pop();
        }
        if line.is_empty() {
            return Err(invalid_argument("aucun secret reçu sur l'entrée standard"));
        }
        Ok(SecretInput::new(line))
    }

    /// Lit les octets jusqu'à la fin de ligne ; refuse une erreur de lecture ou plus de 4096 octets.
    fn read_first_line(&mut self) -> Result<Vec<u8>, MiraError> {
        let mut bytes = Vec::new();
        let mut byte = [0u8; 1];
        loop {
            match self.stdin.read(&mut byte) {
                Ok(0) => return Ok(bytes),
                Ok(_) if byte[0] == b'\n' => return Ok(bytes),
                Ok(_) if bytes.len() >= MAX_SECRET_BYTES => {
                    return Err(invalid_argument("le secret dépasse 4096 octets"));
                }
                Ok(_) => bytes.push(byte[0]),
                Err(error) if error.kind() == ErrorKind::Interrupted => {}
                Err(_) => return Err(invalid_argument("lecture du secret impossible")),
            }
        }
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

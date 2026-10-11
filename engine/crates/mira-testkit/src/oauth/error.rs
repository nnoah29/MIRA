//! Construction d'erreurs d'entrée invalide, sans valeur secrète dans le message.

use std::io;

/// Crée une erreur `InvalidInput` à partir d'un message qui ne contient aucun secret
pub(crate) fn invalid_input(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidInput, message.to_string())
}

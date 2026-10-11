//! Flux de sortie du CLI, injectés pour pouvoir les capturer dans les tests.

use std::fmt;
use std::io::Write;

/// Sortie standard et sortie d'erreur d'une exécution du CLI.
pub struct CliIo<'a> {
    pub out: &'a mut dyn Write,
    pub err: &'a mut dyn Write,
}

impl<'a> CliIo<'a> {
    /// Crée les flux à partir de deux écrivains (processus réel ou tampons de test).
    pub fn new(out: &'a mut dyn Write, err: &'a mut dyn Write) -> Self {
        Self { out, err }
    }
}

impl fmt::Debug for CliIo<'_> {
    /// Affiche le type sans exposer le contenu des flux.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CliIo").finish_non_exhaustive()
    }
}

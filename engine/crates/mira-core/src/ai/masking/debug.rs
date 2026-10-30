//! Affichage de diagnostic du masqueur et du restituteur : des comptes, jamais une valeur.

use std::fmt;

use super::apply::Masker;
use super::restore::Restorer;

impl fmt::Debug for Masker {
    /// Montre le mode et le nombre de valeurs retenues, pas leur contenu.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("Masker")
            .field("enabled", &self.enabled)
            .field("masked", &self.table.len())
            .finish_non_exhaustive()
    }
}

impl fmt::Debug for Restorer {
    /// Montre le nombre de repères connus, pas les valeurs ni l'attente.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("Restorer")
            .field("placeholders", &self.table.len())
            .finish_non_exhaustive()
    }
}

//! Affichage de débogage d'un flux : jamais de vérificateur, d'état ni d'URL d'autorisation.

use std::fmt;

use super::pending::PendingFlow;

impl fmt::Debug for PendingFlow {
    /// Affiche l'adresse de redirection et l'étape, sans vérificateur ni état
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PendingFlow")
            .field("redirect_uri", &self.redirect_uri)
            .field("phase", &self.phase)
            .finish()
    }
}

//! Affichage de débogage du registre, expurgé : aucun secret ne sort (P9).

use super::Credentials;

impl std::fmt::Debug for Credentials {
    /// Affiche le registre sans jamais révéler de mot de passe ni de jeton
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("Credentials { *** }")
    }
}

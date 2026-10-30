//! Altération volontaire du fichier du profil, pour éprouver la tolérance aux caches illisibles.

use super::ModelsFixture;

impl ModelsFixture {
    /// Rend illisible la liste de modèles gardée pour cette connexion, directement dans le fichier du profil
    pub fn corrupt_models(&self, connection: &str) {
        let raw = rusqlite::Connection::open(&self.database_path).unwrap();
        raw.execute(
            "UPDATE ai_connections SET models_json = 'pas du json', models_fetched_at = 5 WHERE id = ?1",
            [connection],
        )
        .unwrap();
    }
}

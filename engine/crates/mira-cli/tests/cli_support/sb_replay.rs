//! Soumission d'actions hors ligne et rejeu de la file sur le banc des scénarios.

use super::sb_world::SbWorld;

impl SbWorld {
    /// Soumet une action sans attendre (l'horloge n'avance pas, elle reste en attente) et rend son identifiant.
    pub async fn apply_pending(&self, kind: &str, message: &str) -> String {
        let line = ["action", "apply", kind, "--message", message];
        let applied = self.run_ok(&line).await;
        applied["data"]["data"][0].as_str().unwrap().to_string()
    }

    /// Laisse la file se rejouer : une action de plus sur ce message est attendue pendant que l'horloge avance.
    pub async fn replay_queue_through(&self, message: &str) {
        let pump = self.pump_clock();
        let line = ["action", "apply", "flag", "--message", message, "--wait"];
        let result = self.run(&line).await;
        pump.abort();
        assert_eq!(result.code, 0, "{}", result.stderr);
    }
}

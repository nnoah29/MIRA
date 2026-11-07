//! Pilotage des flux ouverts d'une connexion : crédits, annulation d'un flux ou de tous.

use super::exchanges::{Entry, Exchanges};
use super::stop::Stop;

impl Exchanges {
    /// Ajoute des crédits au flux `id` ; sans effet sur un identifiant inconnu ou un appel.
    pub fn add_credit(&self, id: u64, n: u32) {
        if let Some(Entry::Stream(stream)) = self.entries().get(&id) {
            stream.add_credit(n);
        }
    }

    /// Annule le flux `id` ; sans effet sur un identifiant inconnu ou un appel.
    pub fn cancel(&self, id: u64) {
        if let Some(Entry::Stream(stream)) = self.entries().get(&id) {
            stream.stop(Stop::Cancelled);
        }
    }

    /// Annule tous les flux : la connexion se ferme. Les appels continuent.
    pub fn cancel_all(&self) {
        for entry in self.entries().values() {
            if let Entry::Stream(stream) = entry {
                stream.stop(Stop::Cancelled);
            }
        }
    }
}

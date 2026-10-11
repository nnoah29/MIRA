//! Poignée publique du magasin, partageable entre le serveur et les tests.

use std::sync::{Arc, Mutex, MutexGuard};

use tokio::sync::watch;

use super::data::StoreData;
use super::faults::Faults;

/// Éléments partagés derrière la poignée.
#[derive(Debug)]
pub(crate) struct Shared {
    pub data: Mutex<StoreData>,
    pub version: watch::Sender<u64>,
    pub faults: Faults,
}

/// Magasin de messages partagé, manipulable par les tests « depuis un autre appareil ».
#[derive(Debug, Clone)]
pub struct FakeMailStore {
    pub(crate) shared: Arc<Shared>,
}

impl Default for FakeMailStore {
    /// Magasin vide
    fn default() -> Self {
        Self::new()
    }
}

impl FakeMailStore {
    /// Crée un magasin vide
    pub fn new() -> Self {
        Self {
            shared: Arc::new(Shared {
                data: Mutex::new(StoreData::default()),
                version: watch::channel(0).0,
                faults: Faults::default(),
            }),
        }
    }

    /// Verrouille les données en ignorant un éventuel empoisonnement
    fn lock(&self) -> MutexGuard<'_, StoreData> {
        self.shared
            .data
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    /// Lit les données sans les modifier
    pub(crate) fn read<R>(&self, action: impl FnOnce(&StoreData) -> R) -> R {
        action(&self.lock())
    }

    /// Modifie les données puis réveille les sessions en attente
    pub(crate) fn write<R>(&self, action: impl FnOnce(&mut StoreData) -> R) -> R {
        let result = action(&mut self.lock());
        self.shared.version.send_modify(|version| *version += 1);
        result
    }

    /// Abonnement aux changements du magasin
    pub(crate) fn subscribe(&self) -> watch::Receiver<u64> {
        self.shared.version.subscribe()
    }
}

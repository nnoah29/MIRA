//! Suivi des tâches de connexion pour pouvoir les couper.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Mutex, MutexGuard};

use tokio::task::JoinHandle;

/// Connexions acceptées et tâches associées.
#[derive(Debug, Default)]
pub(crate) struct Tracker {
    total: AtomicUsize,
    tasks: Mutex<Vec<JoinHandle<()>>>,
}

impl Tracker {
    /// Verrouille la liste des tâches en ignorant un éventuel empoisonnement
    fn tasks(&self) -> MutexGuard<'_, Vec<JoinHandle<()>>> {
        self.tasks
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    /// Enregistre une nouvelle connexion
    pub(crate) fn register(&self, task: JoinHandle<()>) {
        self.total.fetch_add(1, Ordering::SeqCst);
        let mut tasks = self.tasks();
        tasks.retain(|known| !known.is_finished());
        tasks.push(task);
    }

    /// Nombre total de connexions acceptées depuis le démarrage
    pub(crate) fn total(&self) -> usize {
        self.total.load(Ordering::SeqCst)
    }

    /// Nombre de connexions encore ouvertes
    pub(crate) fn active(&self) -> usize {
        self.tasks()
            .iter()
            .filter(|task| !task.is_finished())
            .count()
    }

    /// Coupe brutalement toutes les connexions
    pub(crate) fn abort_all(&self) {
        for task in self.tasks().drain(..) {
            task.abort();
        }
    }
}

//! Sonde de trousseau scriptée : verrouillée pour un nombre de lectures donné ou jusqu'à `release`, compte les sondages.

use std::sync::atomic::{AtomicUsize, Ordering};

use mira_daemon::unlock::KeyringProbe;

/// Sonde dont les lectures répondent « verrouillé » tant qu'il lui reste des refus à donner.
#[derive(Debug)]
pub struct ScriptedProbe {
    refusals: AtomicUsize,
    probes: AtomicUsize,
}

impl ScriptedProbe {
    /// Répond « verrouillé » aux `refusals` premiers sondages puis « ouvert ».
    pub fn locked_for(refusals: usize) -> Self {
        Self {
            refusals: AtomicUsize::new(refusals),
            probes: AtomicUsize::new(0),
        }
    }

    /// Déverrouille : les sondages suivants répondent « ouvert ».
    pub fn release(&self) {
        self.refusals.store(0, Ordering::SeqCst);
    }

    /// Nombre de sondages reçus.
    pub fn probes(&self) -> usize {
        self.probes.load(Ordering::SeqCst)
    }
}

impl KeyringProbe for ScriptedProbe {
    /// Vrai tant qu'il reste des refus.
    fn locked(&self) -> bool {
        self.probes.fetch_add(1, Ordering::SeqCst);
        self.refusals
            .try_update(Ordering::SeqCst, Ordering::SeqCst, |left| {
                left.checked_sub(1)
            })
            .is_ok()
    }
}

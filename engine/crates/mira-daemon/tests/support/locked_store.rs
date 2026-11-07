//! Faux trousseau : verrouillé pour un nombre d'accès donné ou jusqu'à `release`, compte les sondes reçues.

use std::sync::atomic::{AtomicUsize, Ordering};

use mira_assembly::mira_vocabulary::error::codes::secrets;
use mira_assembly::mira_vocabulary::{ErrorCategory, MiraError, SecretLocation};
use mira_assembly::ports::{SecretStore, SecretValue};

/// Trousseau dont les lectures échouent en « verrouillé » tant qu'il lui reste des refus à donner.
#[derive(Debug)]
pub struct LockedStore {
    refusals: AtomicUsize,
    probes: AtomicUsize,
}

impl LockedStore {
    /// Refuse les `refusals` premières lectures puis répond normalement.
    pub fn locked_for(refusals: usize) -> Self {
        Self {
            refusals: AtomicUsize::new(refusals),
            probes: AtomicUsize::new(0),
        }
    }

    /// Déverrouille : les lectures suivantes réussissent.
    pub fn release(&self) {
        self.refusals.store(0, Ordering::SeqCst);
    }

    /// Nombre de lectures reçues.
    pub fn probes(&self) -> usize {
        self.probes.load(Ordering::SeqCst)
    }
}

impl SecretStore for LockedStore {
    /// Accepte sans rien garder.
    fn put(&self, _key: &dyn SecretLocation, _value: &SecretValue) -> Result<(), MiraError> {
        Ok(())
    }

    /// Refuse tant que le trousseau est verrouillé, puis ne trouve rien.
    fn get(&self, _key: &dyn SecretLocation) -> Result<Option<SecretValue>, MiraError> {
        self.probes.fetch_add(1, Ordering::SeqCst);
        let locked = self
            .refusals
            .try_update(Ordering::SeqCst, Ordering::SeqCst, |left| {
                left.checked_sub(1)
            })
            .is_ok();
        if locked {
            return Err(MiraError::new(
                ErrorCategory::Secrets,
                secrets::KEYRING_LOCKED,
                "trousseau verrouillé",
            ));
        }
        Ok(None)
    }

    /// Accepte sans rien faire.
    fn delete(&self, _key: &dyn SecretLocation) -> Result<(), MiraError> {
        Ok(())
    }
}

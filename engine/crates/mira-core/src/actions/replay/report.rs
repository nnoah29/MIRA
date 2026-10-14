//! Bilan d'un passage de rejeu : ce qui a abouti, échoué, devenu incertain, et la raison d'un arrêt.

/// Pourquoi le rejeu s'est arrêté avant d'avoir vidé la file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReplayStop {
    AuthRequired,
    Retry { code: String },
}

/// Résultat d'un passage de rejeu sur un compte.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ReplayReport {
    pub succeeded: u32,
    pub failed: u32,
    pub uncertain: u32,
    pub stopped: Option<ReplayStop>,
}

/// Issue du rejeu d'une seule action.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum StepOutcome {
    Succeeded,
    Failed,
    Uncertain,
    Stop(ReplayStop),
}

impl ReplayReport {
    /// Compte l'issue d'une action ; rend vrai si le rejeu doit s'arrêter
    pub(crate) fn record(&mut self, outcome: StepOutcome) -> bool {
        match outcome {
            StepOutcome::Succeeded => self.succeeded += 1,
            StepOutcome::Failed => self.failed += 1,
            StepOutcome::Uncertain => self.uncertain += 1,
            StepOutcome::Stop(reason) => {
                self.stopped = Some(reason);
                return true;
            }
        }
        false
    }
}

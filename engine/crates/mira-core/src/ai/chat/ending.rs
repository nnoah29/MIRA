//! Issue d'une génération : delta final, libellé stable pour le journal et clôture de l'opération.

use mira_vocabulary::{AiError, ChatDelta, MiraError, StopReason};

use super::guard::Interrupt;
use crate::operations::{OperationHandle, OperationRegistry};

/// Façon dont la génération s'est terminée.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum Ending {
    Finished(StopReason),
    Failed(AiError),
    Cancelled,
    Abandoned,
}

impl Ending {
    /// Dernier delta à remettre au lecteur ; aucun si le lecteur est parti.
    pub(super) fn final_delta(&self) -> Option<ChatDelta> {
        let reason = match self {
            Self::Finished(reason) => reason.clone(),
            Self::Failed(error) => StopReason::Failed {
                code: error.code().0.to_string(),
                retry_after_secs: retry_after(error),
            },
            Self::Cancelled => StopReason::Cancelled,
            Self::Abandoned => return None,
        };
        Some(ChatDelta::Done { reason })
    }

    /// Libellé court et stable de l'issue, pour l'événement de fin.
    pub(super) fn outcome(&self) -> String {
        match self.final_delta() {
            Some(ChatDelta::Done { reason }) => match reason {
                StopReason::EndTurn => "end_turn".to_string(),
                StopReason::MaxTokens => "max_tokens".to_string(),
                StopReason::Cancelled => "cancelled".to_string(),
                StopReason::Failed { code, .. } => code,
            },
            _ => "cancelled".to_string(),
        }
    }

    /// Clôt l'opération une seule fois : réussite, échec avec son code, ou annulation.
    pub(super) fn close(&self, handle: &OperationHandle, operations: &OperationRegistry) {
        match self {
            Self::Finished(_) | Self::Cancelled => handle.complete(Ok(())),
            Self::Failed(error) => handle.complete(Err(&MiraError::from(error.clone()))),
            Self::Abandoned => {
                let _ = operations.cancel(handle.id());
                handle.complete(Ok(()));
            }
        }
    }
}

impl From<Interrupt> for Ending {
    /// Une attente interrompue termine la génération par une annulation.
    fn from(interrupt: Interrupt) -> Self {
        match interrupt {
            Interrupt::Cancelled => Self::Cancelled,
            Interrupt::Abandoned => Self::Abandoned,
        }
    }
}

/// Délai d'attente annoncé par le service, seulement pour une limite de débit.
fn retry_after(error: &AiError) -> Option<u64> {
    match error {
        AiError::RateLimited { retry_after_secs } => *retry_after_secs,
        _ => None,
    }
}

//! Opération de modification en cours : une seule à la fois, suivie par un jeton.

use super::SettingsState;

/// Modification demandée au moteur et pas encore confirmée.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Operation {
    UpdateAccount,
    PauseAccount,
    ResumeAccount,
    DeleteAccount,
    AddIdentity,
    UpdateIdentity,
    SetDefaultIdentity,
    CheckIntegrity,
    PurgeCache,
    RebuildIndex,
    ExportDiagnostic,
    ExportProfile,
}

/// Lecture à refaire une fois l'opération réussie.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FollowUp {
    Accounts,
    Identities,
    Stats,
    Nothing,
}

impl Operation {
    /// Message de succès affiché dans la barre d'état.
    pub fn done_message(self) -> &'static str {
        match self {
            Operation::UpdateAccount => "Compte modifié",
            Operation::PauseAccount => "Compte mis en pause",
            Operation::ResumeAccount => "Compte repris",
            Operation::DeleteAccount => "Compte supprimé",
            Operation::AddIdentity => "Identité ajoutée",
            Operation::UpdateIdentity => "Identité modifiée",
            Operation::SetDefaultIdentity => "Identité par défaut changée",
            Operation::CheckIntegrity => "Contrôle d'intégrité terminé",
            Operation::PurgeCache => "Cache de pièces jointes vidé",
            Operation::RebuildIndex => "Index de recherche reconstruit",
            Operation::ExportDiagnostic | Operation::ExportProfile => "Export écrit",
        }
    }

    /// Lecture à relancer après le succès de l'opération.
    pub fn follow_up(self) -> FollowUp {
        match self {
            Operation::UpdateAccount
            | Operation::PauseAccount
            | Operation::ResumeAccount
            | Operation::DeleteAccount => FollowUp::Accounts,
            Operation::AddIdentity | Operation::UpdateIdentity | Operation::SetDefaultIdentity => {
                FollowUp::Identities
            }
            Operation::PurgeCache | Operation::RebuildIndex => FollowUp::Stats,
            Operation::CheckIntegrity | Operation::ExportDiagnostic | Operation::ExportProfile => {
                FollowUp::Nothing
            }
        }
    }
}

impl SettingsState {
    /// Note l'opération et rend son jeton ; `None` si une autre est déjà en cours.
    pub fn begin_operation(&mut self, operation: Operation) -> Option<u64> {
        if self.tokens.operation.is_some() {
            return None;
        }
        let token = self.tokens.next();
        self.tokens.operation = Some((token, operation));
        Some(token)
    }

    /// Rend (et oublie) l'opération attendue sous ce jeton ; une réponse périmée ne rend rien.
    pub fn take_operation(&mut self, token: u64) -> Option<Operation> {
        match self.tokens.operation.take() {
            Some((pending, operation)) if pending == token => Some(operation),
            other => {
                self.tokens.operation = other;
                None
            }
        }
    }

    /// Vrai tant qu'une opération attend la réponse du moteur.
    pub fn operation_in_flight(&self) -> bool {
        self.tokens.operation.is_some()
    }
}

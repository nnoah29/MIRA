//! Erreur unique du contrat entre le moteur et ses clients (12.6, invariant 31).

use serde::{Deserialize, Serialize};

use super::action::SuggestedAction;
use super::category::ErrorCategory;
use super::code::ErrorCode;
use super::context::ErrorContext;

/// Erreur structurée : catégorie, code stable, message sans secret, rejouable ou non.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, thiserror::Error)]
#[error("{code}: {message}")]
pub struct MiraError {
    pub category: ErrorCategory,
    pub code: String,
    pub message: String,
    pub retryable: bool,
    pub suggested_action: Option<SuggestedAction>,
    pub context: Box<ErrorContext>,
}

impl MiraError {
    /// Crée une erreur ; le message ne doit jamais contenir de secret (responsabilité de l'appelant).
    pub fn new(category: ErrorCategory, code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            category,
            code: code.0.to_string(),
            message: message.into(),
            retryable: category == ErrorCategory::Network,
            suggested_action: None,
            context: Box::default(),
        }
    }

    /// Fixe si l'opération peut être rejouée telle quelle.
    pub fn retryable(mut self, retryable: bool) -> Self {
        self.retryable = retryable;
        self
    }

    /// Attache l'action que le client peut suggérer à l'utilisateur.
    pub fn with_action(mut self, action: SuggestedAction) -> Self {
        self.suggested_action = Some(action);
        self
    }

    /// Attache le contexte (compte, message, boîte, opération, action).
    pub fn with_context(mut self, context: ErrorContext) -> Self {
        self.context = Box::new(context);
        self
    }
}

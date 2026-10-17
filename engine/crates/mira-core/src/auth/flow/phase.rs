//! Phases du flux d'autorisation OAuth2 (machine à états explicite).

/// Étape d'un flux : attente de la redirection, code reçu, jetons obtenus, ou fin anormale.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FlowPhase {
    AwaitingRedirect,
    CodeReceived,
    TokensObtained,
    Cancelled,
    Failed,
}

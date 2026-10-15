//! Intentions de la vue « file » et de la décision sur un envoi incertain.

/// Ce que l'utilisateur fait de la file d'actions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QueueIntent {
    Open,
    Retry,
    Cancel,
    Resolve,
    Resend,
    DontResend,
}

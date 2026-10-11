//! Action que le client peut proposer à l'utilisateur après une erreur.

use serde::{Deserialize, Serialize};

/// Suggestion de remédiation attachée à une erreur.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SuggestedAction {
    Reauthenticate,
    AcceptCertificate,
    FreeDiskSpace,
    RetryLater,
    CheckServerSettings,
    UnlockKeyring,
    ChooseProvider,
    ReportBug,
}

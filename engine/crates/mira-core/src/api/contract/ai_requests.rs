//! Requêtes du domaine « assistant d'IA » : connexions, clés, modèles, consentement, confidentialité, journal.

use mira_vocabulary::{AiConnectionId, AiProviderId, ContentKind, MaskingSettings};
use serde::{Deserialize, Serialize};

use super::secret_input::SecretInput;

/// Opérations sur les connexions d'IA ; la clé arrive une seule fois et ne revient jamais.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum AiRequest {
    ListAiProviders,
    ListAiConnections,
    AddAiConnection {
        provider: AiProviderId,
        label: String,
        base_url: Option<String>,
    },
    UpdateAiConnection {
        connection: AiConnectionId,
        label: String,
        base_url: Option<String>,
        default_model: Option<String>,
    },
    RemoveAiConnection {
        connection: AiConnectionId,
    },
    SetDefaultAiConnection {
        connection: AiConnectionId,
    },
    SetAiApiKey {
        connection: AiConnectionId,
        key: SecretInput,
    },
    ClearAiApiKey {
        connection: AiConnectionId,
    },
    TestAiConnection {
        connection: AiConnectionId,
    },
    ListAiModels {
        connection: AiConnectionId,
        refresh: bool,
    },
    AcknowledgeAiConsent {
        connection: AiConnectionId,
        #[serde(default = "default_consent_kinds")]
        kinds: Vec<ContentKind>,
    },
    SetAiAllowedContent {
        connection: AiConnectionId,
        kinds: Vec<ContentKind>,
    },
    SetAiMasking {
        connection: AiConnectionId,
        masking: MaskingSettings,
    },
    ListAiJournal {
        limit: u32,
        before: Option<String>,
    },
    ClearAiJournal,
}

/// Types autorisés par un consentement sans liste : le mail ouvert, comme avant le contrat 1.2.
fn default_consent_kinds() -> Vec<ContentKind> {
    vec![ContentKind::MailMessage]
}

//! Vérifications locales d'une génération et plan d'exécution, avant tout accès au réseau.

use std::fmt;
use std::sync::Weak;

use mira_vocabulary::error::codes::not_found;
use mira_vocabulary::{
    AiConnection, AiConnectionId, AiProviderDescriptor, AiProviderId, ChatRequest, ContentKind,
    ErrorCategory, MaskingSettings, MiraError,
};

use super::checks::{check, context_summary};
use super::context::render_context;
use super::tools_plan::with_tools;
use super::truncate::{MAX_INPUT_CHARS, request_chars, truncate_request};
use crate::ai::masking::Masker;
use crate::api::{ChatStart, Engine};
use crate::ports::SecretValue;

/// Tout ce que la tâche de génération doit savoir ; la requête est déjà rendue et tronquée.
pub(crate) struct ChatPlan {
    pub(crate) descriptor: AiProviderDescriptor,
    pub(crate) base_url: String,
    pub(crate) key: SecretValue,
    pub(crate) model: String,
    pub(crate) request: ChatRequest,
    pub(crate) masker: Masker,
    pub(crate) connection: AiConnectionId,
    pub(crate) provider: AiProviderId,
    pub(crate) local: bool,
    pub(crate) kinds: Vec<ContentKind>,
    pub(crate) allowed_kinds: Vec<ContentKind>,
    pub(crate) host: Option<Weak<Engine>>,
    pub(crate) parts: u32,
    pub(crate) chars: u32,
}

/// Vérifie connexion, fournisseur, consentement, types de contenu, clé puis modèle, dans cet ordre, et rend le plan ; les outils ne sont annoncés que si le client les permet et que le service les supporte.
pub(crate) fn prepare(
    engine: &Engine,
    start: ChatStart,
    host: Option<Weak<Engine>>,
) -> Result<ChatPlan, MiraError> {
    let connection = load_connection(engine, start.connection.as_ref())?;
    let checked = check(engine, &connection, &start)?;
    let enabled = start.allow_tools && checked.descriptor.supports_tools && host.is_some();
    let (kinds, parts) = context_summary(&start.request);
    let request = with_tools(start.request, enabled);
    let (request, masker) = shape_request(request, &connection.masking);
    Ok(ChatPlan {
        descriptor: checked.descriptor,
        base_url: checked.base_url,
        key: checked.key,
        model: checked.model,
        local: checked.local,
        connection: connection.id,
        provider: connection.provider,
        kinds,
        allowed_kinds: connection.allowed_kinds,
        host: host.filter(|_| enabled),
        parts,
        chars: request_chars(&request),
        request,
        masker,
    })
}

/// Rend le contexte, masque les textes envoyés (pas la consigne), puis tronque ; rend aussi la table du masquage.
fn shape_request(request: ChatRequest, masking: &MaskingSettings) -> (ChatRequest, Masker) {
    let mut masker = Masker::new(masking);
    let mut request = render_context(request);
    masker.reserve(request.system.as_deref().unwrap_or_default());
    request
        .messages
        .iter()
        .for_each(|m| masker.reserve(&m.text));
    for message in &mut request.messages {
        message.text = masker.mask(&message.text);
    }
    (truncate_request(request, MAX_INPUT_CHARS), masker)
}

/// Relit la connexion désignée, ou la connexion par défaut ; absente, `not_found.ai_connection`.
fn load_connection(
    engine: &Engine,
    id: Option<&AiConnectionId>,
) -> Result<AiConnection, MiraError> {
    let found = engine.database.read(|tx| match id {
        Some(id) => tx.get_ai_connection(id),
        None => Ok(tx.list_ai_connections()?.into_iter().find(|c| c.is_default)),
    })?;
    found.ok_or_else(|| {
        MiraError::new(
            ErrorCategory::NotFound,
            not_found::AI_CONNECTION,
            "connexion d'IA inconnue",
        )
    })
}

impl fmt::Debug for ChatPlan {
    /// Montre le fournisseur et le modèle, jamais la clé, l'adresse ni le texte de la requête.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ChatPlan")
            .field("provider", &self.descriptor.id.0)
            .field("model", &self.model)
            .finish_non_exhaustive()
    }
}

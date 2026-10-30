//! Aiguillage par format de dialogue : chaque opération choisit son codec par `match` exhaustif.

use mira_vocabulary::{AiError, AiModel, AiProviderDescriptor, ChatDelta, ChatRequest, WireFormat};

use super::call::HttpCall;
use super::{anthropic_decode, anthropic_encode, anthropic_models};
use super::{openai_decode, openai_encode, openai_models};
use crate::ai::sse::SseEvent;

/// Encode la requête selon le format du descripteur.
pub fn encode(request: &ChatRequest, model: &str, descriptor: &AiProviderDescriptor) -> HttpCall {
    match descriptor.format {
        WireFormat::OpenAiChat => openai_encode::encode_chat(request, model, descriptor),
        WireFormat::AnthropicMessages => anthropic_encode::encode_chat(request, model, descriptor),
    }
}

/// Décode un événement SSE selon le format du service.
pub fn decode_event(format: WireFormat, event: &SseEvent) -> Result<Vec<ChatDelta>, AiError> {
    match format {
        WireFormat::OpenAiChat => openai_decode::decode_chat_event(event),
        WireFormat::AnthropicMessages => anthropic_decode::decode_chat_event(event),
    }
}

/// Décode la liste des modèles selon le format du service.
pub fn decode_models(format: WireFormat, body: &[u8]) -> Result<Vec<AiModel>, AiError> {
    match format {
        WireFormat::OpenAiChat => openai_models::decode_models(body),
        WireFormat::AnthropicMessages => anthropic_models::decode_models(body),
    }
}

/// Donne le chemin de discussion du format, relatif à l'adresse de base.
pub fn chat_path(format: WireFormat) -> &'static str {
    match format {
        WireFormat::OpenAiChat => openai_encode::CHAT_PATH,
        WireFormat::AnthropicMessages => anthropic_encode::CHAT_PATH,
    }
}

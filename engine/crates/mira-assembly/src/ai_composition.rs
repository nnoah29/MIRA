//! Construction du registre des fournisseurs d'IA : un seul endroit qui nomme les services.

use std::sync::Arc;

use mira_ai_anthropic::AnthropicAiProvider;
use mira_ai_generic::GenericAiProvider;
use mira_ai_google::GoogleAiProvider;
use mira_ai_groq::GroqAiProvider;
use mira_ai_mistral::MistralAiProvider;
use mira_ai_openai::OpenAiAiProvider;
use mira_core::ai::AiProviderRegistry;

/// Construit le registre : anthropic, openai, google, mistral, groq, puis le service compatible.
pub fn build_ai_registry() -> AiProviderRegistry {
    let mut registry = AiProviderRegistry::new();
    registry.register(Arc::new(AnthropicAiProvider));
    registry.register(Arc::new(OpenAiAiProvider));
    registry.register(Arc::new(GoogleAiProvider));
    registry.register(Arc::new(MistralAiProvider));
    registry.register(Arc::new(GroqAiProvider));
    registry.register(Arc::new(GenericAiProvider));
    registry
}

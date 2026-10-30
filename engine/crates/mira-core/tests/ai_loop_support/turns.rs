//! Tours joués, forme des deltas et lecture des résultats d'outils renvoyés au faux serveur.
#![allow(dead_code)]

use std::time::Duration;

use mira_core::api::ChatStream;
use mira_testkit::ai::{
    Behavior, ToolCallSpec, anthropic_stream, anthropic_tool_turn, openai_stream, openai_tool_turn,
};
use mira_vocabulary::{ChatDelta, StopReason, WireFormat};
use serde_json::Value;
use tokio::time::timeout;

pub const FORMATS: [WireFormat; 2] = [WireFormat::AnthropicMessages, WireFormat::OpenAiChat];

/// Tour du modèle qui demande ces appels (identifiant, outil, arguments) après ce texte.
pub fn tool_turn(
    format: WireFormat,
    text: &str,
    calls: &[ToolCallSpec<'_>],
    usage: Option<(u32, u32)>,
) -> Behavior {
    match format {
        WireFormat::AnthropicMessages => anthropic_tool_turn(text, calls, usage),
        WireFormat::OpenAiChat => openai_tool_turn(text, calls, usage),
    }
}

/// Tour final du modèle : du texte, sans outil.
pub fn text_turn(format: WireFormat, text: &str, usage: Option<(u32, u32)>) -> Behavior {
    match format {
        WireFormat::AnthropicMessages => anthropic_stream(&[text], usage),
        WireFormat::OpenAiChat => openai_stream(&[text], usage),
    }
}

/// Étiquette courte de chaque delta, pour comparer l'ordre des événements.
pub fn shape(deltas: &[ChatDelta]) -> Vec<String> {
    let label = |delta: &ChatDelta| match delta {
        ChatDelta::Text { .. } => "text".to_string(),
        ChatDelta::Usage { .. } => "usage".to_string(),
        ChatDelta::Masked { .. } => "masked".to_string(),
        ChatDelta::ToolFragment { .. } => "fragment".to_string(),
        ChatDelta::ToolCalled { name, .. } => format!("called:{name}"),
        ChatDelta::ToolFinished { state, .. } => format!("finished:{state:?}").to_lowercase(),
        ChatDelta::Done { reason } => format!("done:{}", reason_name(reason)),
    };
    deltas.iter().map(label).collect()
}

/// Nom court d'une raison d'arrêt.
fn reason_name(reason: &StopReason) -> &'static str {
    match reason {
        StopReason::EndTurn => "end_turn",
        StopReason::MaxTokens => "max_tokens",
        StopReason::ToolUse => "tool_use",
        StopReason::MaxToolRounds => "max_tool_rounds",
        StopReason::Cancelled => "cancelled",
        StopReason::Failed { .. } => "failed",
    }
}

/// Textes des résultats d'outils présents dans un corps de requête, dans l'ordre.
pub fn tool_results(body: &Value, format: WireFormat) -> Vec<String> {
    let messages = body["messages"].as_array().cloned().unwrap_or_default();
    let text = |value: &Value| value.as_str().unwrap_or_default().to_string();
    match format {
        WireFormat::OpenAiChat => messages
            .iter()
            .filter(|message| message["role"] == "tool")
            .map(|message| text(&message["content"]))
            .collect(),
        WireFormat::AnthropicMessages => messages
            .iter()
            .flat_map(|message| message["content"].as_array().cloned().unwrap_or_default())
            .filter(|block| block["type"] == "tool_result")
            .map(|block| text(&block["content"]))
            .collect(),
    }
}

/// Lit tous les deltas jusqu'à la fin du flux (10 s au plus).
pub async fn collect(stream: &mut ChatStream) -> Vec<ChatDelta> {
    let mut deltas = Vec::new();
    let read_all = async {
        while let Some(delta) = stream.next().await {
            deltas.push(delta);
        }
    };
    timeout(Duration::from_secs(10), read_all).await.unwrap();
    deltas
}

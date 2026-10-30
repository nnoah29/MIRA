//! Requête de discussion : messages, consigne, bornes et outils proposés au modèle.

use serde::{Deserialize, Serialize};

use super::content::ContextPart;
use super::tool::{ToolCall, ToolOutcome};

/// Auteur d'un message de la discussion.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChatRole {
    User,
    Assistant,
    Tool,
}

/// Un message de la discussion.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: ChatRole,
    pub text: String,
    #[serde(default)]
    pub tool_calls: Vec<ToolCall>,
    #[serde(default)]
    pub tool_result: Option<ToolOutcome>,
}

impl ChatMessage {
    /// Message de texte seul, sans appel ni résultat d'outil.
    pub fn plain(role: ChatRole, text: String) -> Self {
        Self {
            role,
            text,
            tool_calls: Vec::new(),
            tool_result: None,
        }
    }
}

/// Outil proposé au modèle : nom, description et schéma JSON des arguments.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ToolSpec {
    pub name: String,
    pub description: String,
    pub parameters_schema: String,
}

/// Demande de discussion ; le modèle absent se remplace par celui de la connexion, le contexte est rendu par le moteur.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChatRequest {
    pub model: Option<String>,
    pub system: Option<String>,
    pub messages: Vec<ChatMessage>,
    pub max_output_tokens: Option<u32>,
    pub temperature: Option<f32>,
    #[serde(default)]
    pub tools: Vec<ToolSpec>,
    #[serde(default)]
    pub context: Vec<ContextPart>,
}

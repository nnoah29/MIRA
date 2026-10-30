//! Faux serveur d'IA (formats OpenAI Chat et Anthropic Messages) sur 127.0.0.1.
//! Les comportements se mettent en file, les requêtes reçues sont enregistrées.

mod anthropic_body;
mod behavior;
mod chunked;
mod control;
mod listener;
mod openai_body;
mod oversize;
mod plain;
mod recorded;
mod request;
mod respond;
mod server;
mod state;
mod tool_turns_anthropic;
mod tool_turns_openai;

pub use anthropic_body::anthropic_stream;
pub use behavior::Behavior;
pub use openai_body::openai_stream;
pub use recorded::RecordedRequest;
pub use server::FakeAiServer;
pub use tool_turns_anthropic::{ToolCallSpec, anthropic_tool_turn};
pub use tool_turns_openai::openai_tool_turn;

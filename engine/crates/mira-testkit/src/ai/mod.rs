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

pub use anthropic_body::anthropic_stream;
pub use behavior::Behavior;
pub use openai_body::openai_stream;
pub use recorded::RecordedRequest;
pub use server::FakeAiServer;

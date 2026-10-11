//! Composition d'un message MIME sortant (10.8) avec `mail-builder`.

mod addresses;
mod build;
mod built;
mod input;
mod sanitize;

pub use build::build_message;
pub use built::BuiltMessage;
pub use input::{ComposeAttachment, ComposeInput};

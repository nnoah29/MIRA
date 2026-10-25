//! Modèles de messages : variables `{{…}}` et rendu pur, sans accès au stockage.

mod render;
mod variables;

pub use render::{render, render_template};
pub use variables::TemplateVariable;

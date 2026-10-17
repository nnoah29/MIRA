//! Brouillon en cours d'édition : champs, tampons d'édition, enregistrement et fermeture.

pub mod access;
pub mod addresses;
pub mod buffer_delete;
pub mod buffer_edit;
pub mod buffer_move;
pub mod buffer_vertical;
pub mod buffer_view;
pub mod char_offset;
pub mod field;
pub mod open;
pub mod quit;
pub mod recipients;
pub mod reopen;
pub mod save_apply;
pub mod save_plan;
pub mod sending;
pub mod slot;
pub mod state;
pub mod status;
pub mod text_buffer;
pub mod typing;
pub mod viewport;

pub use field::{FIELD_COUNT, Field, FieldKind};
pub use recipients::RecipientCheck;
pub use save_plan::SavePlan;
pub use sending::SendPhase;
pub use slot::ComposeSlot;
pub use state::{ComposeState, FieldBuffer};
pub use text_buffer::TextBuffer;

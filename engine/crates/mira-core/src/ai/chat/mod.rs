//! Génération en flux : vérifications locales, rendu du contexte, troncature, tâche de diffusion annulable.
//! Les deltas passent par un canal borné ; le journal ne reçoit que des comptes et des codes, sans contenu.

mod answer;
mod answer_tools;
mod context;
mod ending;
mod events;
mod flush;
mod framing;
mod guard;
mod journal;
mod kinds;
mod neutralize;
mod prepare;
mod progress;
mod run;
mod stream;
mod truncate;

pub use context::render_context;
pub use framing::{TOOLS_NOTICE, frame_result};
pub(crate) use truncate::TRUNCATION_MARK;
pub use truncate::{MAX_INPUT_CHARS, truncate_request};

pub(crate) use journal::{interrupt_running, open_row};
pub(crate) use prepare::{ChatPlan, prepare};
pub(crate) use run::{ChatTask, run};

/// Nombre de deltas en attente au plus entre la tâche et le lecteur.
pub(crate) const CHANNEL_CAPACITY: usize = 32;

/// Nature des opérations de génération dans le registre des opérations.
pub(crate) const CHAT_OPERATION_KIND: &str = "ai_chat";

//! Génération en flux : vérifications locales, troncature de l'entrée, tâche de diffusion annulable.
//! Les deltas passent par un canal borné ; le journal ne reçoit que le début et la fin, sans contenu.

mod ending;
mod events;
mod guard;
mod prepare;
mod progress;
mod run;
mod stream;
mod truncate;

pub use truncate::{MAX_INPUT_CHARS, truncate_request};

pub(crate) use prepare::prepare;
pub(crate) use run::{ChatTask, run};

/// Nombre de deltas en attente au plus entre la tâche et le lecteur.
pub(crate) const CHANNEL_CAPACITY: usize = 32;

/// Nature des opérations de génération dans le registre des opérations.
pub(crate) const CHAT_OPERATION_KIND: &str = "ai_chat";

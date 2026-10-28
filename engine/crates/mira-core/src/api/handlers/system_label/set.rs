//! Choix manuel : poser ou retirer un libellé système sur des conversations, tout ou rien.

use mira_vocabulary::{EventKind, MiraError, SystemLabelKind, ThreadId};

use crate::actions::errors::input::invalid_parameter;
use crate::api::contract::{Response, SystemLabelResponse};
use crate::api::engine::Engine;
use crate::events::EventSink;
use crate::system_labels::set_system_label;

/// Nombre maximal de conversations par appel
const MAX_THREADS: usize = 1000;

/// Pose (ou retire) le libellé ; une conversation inconnue refuse toute la pose ; la pose est un ensemble, donc rejouable
pub(crate) fn handle_set_system_label(
    engine: &Engine,
    kind: SystemLabelKind,
    threads: &[ThreadId],
    on: bool,
) -> Result<Response, MiraError> {
    if threads.len() > MAX_THREADS {
        return Err(invalid_parameter("trop de conversations en une fois"));
    }
    engine
        .database
        .write(|tx| set_system_label(tx, kind, threads, on))?;
    engine.events.emit(EventKind::SystemLabelsChanged)?;
    Ok(Response::SystemLabel(SystemLabelResponse::Done))
}

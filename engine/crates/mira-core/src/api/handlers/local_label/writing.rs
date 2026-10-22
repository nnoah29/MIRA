//! Annonce et réponses communes aux écritures de libellés locaux.

use mira_vocabulary::{EventKind, LocalLabel, MiraError};

use crate::api::contract::{LocalLabelResponse, Response};
use crate::api::engine::Engine;
use crate::events::EventSink;

/// Couleurs ARGB proposées tour à tour aux nouveaux libellés
const PALETTE: [u32; 6] = [
    0xFFE8_542B,
    0xFF6A_A8D9,
    0xFF8B_C48A,
    0xFFD9_B46A,
    0xFFC9_8BB9,
    0xFF9A_8BE0,
];

/// Couleur par défaut d'un nouveau libellé, selon le nombre de libellés déjà présents
pub(super) fn default_color(existing: usize) -> u32 {
    PALETTE[existing % PALETTE.len()]
}

/// Annonce que les libellés locaux ou leurs poses ont changé
pub(super) fn announce(engine: &Engine) -> Result<(), MiraError> {
    engine.events.emit(EventKind::LocalLabelsChanged)?;
    Ok(())
}

/// Réponse portant un seul libellé
pub(super) fn single(label: LocalLabel) -> Response {
    Response::LocalLabel(LocalLabelResponse::Label(label))
}

/// Réponse sans contenu
pub(super) fn done() -> Response {
    Response::LocalLabel(LocalLabelResponse::Done)
}

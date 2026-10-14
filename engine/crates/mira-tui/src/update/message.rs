//! Messages que la boucle fournit à `update` et leur routage.

use mira_assembly::api::Response;
use mira_assembly::mira_vocabulary::{Event, MiraError};

use super::intent::{Intent, on_intent};
use crate::commands::{Command, Tag};
use crate::state::AppState;

/// Réponse du moteur à une requête étiquetée.
#[derive(Debug)]
pub struct Reply {
    pub tag: Tag,
    pub result: Result<Response, MiraError>,
}

/// Tout ce qui peut faire évoluer l'état.
#[derive(Debug)]
#[allow(clippy::large_enum_variant)]
pub enum Msg {
    Intent(Intent),
    Reply(Reply),
    Event(Event),
    Resize(u16, u16),
}

/// Applique un message à l'état et rend les effets à réaliser.
pub fn update(state: &mut AppState, msg: Msg) -> Vec<Command> {
    match msg {
        Msg::Intent(intent) => on_intent(state, intent),
        Msg::Reply(reply) => on_reply(state, reply),
        Msg::Event(event) => on_event(state, event),
        Msg::Resize(width, height) => on_resize(state, width, height),
    }
}

/// Enregistre la nouvelle taille de l'écran.
fn on_resize(state: &mut AppState, width: u16, height: u16) -> Vec<Command> {
    state.width = width;
    state.height = height;
    Vec::new()
}

/// Traite la réponse du moteur selon son étiquette.
fn on_reply(_state: &mut AppState, reply: Reply) -> Vec<Command> {
    match reply.tag {
        Tag::Startup => Vec::new(),
    }
}

/// Traite un événement du moteur.
fn on_event(_state: &mut AppState, _event: Event) -> Vec<Command> {
    Vec::new()
}

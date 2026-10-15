//! Routage des intentions de la palette : hors saisie seule l'ouverture agit ; saisie ouverte, elle reçoit tout.

use super::intent::PaletteIntent;
use super::parse::parse_command;
use crate::commands::Command;
use crate::state::AppState;
use crate::update::Intent;
use crate::update::intent::on_intent;
use crate::update::panes;
use crate::update::settings::open::close_settings;
use crate::view::safe_text::display_safe;

/// Intention de palette reçue hors saisie : seule l'ouverture agit.
pub fn route_idle(state: &mut AppState, intent: PaletteIntent) -> Vec<Command> {
    if intent == PaletteIntent::Open {
        state.palette.open();
    }
    Vec::new()
}

/// Saisie ouverte : frappe, validation, fermeture et sortie ; les autres intentions sont ignorées.
#[rustfmt::skip]
pub fn route_open(state: &mut AppState, intent: Intent) -> Vec<Command> {
    match intent {
        Intent::Palette(palette) => type_into_palette(state, palette),
        Intent::Open | Intent::Confirm => submit(state),
        Intent::Back | Intent::Cancel => close(state),
        Intent::Quit => panes::quit(state),
        Intent::MoveDown | Intent::MoveUp | Intent::Top | Intent::Bottom | Intent::NextPane | Intent::PrevPane
        | Intent::NextAccount | Intent::PrevAccount | Intent::Archive | Intent::Trash | Intent::DeletePermanently
        | Intent::ToggleFlag | Intent::ToggleRead | Intent::OpenMovePicker | Intent::Undo | Intent::StartSync
        | Intent::Help | Intent::Search(_) | Intent::Compose(_) | Intent::Queue(_) | Intent::Drafts(_)
        | Intent::Settings(_) => Vec::new(),
    }
}

/// Applique une intention de palette à la saisie ouverte.
fn type_into_palette(state: &mut AppState, intent: PaletteIntent) -> Vec<Command> {
    match intent {
        PaletteIntent::Char(c) => state.palette.push(c),
        PaletteIntent::Backspace => state.palette.pop(),
        PaletteIntent::Submit => return submit(state),
        PaletteIntent::Close => return close(state),
        PaletteIntent::Open => {}
    }
    Vec::new()
}

/// Ferme la saisie sans rien exécuter.
fn close(state: &mut AppState) -> Vec<Command> {
    state.palette.close();
    Vec::new()
}

/// Exécute la commande tapée ; un mot inconnu est refusé dans la barre d'état et la saisie se ferme.
fn submit(state: &mut AppState) -> Vec<Command> {
    let text = state.palette.input().trim().to_string();
    state.palette.close();
    if text.is_empty() {
        return Vec::new();
    }
    match parse_command(&text) {
        Some(intent) => {
            close_settings(state);
            on_intent(state, intent)
        }
        None => {
            let shown = display_safe(&text);
            state.status.message = Some(format!("Commande inconnue : {shown} (essayez :help)"));
            Vec::new()
        }
    }
}

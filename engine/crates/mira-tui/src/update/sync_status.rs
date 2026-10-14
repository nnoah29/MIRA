//! Demande de synchronisation et lecture de l'état affiché par la barre d'état.

use mira_assembly::api::{EngineRequest, Request, SyncRequest, SyncScope};

use crate::commands::{Command, Tag};
use crate::state::AppState;

/// Demande la synchronisation de tous les comptes.
pub fn start_sync(_state: &mut AppState) -> Vec<Command> {
    vec![Command::Execute {
        tag: Tag::StartSync,
        request: Request::Sync(SyncRequest::StartSync {
            scope: SyncScope::All,
        }),
    }]
}

/// Relit la file d'actions et l'état de synchro, sauf pour ce qui est déjà demandé.
pub fn refresh_status(state: &mut AppState) -> Vec<Command> {
    let mut commands = Vec::new();
    if !state.refresh.status {
        state.refresh.status = true;
        commands.push(Command::Execute {
            tag: Tag::Status,
            request: Request::Engine(EngineRequest::GetStatus),
        });
    }
    if !state.refresh.sync {
        state.refresh.sync = true;
        commands.push(Command::Execute {
            tag: Tag::SyncStatus,
            request: Request::Sync(SyncRequest::GetSyncStatus),
        });
    }
    commands
}

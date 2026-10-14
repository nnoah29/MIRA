//! Ouverture du message sélectionné et nouvelle tentative de téléchargement du corps.

use mira_assembly::api::{MessageRequest, Request, ThreadRequest};
use mira_assembly::mira_vocabulary::{MessageSummary, ThreadId};

use super::mark_read::mark_read_command;
use crate::commands::{Command, Tag};
use crate::state::{AppState, ReaderContent};

/// Commande de lecture sur un message, étiquetée pour sa réponse.
fn message_command(tag: Tag, request: MessageRequest) -> Command {
    Command::Execute {
        tag,
        request: Request::Message(request),
    }
}

/// Commande de lecture du fil d'un message.
fn thread_command(thread: &ThreadId) -> Command {
    Command::Execute {
        tag: Tag::Thread(thread.clone()),
        request: Request::Thread(ThreadRequest::GetThread {
            thread: thread.clone(),
        }),
    }
}

/// Commandes qui complètent le lecteur : pièces jointes, fil, marquage lu.
fn companions(state: &mut AppState, message: &MessageSummary) -> Vec<Command> {
    let id = &message.id;
    let mut commands = Vec::new();
    if message.has_attachments {
        let request = MessageRequest::ListAttachments {
            message: id.clone(),
        };
        commands.push(message_command(Tag::Attachments(id.clone()), request));
    }
    commands.extend(message.thread.as_ref().map(thread_command));
    if !message.flags.seen {
        state.messages.mark_seen(id);
        commands.push(mark_read_command(state, id));
    }
    commands
}

/// Ouvre le message sélectionné dans le lecteur et demande son corps local et ses compléments.
pub fn open_selected_message(state: &mut AppState) -> Vec<Command> {
    let selected = state.messages.selection.selected;
    let Some(message) = selected.and_then(|index| state.messages.items().get(index).cloned())
    else {
        return Vec::new();
    };
    state.reader.open(message.clone());
    let request = MessageRequest::GetMessageBody {
        message: message.id.clone(),
    };
    let mut commands = vec![message_command(Tag::Body(message.id.clone()), request)];
    commands.extend(companions(state, &message));
    commands
}

/// Redemande le corps au serveur quand il manque hors ligne ou que la lecture a échoué.
pub fn retry_body_download(state: &mut AppState) -> Vec<Command> {
    let retryable = matches!(
        state.reader.content,
        ReaderContent::OfflineMissing | ReaderContent::Failed(_)
    );
    let Some(id) = state
        .reader
        .message()
        .map(|m| m.id.clone())
        .filter(|_| retryable)
    else {
        return Vec::new();
    };
    state.reader.content = ReaderContent::Loading;
    let request = MessageRequest::FetchMessageBody {
        message: id.clone(),
    };
    vec![message_command(Tag::Body(id), request)]
}

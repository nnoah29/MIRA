//! Données de test du lecteur : états, messages, corps, erreurs, pièces jointes, fils.
#![allow(dead_code)]

use crate::message_fixtures::{inbox_view, summary};
use mira_assembly::api::contract::ThreadView;
use mira_assembly::mira_vocabulary::error::codes::incompatible_state::BODY_UNAVAILABLE_OFFLINE;
use mira_assembly::mira_vocabulary::{
    AccountId, Attachment, AttachmentId, CacheState, ErrorCategory, MessageId, MessageSummary,
    MiraError, Page, Thread, ThreadId,
};
use mira_assembly::reading::BodyView;
use mira_tui::state::{AppState, Pane};

/// État sur la liste avec ces messages, le premier sélectionné.
pub fn list_state(items: Vec<MessageSummary>) -> AppState {
    let mut state = AppState::new(100, 30);
    state
        .accounts
        .replace_items(vec![crate::sidebar_fixtures::account("a1")]);
    let page = Page {
        items,
        next_cursor: None,
    };
    state.messages.replace_page(inbox_view(), page);
    state.focus = Pane::List;
    state
}

/// Message non lu `m1` d'objet « Bonjour ».
pub fn unread_message() -> MessageSummary {
    summary("m1", "Bonjour", "Bob")
}

/// Corps texte sans version HTML.
pub fn body(text: &str) -> BodyView {
    BodyView {
        text: text.to_string(),
        html_available: false,
    }
}

/// Erreur « corps indisponible hors ligne ».
pub fn offline_error() -> MiraError {
    MiraError::new(
        ErrorCategory::IncompatibleState,
        BODY_UNAVAILABLE_OFFLINE,
        "corps absent",
    )
}

/// Pièce jointe de test.
pub fn attachment(id: &str, message: &str, filename: &str) -> Attachment {
    Attachment {
        id: AttachmentId::from_raw(id),
        message: MessageId::from_raw(message),
        filename: Some(filename.to_string()),
        mime_type: "application/pdf".to_string(),
        size: 2048,
        content_id: None,
        inline: false,
        cache: CacheState::NotDownloaded,
    }
}

/// Fil de test contenant ces messages.
pub fn thread_view(id: &str, messages: Vec<MessageSummary>) -> ThreadView {
    ThreadView {
        thread: Thread {
            id: ThreadId::from_raw(id),
            account: AccountId::from_raw("a1"),
            subject: "Sujet".to_string(),
            message_count: 2,
            unread_count: 0,
            last_date: None,
            participants: Vec::new(),
        },
        messages,
    }
}

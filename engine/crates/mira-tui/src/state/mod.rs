//! État complet de l'application : données affichées, focus et taille d'écran.

pub mod accounts;
pub mod actions;
pub mod boot;
pub mod compose;
pub mod focus;
pub mod keys;
pub mod mailbox_order;
pub mod mailboxes;
pub mod messages;
pub mod messages_edit;
pub mod messages_load;
pub mod messages_paging;
pub mod messages_refresh;
pub mod move_picker;
pub mod popup;
pub mod queue;
pub mod reader;
pub mod refresh;
pub mod search;
pub mod search_edit;
pub mod search_restore;
pub mod selection;
pub mod sent_subjects;
pub mod status;
pub mod view_change;

use mira_assembly::mira_vocabulary::IdempotencyKey;

pub use accounts::AccountsState;
pub use actions::ActionsState;
pub use compose::ComposeSlot;
pub use focus::Pane;
pub use mailboxes::MailboxesState;
pub use messages::MessagesState;
pub use move_picker::MovePicker;
pub use popup::Popup;
pub use queue::QueueState;
pub use reader::{ReaderContent, ReaderState};
pub use refresh::RefreshState;
pub use search::SearchState;
pub use selection::Selection;
pub use sent_subjects::SentSubjects;
pub use status::StatusState;

/// Source unique de vérité de l'interface.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppState {
    pub width: u16,
    pub height: u16,
    pub focus: Pane,
    pub should_quit: bool,
    pub accounts: AccountsState,
    pub mailboxes: MailboxesState,
    pub messages: MessagesState,
    pub reader: ReaderState,
    pub status: StatusState,
    pub popup: Popup,
    pub actions: ActionsState,
    pub search: SearchState,
    pub compose: ComposeSlot,
    pub queue: QueueState,
    pub sent_subjects: SentSubjects,
    pub cancelled_drafts: Vec<IdempotencyKey>,
    pub refresh: RefreshState,
    pub key_prefix: String,
    pub key_counter: u64,
}

impl AppState {
    /// État initial pour un terminal de la taille donnée ; le nonce distingue les clés de cette session.
    pub fn new(width: u16, height: u16, nonce: u64) -> AppState {
        AppState {
            width,
            height,
            focus: Pane::Sidebar,
            should_quit: false,
            accounts: AccountsState::default(),
            mailboxes: MailboxesState::default(),
            messages: MessagesState::default(),
            reader: ReaderState::default(),
            status: StatusState::default(),
            popup: Popup::Closed,
            actions: ActionsState::default(),
            search: SearchState::default(),
            compose: ComposeSlot::default(),
            queue: QueueState::default(),
            sent_subjects: SentSubjects::default(),
            cancelled_drafts: Vec::new(),
            refresh: RefreshState::default(),
            key_prefix: format!("tui-{nonce}"),
            key_counter: 0,
        }
    }
}

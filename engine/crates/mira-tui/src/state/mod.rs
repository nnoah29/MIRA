//! État complet de l'application : données affichées, focus et taille d'écran.

pub mod accounts;
pub mod boot;
pub mod focus;
pub mod mailbox_order;
pub mod mailboxes;
pub mod messages;
pub mod messages_load;
pub mod messages_paging;
pub mod popup;
pub mod reader;
pub mod selection;
pub mod status;

pub use accounts::AccountsState;
pub use focus::Pane;
pub use mailboxes::MailboxesState;
pub use messages::MessagesState;
pub use popup::Popup;
pub use reader::ReaderState;
pub use selection::Selection;
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
}

impl AppState {
    /// État initial pour un terminal de la taille donnée, focus sur la barre latérale.
    pub fn new(width: u16, height: u16) -> AppState {
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
        }
    }
}

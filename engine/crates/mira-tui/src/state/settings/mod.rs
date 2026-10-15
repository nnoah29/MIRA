//! Vue Paramètres : comptes, identités, maintenance et journal d'événements, avec ses fenêtres.

pub mod dialog;
pub mod events;
pub mod events_page;
pub mod events_request;
pub mod form;
pub mod identities;
pub mod maintenance;
pub mod operation;
pub mod tab;
pub mod tokens;

use mira_assembly::mira_vocabulary::AccountId;

pub use dialog::{Ask, Dialog, ExportKind, Field, Form, FormKind};
pub use events::{EVENTS_PAGE, EventsState, PageMove, PendingPage};
pub use identities::IdentitiesState;
pub use maintenance::MaintenanceState;
pub use operation::{FollowUp, Operation};
pub use tab::SettingsTab;
use tokens::Tokens;

/// Vue Paramètres ; fermée par défaut. Les jetons ne reviennent jamais en arrière.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SettingsState {
    open: bool,
    pub tab: SettingsTab,
    pub account: Option<AccountId>,
    pub identities: IdentitiesState,
    pub maintenance: MaintenanceState,
    pub events: EventsState,
    pub dialog: Dialog,
    tokens: Tokens,
}

impl SettingsState {
    /// Ouvre la vue, vide, sur cet onglet et ce compte ; l'opération en cours est gardée.
    pub fn open(&mut self, tab: SettingsTab, account: Option<AccountId>) {
        self.reset();
        self.open = true;
        self.tab = tab;
        self.account = account;
    }

    /// Ferme la vue ; les lectures encore attendues seront ignorées.
    pub fn close(&mut self) {
        self.reset();
    }

    /// Vrai si la vue est ouverte.
    pub fn is_open(&self) -> bool {
        self.open
    }

    /// Change d'onglet et ferme la fenêtre ouverte.
    pub fn set_tab(&mut self, tab: SettingsTab) {
        self.tab = tab;
        self.dialog = Dialog::None;
    }

    /// Remet tout à zéro sauf les jetons ; l'opération en cours reste attendue.
    fn reset(&mut self) {
        let mut tokens = std::mem::take(&mut self.tokens);
        tokens.clear_reads();
        *self = SettingsState::default();
        self.tokens = tokens;
    }
}

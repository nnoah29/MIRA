//! Fabriques de comptes, de boîtes et de rendus partagées par les tests de la barre latérale.
#![allow(dead_code)]

use mira_assembly::api::{MailboxRequest, Request, Response};
use mira_assembly::mira_vocabulary::{
    Account, AccountId, AccountState, AuthMethodKind, EmailAddress, Mailbox, MailboxId,
    MailboxRole, ProfileId, ProviderId, SyncSettings, Timestamp,
};
use mira_tui::commands::{Command, Tag};
use mira_tui::state::AppState;
use mira_tui::update::{Msg, Reply, update};
use mira_tui::view::render;
use ratatui::Terminal;
use ratatui::backend::TestBackend;

/// Compte de test d'identifiant `id` et d'adresse `<id>@example.org`.
pub fn account(id: &str) -> Account {
    Account {
        id: AccountId::from_raw(id),
        profile: ProfileId::from_raw("profile"),
        address: EmailAddress::parse(&format!("{id}@example.org")).unwrap(),
        display_name: id.to_string(),
        provider: ProviderId::new("generic"),
        auth: AuthMethodKind::Password,
        servers: None,
        sync: SyncSettings::default(),
        state: AccountState::Connected,
        created_at: Timestamp::from_millis(0),
    }
}

/// Boîte de test du compte `account` avec ce rôle et ce nombre de non lus.
pub fn mailbox(account: &str, name: &str, role: MailboxRole, unread: u64) -> Mailbox {
    Mailbox {
        id: MailboxId::from_raw(format!("{account}-{name}")),
        account: AccountId::from_raw(account),
        name: name.to_string(),
        display_name: name.to_string(),
        delimiter: Some("/".to_string()),
        parent: None,
        role,
        sync_enabled: true,
        marker: None,
        total: unread,
        unread,
    }
}

/// Rend l'état dans un terminal de test 100x30 et rend tout le texte affiché, ligne par ligne.
pub fn rendered_text(state: &AppState) -> String {
    let mut terminal = Terminal::new(TestBackend::new(100, 30)).unwrap();
    terminal.draw(|frame| render(frame, state)).unwrap();
    let buffer = terminal.backend().buffer().clone();
    let width = buffer.area.width as usize;
    let symbols: Vec<&str> = buffer.content().iter().map(|cell| cell.symbol()).collect();
    symbols
        .chunks(width)
        .map(|row| row.concat())
        .collect::<Vec<_>>()
        .join("\n")
}

/// État avec les comptes a1 et a2, a1 sélectionné et sa boîte chargée.
pub fn loaded_state() -> AppState {
    let mut state = AppState::new(100, 30);
    state
        .accounts
        .replace_items(vec![account("a1"), account("a2")]);
    state
        .mailboxes
        .replace_items(vec![mailbox("a1", "INBOX", MailboxRole::Inbox, 2)]);
    state
}

/// Applique une réponse de liste de comptes.
pub fn refresh(state: &mut AppState, ids: &[&str]) -> Vec<Command> {
    let accounts = ids.iter().map(|id| account(id)).collect();
    let reply = Reply {
        tag: Tag::Accounts,
        result: Ok(Response::Accounts(accounts)),
    };
    update(state, Msg::Reply(reply))
}

/// Commande attendue pour charger les boîtes d'un compte.
pub fn load_for(id: &str) -> Command {
    let account = AccountId::from_raw(id);
    Command::Execute {
        tag: Tag::Mailboxes(account.clone()),
        request: Request::Mailbox(MailboxRequest::ListMailboxes { account }),
    }
}

//! Vue « Brouillons » : ouverture, réouverture d'un brouillon enregistré, sélection ancrée, brouillons non modifiables.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod action_fixtures;
mod compose_fixtures;
mod drafts_fixtures;
mod input_fixtures;
mod message_fixtures;
mod navigation_fixtures;
mod reader_fixtures;
mod sidebar_fixtures;

use action_fixtures::state_with_mailboxes;
use compose_fixtures::{draft, focus_on, subject_only, type_in_draft, update_command};
use drafts_fixtures::{
    drafts_reply, get_command, list_command, loaded_reply, named, only_drafts, open_drafts_with,
    reopen, saved_draft, scheduled, sending,
};
use input_fixtures::{key, map};
use mira_assembly::mira_vocabulary::{AccountId, ActionId, Event, EventKind, EventSeq, Timestamp};
use mira_tui::commands::Command;
use mira_tui::state::compose::Field;
use mira_tui::update::{ComposeIntent, DraftsIntent, Intent, Msg, update};
use navigation_fixtures::press;

/// `B` (liste) ouvre la vue sur le compte sélectionné et lit ses brouillons.
#[test]
fn b_opens_drafts_view_for_selected_account() {
    let mut state = state_with_mailboxes();
    assert_eq!(
        map(&state, key('B')),
        Some(Intent::Drafts(DraftsIntent::Open))
    );
    let commands = press(&mut state, Intent::Drafts(DraftsIntent::Open));
    assert_eq!(commands, vec![list_command("a1", 1)]);
    assert!(state.drafts.is_open());
    assert_eq!(state.drafts.account(), Some(&AccountId::from_raw("a1")));
    update(&mut state, drafts_reply(1, named(&["A", "B"])));
    assert_eq!(state.drafts.items().len(), 2);
    assert_eq!(state.drafts.selected().unwrap().id.as_str(), "A");
}

/// Entrée relit le brouillon puis l'ouvre dans l'éditeur, déjà enregistré ; le fermer ne supprime rien.
#[test]
fn enter_reopens_draft_in_editor_without_delete_on_close() {
    let mut state = state_with_mailboxes();
    open_drafts_with(&mut state, vec![saved_draft()]);
    assert_eq!(press(&mut state, Intent::Open), vec![get_command(2, "d1")]);
    update(&mut state, loaded_reply(2, Ok(saved_draft())));
    let compose = state.compose.open().expect("éditeur ouvert");
    assert_eq!(compose.draft().as_str(), "d1");
    assert!(compose.saved_once());
    assert!(!compose.has_unsaved());
    assert_eq!(compose.send_key, None);
    assert_eq!(compose.text(Field::Subject), "Re: Bonjour");
    assert!(!state.drafts.is_open());
    let commands = press(&mut state, Intent::Compose(ComposeIntent::Close));
    assert!(commands.is_empty(), "{commands:?}");
    assert!(state.compose.is_closed());
}

/// Rouvert puis fermé ou quitté sans modification : aucune commande (ni suppression ni enregistrement).
#[test]
fn reopened_draft_unmodified_close_sends_no_command() {
    let mut state = reopen(saved_draft());
    assert!(press(&mut state, Intent::Compose(ComposeIntent::Close)).is_empty());
    let mut state = reopen(saved_draft());
    assert_eq!(press(&mut state, Intent::Quit), vec![Command::Quit]);
    let mut state = reopen(saved_draft());
    assert!(mira_tui::update::shutdown::shutdown_commands(&mut state).is_empty());
}

/// Rouvert puis modifié : Ctrl+s n'envoie que le champ changé, au même brouillon.
#[test]
fn reopened_draft_edit_save_sends_only_changed_fields() {
    let mut state = reopen(saved_draft());
    focus_on(&mut state, Field::Subject);
    type_in_draft(&mut state, "!");
    let commands = press(&mut state, Intent::Compose(ComposeIntent::Save));
    assert_eq!(
        commands,
        vec![update_command("d1", subject_only("Re: Bonjour!"))]
    );
}

/// Une relecture retire une entrée placée avant la sélection : le même brouillon reste visé par Entrée.
#[test]
fn selection_anchored_on_draft_id_after_refresh() {
    let mut state = state_with_mailboxes();
    open_drafts_with(&mut state, named(&["A", "B", "C"]));
    press(&mut state, Intent::MoveDown);
    let event = Msg::Event(Event {
        seq: EventSeq(2),
        at: Timestamp::from_millis(0),
        kind: EventKind::ActionApplied {
            account: AccountId::from_raw("a1"),
            action: ActionId::from_raw("x"),
        },
    });
    assert_eq!(
        only_drafts(update(&mut state, event)),
        vec![list_command("a1", 2)]
    );
    update(&mut state, drafts_reply(2, named(&["B", "C"])));
    assert_eq!(state.drafts.selected().unwrap().id.as_str(), "B");
    assert_eq!(press(&mut state, Intent::Open), vec![get_command(3, "B")]);
}

/// Un brouillon planifié ou en cours d'envoi n'est pas rouvert : message clair, la vue reste ouverte.
#[test]
fn not_editable_draft_shows_message_and_stays_in_view() {
    let mut state = state_with_mailboxes();
    let drafts = vec![
        scheduled(draft("p1", &[], "Planifié", "")),
        sending(draft("s1", &[], "Parti", "")),
        draft("e1", &[], "Libre", ""),
    ];
    open_drafts_with(&mut state, drafts);
    assert!(press(&mut state, Intent::Open).is_empty());
    let shown = state.status.message.clone().unwrap();
    assert!(shown.contains("planifié"), "{shown}");
    assert!(state.drafts.is_open() && state.compose.is_closed());
    press(&mut state, Intent::MoveDown);
    assert!(press(&mut state, Intent::Open).is_empty());
    let shown = state.status.message.clone().unwrap();
    assert!(shown.contains("en cours d'envoi"), "{shown}");
    press(&mut state, Intent::MoveDown);
    assert_eq!(press(&mut state, Intent::Open), vec![get_command(2, "e1")]);
    let now_scheduled = scheduled(draft("e1", &[], "Libre", ""));
    update(&mut state, loaded_reply(2, Ok(now_scheduled)));
    assert!(state.compose.is_closed() && state.drafts.is_open());
    assert!(state.status.message.clone().unwrap().contains("planifié"));
}

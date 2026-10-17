//! Vue « Brouillons » : réponses tardives ignorées, `B` sans effet hors liste/lecteur, abandon qui garde le brouillon.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod action_fixtures;
mod compose_fixtures;
mod drafts_fixtures;
mod input_fixtures;
mod message_fixtures;
mod navigation_fixtures;
mod queue_fixtures;
mod reader_fixtures;
mod sidebar_fixtures;

use action_fixtures::state_with_mailboxes;
use compose_fixtures::{focus_on, open_reply, type_in_draft};
use drafts_fixtures::{
    drafts_reply, list_command, loaded_reply, named, only_drafts, reopen, saved_draft,
};
use mira_assembly::mira_vocabulary::ActionState;
use mira_tui::commands::Command;
use mira_tui::state::compose::Field;
use mira_tui::update::{ComposeIntent, DraftsIntent, Intent, QueueIntent, SearchIntent, update};
use navigation_fixtures::press;
use queue_fixtures::{change_reply, open_queue_with, outbox_entry};

/// La lecture d'avant un changement de compte, ou d'un tour périmé, est ignorée.
#[test]
fn late_list_reply_after_account_switch_ignored() {
    let mut state = state_with_mailboxes();
    press(&mut state, Intent::Drafts(DraftsIntent::Open));
    sidebar_fixtures::refresh(&mut state, &["a2"]);
    assert!(!state.drafts.is_open());
    update(&mut state, drafts_reply(1, named(&["A"])));
    assert!(state.drafts.items().is_empty());
    let commands = press(&mut state, Intent::Drafts(DraftsIntent::Open));
    assert_eq!(commands, vec![list_command("a2", 2)]);
    update(&mut state, drafts_reply(1, named(&["A"])));
    assert!(state.drafts.items().is_empty());
    update(&mut state, drafts_reply(2, named(&["Z"])));
    assert_eq!(state.drafts.items().len(), 1);
}

/// La relecture d'un brouillon arrivée vue fermée n'ouvre pas l'éditeur.
#[test]
fn late_load_reply_after_view_closed_ignored() {
    let mut state = state_with_mailboxes();
    drafts_fixtures::open_drafts_with(&mut state, vec![saved_draft()]);
    press(&mut state, Intent::Open);
    press(&mut state, Intent::Back);
    assert!(!state.drafts.is_open());
    assert!(update(&mut state, loaded_reply(2, Ok(saved_draft()))).is_empty());
    assert!(state.compose.is_closed());
}

/// `B` ne fait rien pendant l'édition, la création, une fenêtre ou la saisie de recherche.
#[test]
fn b_does_nothing_outside_list_and_reader() {
    let open = Intent::Drafts(DraftsIntent::Open);
    let mut state = open_reply();
    assert!(press(&mut state, open).is_empty());
    assert!(!state.drafts.is_open() && state.compose.open().is_some());
    let mut state = state_with_mailboxes();
    press(&mut state, Intent::Compose(ComposeIntent::Reply));
    assert!(press(&mut state, open).is_empty() && !state.drafts.is_open());
    let mut state = state_with_mailboxes();
    press(&mut state, Intent::Help);
    assert!(press(&mut state, open).is_empty() && !state.drafts.is_open());
    let mut state = state_with_mailboxes();
    press(&mut state, Intent::Search(SearchIntent::Open));
    assert!(press(&mut state, open).is_empty() && !state.drafts.is_open());
    let mut state = state_with_mailboxes();
    press(&mut state, Intent::Queue(QueueIntent::Open));
    assert!(press(&mut state, open).is_empty() && !state.drafts.is_open());
}

/// Abandonner les modifications d'un brouillon rouvert (Échap, ou Ctrl+c) ne supprime jamais le brouillon du moteur.
#[test]
fn reopened_draft_abandon_keeps_engine_draft() {
    for leave in [Intent::Compose(ComposeIntent::Close), Intent::Quit] {
        let mut state = reopen(saved_draft());
        focus_on(&mut state, Field::Subject);
        type_in_draft(&mut state, "!");
        assert!(press(&mut state, leave).is_empty());
        press(&mut state, Intent::Compose(ComposeIntent::AskDiscard));
        let commands = press(&mut state, Intent::Compose(ComposeIntent::Discard));
        let quits = leave == Intent::Quit;
        let expected = if quits {
            vec![Command::Quit]
        } else {
            Vec::new()
        };
        assert_eq!(commands, expected, "{leave:?}");
        assert!(state.compose.is_closed());
    }
}

/// Les messages d'envoi abandonné ou annulé disent comment rouvrir le brouillon.
#[test]
fn abandon_message_points_to_drafts_view() {
    let mut state = state_with_mailboxes();
    let doubtful = outbox_entry("s1", ActionState::Uncertain, "Peut-être", None);
    open_queue_with(&mut state, vec![doubtful], Vec::new());
    press(&mut state, Intent::Queue(QueueIntent::Resolve));
    press(&mut state, Intent::Queue(QueueIntent::DontResend));
    let shown = state.status.message.clone().unwrap();
    assert!(shown.contains("(rouvrez-le avec B)"), "{shown}");
    let mut state = state_with_mailboxes();
    let waiting = outbox_entry("s2", ActionState::Pending, "Attente", None);
    open_queue_with(&mut state, vec![waiting], Vec::new());
    press(&mut state, Intent::Queue(QueueIntent::Cancel));
    update(&mut state, change_reply("s2", Ok(())));
    let shown = state.status.message.clone().unwrap();
    assert!(
        shown.contains("annulé") && shown.contains("(rouvrez-le avec B)"),
        "{shown}"
    );
}

/// Les événements du compte relisent la vue sans la vider ni empiler les lectures ; ceux d'un autre compte non.
#[test]
fn drafts_events_refresh_without_bursts() {
    let mut state = state_with_mailboxes();
    drafts_fixtures::open_drafts_with(&mut state, named(&["A"]));
    let event = |account: &str| {
        mira_tui::update::Msg::Event(mira_assembly::mira_vocabulary::Event {
            seq: mira_assembly::mira_vocabulary::EventSeq(5),
            at: mira_assembly::mira_vocabulary::Timestamp::from_millis(0),
            kind: mira_assembly::mira_vocabulary::EventKind::SendDeparted {
                account: mira_assembly::mira_vocabulary::AccountId::from_raw(account),
                action: mira_assembly::mira_vocabulary::ActionId::from_raw("s"),
            },
        })
    };
    assert!(only_drafts(update(&mut state, event("a2"))).is_empty());
    assert_eq!(
        only_drafts(update(&mut state, event("a1"))),
        vec![list_command("a1", 2)]
    );
    assert!(only_drafts(update(&mut state, event("a1"))).is_empty());
    assert_eq!(state.drafts.items().len(), 1);
    let again = only_drafts(update(&mut state, drafts_reply(2, named(&["A", "B"]))));
    assert_eq!(again, vec![list_command("a1", 3)]);
    assert_eq!(state.drafts.items().len(), 2);
}

//! Vue « file » : ouverture, réessayer, annuler, décision sur un envoi incertain, jamais pendant l'édition.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod action_fixtures;
mod compose_fixtures;
mod message_fixtures;
mod navigation_fixtures;
mod queue_fixtures;
mod reader_fixtures;
mod sidebar_fixtures;

use action_fixtures::state_with_mailboxes;
use mira_assembly::api::{ComposeRequest, QueueRequest, Request};
use mira_assembly::mira_vocabulary::{ActionId, ActionState};
use mira_tui::state::Popup;
use mira_tui::update::{Intent, QueueIntent};
use navigation_fixtures::press;
use queue_fixtures::{
    change_command, list_commands, open_mixed_queue, open_queue_with, outbox_entry,
};

/// `Q` ouvre la vue sur le compte sélectionné et lit boîte d'envoi et actions ; Échap la referme.
#[test]
fn queue_key_opens_the_view_and_lists_both() {
    let mut state = state_with_mailboxes();
    let commands = press(&mut state, Intent::Queue(QueueIntent::Open));
    assert_eq!(commands, list_commands("a1", 1));
    assert!(state.queue.is_open());
    assert!(press(&mut state, Intent::Back).is_empty());
    assert!(!state.queue.is_open());
}

/// `r` relance l'envoi échoué (même action, donc même clé) ; sur une fiche non échouée, un message et rien d'envoyé.
#[test]
fn retry_emits_a_retry_for_failed_entries_only() {
    let mut state = state_with_mailboxes();
    open_mixed_queue(&mut state);
    assert_eq!(state.queue.entries().len(), 3);
    let retry = Request::Queue(QueueRequest::RetryAction {
        action: ActionId::from_raw("s1"),
    });
    let commands = press(&mut state, Intent::Queue(QueueIntent::Retry));
    assert_eq!(commands, vec![change_command("s1", retry)]);
    press(&mut state, Intent::MoveDown);
    assert!(press(&mut state, Intent::Queue(QueueIntent::Retry)).is_empty());
    assert!(state.status.message.is_some());
}

/// `x` annule l'envoi (`CancelSend`) ou l'action (`CancelAction`) en attente ; une fiche échouée ne s'annule pas.
#[test]
fn retry_and_cancel_emit_queue_requests() {
    let mut state = state_with_mailboxes();
    open_mixed_queue(&mut state);
    assert!(press(&mut state, Intent::Queue(QueueIntent::Cancel)).is_empty());
    press(&mut state, Intent::MoveDown);
    let cancel_send = Request::Compose(ComposeRequest::CancelSend {
        action: ActionId::from_raw("s2"),
    });
    let commands = press(&mut state, Intent::Queue(QueueIntent::Cancel));
    assert_eq!(commands, vec![change_command("s2", cancel_send)]);
    press(&mut state, Intent::MoveDown);
    let cancel = Request::Queue(QueueRequest::CancelAction {
        action: ActionId::from_raw("x3"),
    });
    let commands = press(&mut state, Intent::Queue(QueueIntent::Cancel));
    assert_eq!(commands, vec![change_command("x3", cancel)]);
}

/// Un envoi incertain demande « renvoyer ou non » ; rien ne part sans la décision, Échap remet à plus tard.
#[test]
fn uncertain_send_asks_resend_or_not() {
    let mut state = state_with_mailboxes();
    let doubtful = outbox_entry("s1", ActionState::Uncertain, "Peut-être", None);
    open_queue_with(&mut state, vec![doubtful], Vec::new());
    assert!(press(&mut state, Intent::Queue(QueueIntent::Resolve)).is_empty());
    assert!(
        matches!(&state.popup, Popup::ResolveUncertain { action, .. } if action.as_str() == "s1")
    );
    assert!(press(&mut state, Intent::Confirm).is_empty());
    assert!(press(&mut state, Intent::Cancel).is_empty());
    assert_eq!(state.popup, Popup::Closed);
    let resolve = |resend| {
        Request::Compose(ComposeRequest::ResolveUncertainSend {
            action: ActionId::from_raw("s1"),
            resend,
        })
    };
    press(&mut state, Intent::Open);
    assert_eq!(
        press(&mut state, Intent::Queue(QueueIntent::Resend)),
        vec![change_command("s1", resolve(true))]
    );
    assert_eq!(state.popup, Popup::Closed);
    press(&mut state, Intent::Queue(QueueIntent::Resolve));
    assert_eq!(
        press(&mut state, Intent::Queue(QueueIntent::DontResend)),
        vec![change_command("s1", resolve(false))]
    );
}

/// Seul un envoi incertain ouvre la décision ; une autre fiche laisse un message et aucune fenêtre.
#[test]
fn resolve_on_a_certain_entry_does_nothing() {
    let mut state = state_with_mailboxes();
    let waiting = outbox_entry("s2", ActionState::Pending, "Attente", None);
    open_queue_with(&mut state, vec![waiting], Vec::new());
    assert!(press(&mut state, Intent::Queue(QueueIntent::Resolve)).is_empty());
    assert_eq!(state.popup, Popup::Closed);
    assert!(state.status.message.is_some());
}

/// `Q` n'agit jamais pendant l'édition d'un brouillon : la vue ne s'ouvre pas, rien n'est lu.
#[test]
fn queue_never_opens_while_editing() {
    let mut state = compose_fixtures::open_reply();
    assert!(press(&mut state, Intent::Queue(QueueIntent::Open)).is_empty());
    assert!(!state.queue.is_open());
    assert!(state.compose.open().is_some());
}

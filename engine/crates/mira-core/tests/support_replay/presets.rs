//! Bancs prêts à l'emploi : service à dossiers standard, rejeu d'une seule action sur msg-1.

use mira_core::actions::UNDO_DELAY_MILLIS;
use mira_testkit::imap::CapabilitySet;
use mira_vocabulary::{ActionKind, ProviderQuirks};

use super::fixture::Fixture;
use crate::support_actions::requests::submit_folders;

/// Banc à dossiers dont le serveur ne copie pas lui-même dans Envoyés
pub async fn standard() -> Fixture {
    Fixture::start(CapabilitySet::full(), ProviderQuirks::default()).await
}

/// Banc standard dont l'action donnée sur msg-1 est soumise, rejouée et a réussi
pub async fn replay_on_msg_1(kind: ActionKind) -> Fixture {
    let fx = standard().await;
    submit_folders(&fx.world, kind, &["msg-1"], "k-1");
    fx.advance(UNDO_DELAY_MILLIS + 1);
    let mut client = fx.client().await;
    let report = fx.replay(&mut client).await;
    assert_eq!((report.succeeded, report.failed), (1, 0));
    fx
}

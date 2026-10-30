#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Boucle d'outils, annulation : pendant le tour qui suit un outil, la requête en cours est abandonnée.

mod ai_loop_support;
mod ai_models_support;
mod support;
mod support_api;
mod support_mirror;
mod support_read;

use std::time::Duration;

use ai_loop_support::fixture::LoopFixture;
use ai_loop_support::turns::{FORMATS, tool_turn};
use mira_core::api::{OperationRequest, Request};
use mira_testkit::ai::Behavior;
use mira_vocabulary::{ChatDelta, StopReason};
use tokio::time::timeout;

/// Annuler pendant le tour qui suit l'outil termine par l'annulation, sans autre requête au modèle
#[tokio::test]
async fn cancel_during_the_round_after_a_tool_stops_the_loop() {
    for format in FORMATS {
        let fixture = LoopFixture::start(format).await;
        let calls = [("c1", "list_mailboxes", "{}")];
        fixture.queue(tool_turn(format, "", &calls, None));
        fixture.queue(Behavior::Hang);
        let mut stream = fixture.ask("Liste").await;
        let mut seen = Vec::new();
        let read = async {
            while let Some(delta) = stream.next().await {
                let finished = matches!(delta, ChatDelta::ToolFinished { .. });
                seen.push(delta);
                if finished {
                    break;
                }
            }
        };
        timeout(Duration::from_secs(10), read).await.unwrap();
        let cancel = Request::Operation(OperationRequest::CancelOperation {
            operation: stream.operation.clone(),
        });
        fixture.engine.execute(cancel).await.unwrap();
        let rest = ai_loop_support::turns::collect(&mut stream).await;
        let done = ChatDelta::Done {
            reason: StopReason::Cancelled,
        };
        assert_eq!(rest.last(), Some(&done), "{format:?}");
        assert_eq!(fixture.bodies().len(), 2);
    }
}

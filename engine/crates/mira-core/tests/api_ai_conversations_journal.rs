#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Contrat 1.4 : effacer les conversations ne touche pas au journal des envois à l'IA.

mod ai_chat_support;
mod ai_models_support;
mod support_ai_api;
mod support_ai_conversations;
mod support_api;

use ai_chat_support::{chat_start, connection_with, run_chat};
use ai_models_support::{ModelsFixture, TestProvider};
use mira_testkit::ai::openai_stream;
use mira_vocabulary::MaskingSettings;
use support_ai_api::privacy::journal;
use support_ai_conversations::changes::clear;
use support_ai_conversations::{list, save};

/// Après une discussion et un enregistrement, tout effacer laisse la ligne du journal intacte
#[tokio::test]
async fn clear_leaves_ai_journal_untouched() {
    let f = ModelsFixture::start(TestProvider::new("test-provider")).await;
    let connection = connection_with(&f, &MaskingSettings::remote_default());
    f.server.queue(openai_stream(&["ok"], Some((3, 1))));
    run_chat(&f, chat_start(&connection, "bonjour")).await;
    save(&f.engine, "a", "A", "{}").await.unwrap();
    let before = journal(&f.engine, 50, None).await.unwrap();
    assert_eq!(before.len(), 1);
    clear(&f.engine).await.unwrap();
    assert!(list(&f.engine, 50, None).await.unwrap().is_empty());
    assert_eq!(journal(&f.engine, 50, None).await.unwrap(), before);
}

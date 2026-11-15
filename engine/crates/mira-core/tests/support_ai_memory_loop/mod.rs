#![allow(dead_code, clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Outils des tests de la mémoire dans la boucle du chat : activation, souvenir indexé, réponses factices.

use crate::ai_loop_support::fixture::LoopFixture;
use crate::ai_loop_support::turns::collect;
use mira_core::ai::memory::encode_vector;
use mira_testkit::ai::Behavior;
use mira_vocabulary::{
    ChatDelta, MemoryEmbedder, MemoryKind, MemoryRecord, MemorySettings, MemorySource, Timestamp,
};

/// Active la mémoire sur la connexion du faux serveur et y range un souvenir déjà indexé
pub fn enable_with_one_memory(f: &LoopFixture) {
    let settings = MemorySettings {
        enabled: true,
        embedder: MemoryEmbedder::Provider {
            connection: f.connection.id.clone(),
            model: "embed-1".into(),
        },
    };
    f.db.write(|tx| tx.set_ai_memory_settings(&settings))
        .unwrap();
    let record = MemoryRecord {
        id: "mem-1".into(),
        text: "Mon chat s'appelle Mochi".into(),
        kind: MemoryKind::Fact,
        source: MemorySource::User,
        created_at: Timestamp::from_millis(1),
        model_id: Some("test-provider:embed-1".into()),
    };
    let bytes = encode_vector(&[1.0, 0.0]);
    f.db.write(|tx| tx.insert_ai_memory(&record, &bytes, 2))
        .unwrap();
}

/// Réponse d'embeddings à un seul texte
pub fn one_vector() -> Behavior {
    Behavior::Status {
        code: 200,
        headers: vec![],
        body: r#"{"data":[{"index":0,"embedding":[1.0,0.0]}]}"#.to_string(),
    }
}

/// Met ces comportements en file, dans l'ordre où le faux serveur les servira
pub fn queue_all<const N: usize>(f: &LoopFixture, behaviors: [Behavior; N]) {
    for behavior in behaviors {
        f.queue(behavior);
    }
}

/// Met ces comportements en file, pose la question avec les outils et rend tous les deltas
pub async fn run_turns<const N: usize>(
    f: &LoopFixture,
    behaviors: [Behavior; N],
    question: &str,
) -> Vec<ChatDelta> {
    queue_all(f, behaviors);
    let mut stream = f.ask(question).await;
    collect(&mut stream).await
}

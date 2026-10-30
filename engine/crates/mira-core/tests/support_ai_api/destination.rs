//! Connexion complète (clé, consentement, modèles gardés) pour éprouver ce qu'un changement d'adresse oublie.

use mira_vocabulary::AiConnection;

use super::calls::add;
use super::operations::{consent, models, set_key};
use super::providers::EDITABLE;
use crate::ai_models_support::ModelsFixture;
use mira_testkit::ai::Behavior;

/// Crée une connexion vers le faux serveur avec une clé, le consentement et une liste de modèles gardée
pub async fn prepared(f: &ModelsFixture) -> AiConnection {
    let url = f.server.base_url();
    let created = add(&f.engine, EDITABLE, "A", Some(&url)).await.unwrap();
    set_key(&f.engine, &created.id, "sk-destination")
        .await
        .unwrap();
    consent(&f.engine, &created.id).await;
    f.server.queue(Behavior::Models {
        json: r#"{"data":[{"id":"model-a","created":100}]}"#.to_string(),
    });
    models(&f.engine, &created.id, true).await.unwrap();
    let ready = list_one(f).await;
    assert!(ready.has_key && ready.consent_at.is_some() && ready.models.is_some());
    ready
}

/// Relit l'unique connexion par la façade
async fn list_one(f: &ModelsFixture) -> AiConnection {
    super::calls::list(&f.engine).await.remove(0)
}

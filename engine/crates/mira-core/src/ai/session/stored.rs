//! Lecture au trousseau de la session d'une connexion d'IA.

use mira_vocabulary::{AiConnectionId, AiSecretRef};

use crate::api::Engine;
use crate::auth::session::AiSession;

/// Session rangée pour la connexion ; un secret absent ou qui n'en est pas une rend `None`
pub(crate) fn read_session(engine: &Engine, id: &AiConnectionId) -> Option<AiSession> {
    let location = AiSecretRef {
        profile: engine.config.profile.clone(),
        connection: id.clone(),
    };
    let secret = engine.config.secrets.get(&location).ok()??;
    AiSession::from_secret(&secret).ok()
}

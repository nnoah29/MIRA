#![allow(dead_code, clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Raccourcis des tests du champ « ce que tu dois savoir de moi » : requête, enregistrement, relecture.

use crate::local_chat_support::fixture::LocalFixture;
use mira_core::api::{AiRequest, AiResponse, Request, Response};
use mira_vocabulary::MiraError;

/// Exécute une requête du champ et rend la réponse du domaine IA
pub async fn ask(f: &LocalFixture, request: AiRequest) -> Result<AiResponse, MiraError> {
    match f.engine.execute(Request::Ai(request)).await? {
        Response::Ai(answer) => Ok(answer),
        other => panic!("réponse inattendue : {other:?}"),
    }
}

/// Enregistre ce texte et rend ce que le moteur a gardé
pub async fn save(f: &LocalFixture, text: &str) -> Result<String, MiraError> {
    let request = AiRequest::SetAboutMe {
        text: text.to_string(),
    };
    match ask(f, request).await? {
        AiResponse::AboutMe(kept) => Ok(kept),
        other => panic!("réponse inattendue : {other:?}"),
    }
}

/// Relit le texte enregistré
pub async fn read(f: &LocalFixture) -> String {
    match ask(f, AiRequest::GetAboutMe).await.unwrap() {
        AiResponse::AboutMe(text) => text,
        other => panic!("réponse inattendue : {other:?}"),
    }
}

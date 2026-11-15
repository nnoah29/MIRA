//! Champ « ce que tu dois savoir de moi » : lecture et enregistrement du texte de l'utilisateur.

use mira_vocabulary::MiraError;

use crate::api::contract::{AiResponse, Response};
use crate::api::engine::Engine;

/// Rend le texte enregistré, vide tant que rien n'a été écrit
pub(crate) fn handle_get_about_me(engine: &Engine) -> Result<Response, MiraError> {
    let text = engine.database.read(|tx| tx.ai_about())?;
    Ok(Response::Ai(AiResponse::AboutMe(text)))
}

/// Enregistre le texte, rogné et borné, et rend ce qui a été gardé
pub(crate) fn handle_set_about_me(engine: &Engine, text: &str) -> Result<Response, MiraError> {
    let kept = engine.database.write(|tx| tx.set_ai_about(text))?;
    Ok(Response::Ai(AiResponse::AboutMe(kept)))
}

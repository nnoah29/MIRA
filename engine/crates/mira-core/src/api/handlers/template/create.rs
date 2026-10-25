//! Création d'un modèle de message : champs contrôlés par le dépôt, idempotente par clé.

use mira_vocabulary::{IdempotencyKey, MessageTemplate, MiraError, TemplateId};

use super::idempotency::TemplateCommand;
use super::writing::{announce, single};
use crate::api::contract::Response;
use crate::api::engine::Engine;

/// Crée le modèle sous un identifiant du moteur ; la même clé rejouée rend le même modèle sans rien écrire
pub(crate) fn handle_create_template(
    engine: &Engine,
    name: &str,
    subject: Option<&str>,
    body: &str,
    key: IdempotencyKey,
) -> Result<Response, MiraError> {
    let command = TemplateCommand::new(key, "create_template", &(name, subject, body))?;
    let id = TemplateId::from_raw(engine.config.ids.next_id());
    let now = engine.config.clock.now().as_millis();
    let (template, written) = engine.database.write(|tx| {
        if let Some(template) = command.replay::<MessageTemplate>(tx)? {
            return Ok((template, false));
        }
        let template = new_template(id.clone(), name, subject, body, now);
        tx.insert_template(&template)?;
        command.record(tx, &template)?;
        Ok((template, true))
    })?;
    if written {
        announce(engine)?;
    }
    Ok(single(template))
}

/// Modèle neuf : nom sans espaces de bord, créé et mis à jour au même instant
fn new_template(
    id: TemplateId,
    name: &str,
    subject: Option<&str>,
    body: &str,
    now: i64,
) -> MessageTemplate {
    MessageTemplate {
        id,
        name: name.trim().to_string(),
        subject: subject.map(str::to_string),
        body: body.to_string(),
        created_at: now,
        updated_at: now,
    }
}

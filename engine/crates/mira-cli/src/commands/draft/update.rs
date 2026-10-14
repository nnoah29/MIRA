//! `mira draft update` : modifie les champs d'un brouillon ; un champ absent ne change pas.

use clap::Args;
use mira_assembly::api::{ComposeRequest, DraftChanges};
use mira_assembly::mira_vocabulary::{DraftId, IdentityId, MiraError};

use super::recipients::recipients;
use crate::argument_error::invalid_argument;

/// Options de `mira draft update`.
#[derive(Debug, Args)]
pub struct UpdateArgs {
    /// Identifiant du brouillon.
    pub draft: String,
    /// Identité d'envoi.
    #[arg(long)]
    pub identity: Option<String>,
    /// Destinataires (répétable ; une valeur vide efface la liste).
    #[arg(long)]
    pub to: Vec<String>,
    /// Destinataires en copie (répétable ; une valeur vide efface la liste).
    #[arg(long)]
    pub cc: Vec<String>,
    /// Destinataires en copie cachée (répétable ; une valeur vide efface la liste).
    #[arg(long)]
    pub bcc: Vec<String>,
    /// Objet du message.
    #[arg(long)]
    pub subject: Option<String>,
    /// Corps en texte.
    #[arg(long, conflicts_with = "body_file")]
    pub body: Option<String>,
    /// Fichier dont le contenu devient le corps en texte.
    #[arg(long)]
    pub body_file: Option<String>,
    /// Corps en HTML (une valeur vide efface la version HTML).
    #[arg(long)]
    pub html: Option<String>,
}

/// Corps en texte : option `--body`, sinon contenu du fichier `--body-file`.
fn body_text(args: &UpdateArgs) -> Result<Option<String>, MiraError> {
    match (&args.body, &args.body_file) {
        (Some(text), _) => Ok(Some(text.clone())),
        (None, Some(path)) => std::fs::read_to_string(path)
            .map(Some)
            .map_err(|_| invalid_argument("fichier du corps illisible")),
        (None, None) => Ok(None),
    }
}

/// Construit la requête de modification d'un brouillon.
pub fn request(args: &UpdateArgs) -> Result<ComposeRequest, MiraError> {
    Ok(ComposeRequest::UpdateDraft {
        draft: DraftId::from_raw(args.draft.as_str()),
        changes: DraftChanges {
            identity: args.identity.as_deref().map(IdentityId::from_raw),
            to: recipients(&args.to)?,
            cc: recipients(&args.cc)?,
            bcc: recipients(&args.bcc)?,
            subject: args.subject.clone(),
            body_text: body_text(args)?,
            body_html: args.html.clone(),
        },
    })
}

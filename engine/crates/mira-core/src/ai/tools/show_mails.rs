//! Outil `show_mails` : montre à l'utilisateur des messages lus avec les outils, chacun en carte cliquable avec sa raison.

use mira_vocabulary::{MailCard, MailCards, ToolSpec};
use serde::Deserialize;

use super::args::parse;
use super::run_result::ToolRun;

pub(super) const NAME: &str = "show_mails";

const MAX_CARDS: usize = 50;
const MAX_ID_CHARS: usize = 200;
const MAX_TEXT_CHARS: usize = 200;

const SCHEMA: &str = r#"{"type":"object","properties":{
"title":{"type":"string","description":"Titre court de la liste, facultatif."},
"mails":{"type":"array","items":{"type":"object","properties":{
"id":{"type":"string","description":"Identifiant du message (champ id donné par les outils de lecture)."},
"reason":{"type":"string","description":"Pourquoi ce message, en une phrase."}},"required":["id"]},
"description":"Les messages à montrer, 50 au plus, dans l'ordre voulu."}
},"required":["mails"]}"#;

/// Un message à montrer, tel que le modèle l'écrit.
#[derive(Debug, Deserialize)]
struct CardArgs {
    id: String,
    reason: Option<String>,
}

/// Arguments de l'outil : titre facultatif et messages.
#[derive(Debug, Deserialize)]
struct ShowArgs {
    title: Option<String>,
    mails: Vec<CardArgs>,
}

/// Fiche de l'outil annoncée au modèle.
pub(super) fn spec() -> ToolSpec {
    ToolSpec {
        name: NAME.to_string(),
        description: "Montre à l'utilisateur des messages que tu as trouvés avec les outils de lecture : chacun devient une carte cliquable dans la conversation, avec ta raison. À utiliser dès que tu présentes une liste de messages.".to_string(),
        parameters_schema: SCHEMA.to_string(),
    }
}

/// Valide les cartes ; le modèle apprend que l'utilisateur les voit et qu'il n'a pas à les recopier.
pub(super) fn run(arguments: &str) -> ToolRun {
    match build(arguments) {
        Ok(cards) => ToolRun::cards_shown(
            "Les cartes sont affichées à l'utilisateur, qui peut ouvrir chaque message : ne recopie pas la liste dans ta réponse, commente-la en une ou deux phrases.".to_string(),
            cards,
        ),
        Err(text) => ToolRun::error(text),
    }
}

/// Lot de cartes tiré des arguments : titre et raisons sur une ligne, identifiants distincts et bornés.
fn build(arguments: &str) -> Result<MailCards, String> {
    let args: ShowArgs = parse(arguments)?;
    let title = one_line(args.title, "le titre")?;
    let cards = distinct_cards(args.mails)?;
    Ok(MailCards { title, cards })
}

/// Texte rogné, absent s'il est vide, refusé s'il court sur plusieurs lignes ou dépasse la borne.
fn one_line(text: Option<String>, what: &str) -> Result<Option<String>, String> {
    let text = text
        .map(|text| text.trim().to_string())
        .filter(|text| !text.is_empty());
    let bad = text
        .as_deref()
        .is_some_and(|text| text.contains(['\n', '\r']) || text.chars().count() > MAX_TEXT_CHARS);
    match bad {
        true => Err(format!(
            "{what} doit tenir sur une ligne de {MAX_TEXT_CHARS} caractères au plus"
        )),
        false => Ok(text),
    }
}

/// Cartes sans doublon d'identifiant (la première raison reste), de 1 à 50.
fn distinct_cards(list: Vec<CardArgs>) -> Result<Vec<MailCard>, String> {
    let mut kept: Vec<MailCard> = Vec::new();
    for item in list {
        let id = item.id.trim().to_string();
        if id.is_empty() || id.chars().count() > MAX_ID_CHARS {
            return Err("identifiant de message vide ou trop long".to_string());
        }
        let reason = one_line(item.reason, "la raison")?;
        if !kept.iter().any(|card| card.id == id) {
            kept.push(MailCard { id, reason });
        }
    }
    match kept.is_empty() || kept.len() > MAX_CARDS {
        true => Err(format!("il faut de 1 à {MAX_CARDS} messages")),
        false => Ok(kept),
    }
}

//! Consigne de style ajoutée au message système : courte, sans donnée personnelle, absente si le profil ne sert pas.

use mira_vocabulary::{StyleProfile, StyleRegister};

const MAX_CHARS: usize = 800;
const HEAD: &str =
    "Style de l'utilisateur, appris de ses mails envoyés, à respecter pour écrire à sa place : ";
const NOTES: &str = " Notes de l'utilisateur : ";

/// Phrase sur le registre, absente s'il est inconnu.
fn register_part(register: StyleRegister) -> Option<String> {
    match register {
        StyleRegister::Tu => Some("il tutoie".to_string()),
        StyleRegister::Vous => Some("il vouvoie".to_string()),
        StyleRegister::Mixed => Some("il tutoie ou vouvoie selon le destinataire".to_string()),
        StyleRegister::Unknown => None,
    }
}

/// Phrase sur les ouvertures, absente s'il n'y en a pas.
fn greeting_part(greetings: &[String]) -> Option<String> {
    let quoted: Vec<String> = greetings.iter().map(|g| format!("« {g} »")).collect();
    (!quoted.is_empty()).then(|| format!("il commence par {}", quoted.join(" ou ")))
}

/// Les constats du profil, dans l'ordre : registre, longueur, ouverture, ponctuation, émojis.
fn parts(profile: &StyleProfile) -> Vec<String> {
    let length = format!(
        "ses mails font environ {} mots, ses phrases environ {} mots",
        profile.median_words, profile.median_sentence_words
    );
    let marks = match profile.exclamation_permille >= 100 {
        true => "il utilise volontiers des points d'exclamation",
        false => "il n'abuse pas des points d'exclamation",
    };
    let emoji = match profile.emoji {
        true => "il emploie des émojis",
        false => "il n'emploie pas d'émojis",
    };
    let first = [
        register_part(profile.register),
        Some(length),
        greeting_part(&profile.greetings),
    ];
    first
        .into_iter()
        .flatten()
        .chain([marks.to_string(), emoji.to_string()])
        .collect()
}

/// Consigne de style (800 caractères au plus, notes comprises), ou rien si le profil n'est pas exploitable.
pub fn render_style_notice(profile: &StyleProfile, notes: &str) -> Option<String> {
    if !profile.usable() {
        return None;
    }
    let mut notice = format!("{HEAD}{}.", parts(profile).join(" ; "));
    let notes = notes.split_whitespace().collect::<Vec<_>>().join(" ");
    let room = MAX_CHARS.saturating_sub(notice.chars().count() + NOTES.chars().count());
    if !notes.is_empty() && room > 0 {
        notice.push_str(NOTES);
        notice.extend(notes.chars().take(room));
    }
    Some(notice)
}

//! Analyse du style : un profil calculé sur les mails envoyés, sans aucune sortie ni donnée personnelle.

use mira_vocabulary::{STYLE_MIN_MAILS, StyleProfile};

use super::clean::clean_mail;
use super::greetings::{greeting_of, top_greetings};
use super::measures::{exclamations, has_emoji, median, sentence_word_counts, word_count};
use super::register::dominant_register;

const MAX_GREETINGS: usize = 3;

/// Mesures détaillées d'un lot de mails déjà nettoyés et non vides.
fn measured(cleaned: &[String]) -> StyleProfile {
    let mut words: Vec<u32> = cleaned.iter().map(|m| word_count(m)).collect();
    let mut sentences: Vec<u32> = cleaned
        .iter()
        .flat_map(|m| sentence_word_counts(m))
        .collect();
    let marks: u32 = cleaned.iter().map(|m| exclamations(m)).sum();
    let total = u32::try_from(sentences.len()).unwrap_or(u32::MAX).max(1);
    let found: Vec<String> = cleaned.iter().filter_map(|m| greeting_of(m)).collect();
    StyleProfile {
        samples: u32::try_from(cleaned.len()).unwrap_or(u32::MAX),
        register: dominant_register(cleaned),
        greetings: top_greetings(&found, MAX_GREETINGS),
        median_words: median(&mut words),
        median_sentence_words: median(&mut sentences),
        exclamation_permille: (marks.saturating_mul(1000) / total).min(1000),
        emoji: cleaned.iter().any(|m| has_emoji(m)),
    }
}

/// Profil de style de ces mails ; sous le seuil de mails exploitables, seul leur nombre est renseigné.
pub fn analyze(mails: &[String]) -> StyleProfile {
    let cleaned: Vec<String> = mails
        .iter()
        .map(|m| clean_mail(m))
        .filter(|m| !m.is_empty())
        .collect();
    let samples = u32::try_from(cleaned.len()).unwrap_or(u32::MAX);
    match samples >= STYLE_MIN_MAILS {
        true => measured(&cleaned),
        false => StyleProfile {
            samples,
            ..StyleProfile::default()
        },
    }
}

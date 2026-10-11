//! Destinataires d'une réponse et d'une réponse à tous.

use mira_vocabulary::{AddressEntry, MessageSummary};

/// Destinataires principaux et en copie d'un brouillon de réponse.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct ReplyRecipients {
    pub(crate) to: Vec<AddressEntry>,
    pub(crate) cc: Vec<AddressEntry>,
}

/// Réponse : à `Reply-To`, sinon à l'expéditeur ; si l'expéditeur est soi-même, aux destinataires d'origine
pub(crate) fn reply_to(original: &MessageSummary, own: &[String]) -> Vec<AddressEntry> {
    let direct = if original.reply_to.is_empty() {
        &original.from
    } else {
        &original.reply_to
    };
    if direct.iter().all(|entry| is_own(entry, own)) && !original.to.is_empty() {
        return dedupe(original.to.clone());
    }
    dedupe(direct.clone())
}

/// Réponse à tous : les destinataires de la réponse, plus les autres destinataires d'origine en copie, sans soi-même
pub(crate) fn reply_all(original: &MessageSummary, own: &[String]) -> ReplyRecipients {
    let to = reply_to(original, own);
    let mut cc: Vec<AddressEntry> = original
        .to
        .iter()
        .chain(original.cc.iter())
        .filter(|entry| !is_own(entry, own))
        .filter(|entry| !contains(&to, entry))
        .cloned()
        .collect();
    cc = dedupe(cc);
    ReplyRecipients { to, cc }
}

/// Dit si l'adresse est l'une des adresses de l'utilisateur (comparaison sans casse)
fn is_own(entry: &AddressEntry, own: &[String]) -> bool {
    own.iter()
        .any(|address| address.eq_ignore_ascii_case(&entry.address))
}

/// Dit si la liste contient déjà cette adresse (sans casse)
fn contains(list: &[AddressEntry], entry: &AddressEntry) -> bool {
    list.iter()
        .any(|other| other.address.eq_ignore_ascii_case(&entry.address))
}

/// Retire les adresses en double en gardant la première occurrence
fn dedupe(entries: Vec<AddressEntry>) -> Vec<AddressEntry> {
    let mut kept: Vec<AddressEntry> = Vec::new();
    for entry in entries {
        if !contains(&kept, &entry) {
            kept.push(entry);
        }
    }
    kept
}

//! Application d'une reprise dans la transaction : libellés créés ou réutilisés, poses, clés ignorées.

use mira_vocabulary::{LocalLabel, LocalLabelId, MessageId, MiraError, ThreadId};

use super::checks::{clean_name, ensure_room};
use super::writing::default_color;
use crate::api::contract::{ImportReport, ImportedLabel};
use crate::ports::IdGenerator;
use crate::storage::Tx;

/// Libellé de même nom s'il existe, sinon libellé neuf créé avec la couleur reprise
fn label_for(
    tx: &Tx<'_>,
    ids: &dyn IdGenerator,
    wanted: &ImportedLabel,
) -> Result<LocalLabel, MiraError> {
    let name = clean_name(&wanted.name)?;
    if let Some(found) = tx.find_local_label_by_name(&name)? {
        return Ok(found);
    }
    ensure_room(tx, 1)?;
    let label = LocalLabel {
        id: LocalLabelId::from_raw(ids.next_id()),
        name,
        color: if wanted.color == 0 {
            default_color(0)
        } else {
            wanted.color
        },
    };
    tx.insert_local_label(&label)?;
    Ok(label)
}

/// Fil désigné par une clé de conversation : un fil, sinon le fil d'un message, sinon rien
fn resolve_thread(tx: &Tx<'_>, key: &str) -> Result<Option<ThreadId>, MiraError> {
    let thread = ThreadId::from_raw(key);
    if tx.thread_root_key(&thread)?.is_some() {
        return Ok(Some(thread));
    }
    let message = tx.get_message(&MessageId::from_raw(key))?;
    Ok(message.and_then(|m| m.thread))
}

/// Reprend tous les libellés demandés et leurs poses ; rend le bilan
pub(super) fn apply_import(
    tx: &Tx<'_>,
    ids: &dyn IdGenerator,
    labels: &[ImportedLabel],
) -> Result<ImportReport, MiraError> {
    let mut report = ImportReport {
        labels: 0,
        assignments: 0,
        ignored: 0,
    };
    for wanted in labels {
        let label = label_for(tx, ids, wanted)?;
        report.labels += 1;
        for key in &wanted.conversations {
            match resolve_thread(tx, key)? {
                Some(thread) => {
                    tx.set_label_on_thread(&label.id, &thread, true)?;
                    report.assignments += 1;
                }
                None => report.ignored += 1,
            }
        }
    }
    Ok(report)
}

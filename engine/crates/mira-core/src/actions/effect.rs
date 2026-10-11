//! Application d'une suite d'étapes au miroir, avec mémorisation de l'inverse et mise à jour des compteurs.

use mira_vocabulary::{MailboxId, MiraError};

use super::model::LocalStep;
use super::steps::{apply_step, capture_inverse, step_in_effect};
use crate::storage::Tx;

/// Applique toutes les étapes ; rend, pour chacune, les étapes qui rétabliraient l'état précédent
pub(crate) fn apply_all(
    tx: &Tx<'_>,
    steps: &[LocalStep],
) -> Result<Vec<Vec<LocalStep>>, MiraError> {
    apply_each(tx, steps, false)
}

/// N'applique que les étapes que le miroir ne reflète pas encore ; rend leurs inverses
pub(crate) fn apply_missing(
    tx: &Tx<'_>,
    steps: &[LocalStep],
) -> Result<Vec<Vec<LocalStep>>, MiraError> {
    apply_each(tx, steps, true)
}

/// Défait un effet : rejoue les inverses du plus récent au plus ancien
pub(crate) fn apply_inverse(tx: &Tx<'_>, inverse: &[Vec<LocalStep>]) -> Result<(), MiraError> {
    let mut touched = Vec::new();
    for group in inverse.iter().rev() {
        for step in group {
            touched.extend(apply_step(tx, step)?);
        }
    }
    refresh_counters(tx, touched)
}

/// Capture l'inverse puis applique chaque étape, en sautant celles déjà en place si demandé
fn apply_each(
    tx: &Tx<'_>,
    steps: &[LocalStep],
    only_missing: bool,
) -> Result<Vec<Vec<LocalStep>>, MiraError> {
    let mut inverse = Vec::new();
    let mut touched = Vec::new();
    for step in steps {
        if only_missing && step_in_effect(tx, step)? {
            continue;
        }
        inverse.push(capture_inverse(tx, step)?);
        touched.extend(apply_step(tx, step)?);
    }
    refresh_counters(tx, touched)?;
    Ok(inverse)
}

/// Recalcule les compteurs des boîtes touchées qui existent encore
fn refresh_counters(tx: &Tx<'_>, mut touched: Vec<MailboxId>) -> Result<(), MiraError> {
    touched.sort();
    touched.dedup();
    for mailbox in touched {
        if tx.get_mailbox(&mailbox)?.is_some() {
            tx.refresh_mailbox_counters(&mailbox)?;
        }
    }
    Ok(())
}

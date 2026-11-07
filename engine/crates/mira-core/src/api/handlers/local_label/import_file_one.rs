//! Import d'un libellé du fichier : lecture, nom, doublons, plafond, puis insertion ou remplacement, réglages compris.

use std::collections::HashSet;

use mira_vocabulary::{
    LabelImportOutcome, LabelImportRefusal, LocalLabel, LocalLabelId, MiraError,
};
use serde_json::Value;

use super::checks::{MAX_LABELS, MAX_NAME_LEN, clean_name};
use super::import_file_settings::{merge_existing, write_new};
use super::writing::default_color;
use crate::portability::labels::{PortableLabelError, PortableSettings, read_portable_entry};
use crate::ports::IdGenerator;
use crate::storage::Tx;

/// Clés de noms déjà vues dans le fichier, et nombre de libellés présents ou ajoutés
type Progress<'a> = (&'a mut HashSet<String>, &'a mut usize);

/// Sort « refusé » pour cette raison
fn refused(reason: LabelImportRefusal) -> LabelImportOutcome {
    LabelImportOutcome::Refused { reason }
}

/// Nom de l'entrée tel que le rapport le montre : tronqué, vide si l'entrée n'en a pas
pub(super) fn shown_name(value: &Value) -> String {
    let name = value.get("name").and_then(Value::as_str).unwrap_or("");
    name.chars().take(MAX_NAME_LEN).collect()
}

/// Nom contrôlé, couleur et réglages facultatifs d'une entrée, ou le sort « refusé » qui l'écarte
fn read_entry(value: Value) -> Result<(String, Option<u32>, PortableSettings), LabelImportOutcome> {
    let (parsed, settings) = read_portable_entry(value).map_err(|error| match error {
        PortableLabelError::InvalidColor => refused(LabelImportRefusal::InvalidColor),
        PortableLabelError::Invalid => refused(LabelImportRefusal::InvalidLabel),
    })?;
    let name = clean_name(&parsed.name).map_err(|_| refused(LabelImportRefusal::InvalidName))?;
    Ok((name, parsed.color, settings))
}

/// Importe une entrée du fichier ; seule une panne de stockage interrompt tout l'import
pub(super) fn import_label(
    tx: &Tx<'_>,
    ids: &dyn IdGenerator,
    value: Value,
    replace: bool,
    (seen, count): Progress<'_>,
) -> Result<LabelImportOutcome, MiraError> {
    let (name, color, settings) = match read_entry(value) {
        Ok(entry) => entry,
        Err(refusal) => return Ok(refusal),
    };
    if !seen.insert(name.to_lowercase()) {
        return Ok(refused(LabelImportRefusal::DuplicateInFile));
    }
    match tx.find_local_label_by_name(&name)? {
        Some(existing) => update_existing(tx, existing, (&name, color, &settings), replace),
        None => add_new(tx, ids, (name, color, &settings), count),
    }
}

/// Libellé neuf, sauf si le plafond est atteint ; la couleur absente vient de la palette
fn add_new(
    tx: &Tx<'_>,
    ids: &dyn IdGenerator,
    (name, color, settings): (String, Option<u32>, &PortableSettings),
    count: &mut usize,
) -> Result<LabelImportOutcome, MiraError> {
    if *count >= MAX_LABELS {
        return Ok(refused(LabelImportRefusal::TooManyLabels));
    }
    let color = color.unwrap_or_else(|| default_color(*count));
    let id = LocalLabelId::from_raw(ids.next_id());
    tx.insert_local_label(&LocalLabel {
        id: id.clone(),
        name,
        color,
    })?;
    write_new(tx, &id, settings)?;
    *count += 1;
    Ok(LabelImportOutcome::Imported)
}

/// Libellé déjà présent : laissé tel quel, ou aligné sur le fichier si le remplacement est demandé et change quelque chose
fn update_existing(
    tx: &Tx<'_>,
    existing: LocalLabel,
    (name, color, settings): (&str, Option<u32>, &PortableSettings),
    replace: bool,
) -> Result<LabelImportOutcome, MiraError> {
    if !replace {
        return Ok(LabelImportOutcome::AlreadyPresent);
    }
    let color = color.unwrap_or(existing.color);
    let (same_name, same_color) = (existing.name == name, existing.color == color);
    let settings_changed = merge_existing(tx, &existing.id, settings)?;
    if same_name && same_color && !settings_changed {
        return Ok(LabelImportOutcome::AlreadyPresent);
    }
    if !same_name {
        tx.rename_local_label(&existing.id, name)?;
    }
    if !same_color {
        tx.recolor_local_label(&existing.id, color)?;
    }
    Ok(LabelImportOutcome::Replaced)
}

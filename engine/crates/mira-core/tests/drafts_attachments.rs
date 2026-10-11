#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Pièces jointes d'un brouillon : fichier local, limite de 25 Mo, retrait.

mod support;
mod support_actions;
mod support_mirror;

use std::fs::File;

use mira_core::drafts::{MAX_ATTACHMENT_BYTES, attach_file, remove_attachment};
use mira_vocabulary::{DraftState, Timestamp};
use support_actions::drafts::{blank, write_file};
use support_actions::world::World;

/// Joindre un fichier garde son nom sans chemin, un type MIME deviné, sa taille et son contenu
#[test]
fn attach_stores_name_without_path_type_size_and_content() {
    let world = World::seeded();
    let draft = blank(&world);
    let dir = tempfile::tempdir().unwrap();
    let path = write_file(dir.path(), "Rapport.PDF", b"%PDF-1.4 contenu");
    let updated = attach_file(&world.ctx(), &draft.id, &path).unwrap();
    assert_eq!(updated.attachments.len(), 1);
    let attachment = &updated.attachments[0];
    assert_eq!(attachment.filename, "Rapport.PDF");
    assert_eq!(attachment.mime_type, "application/pdf");
    assert_eq!(attachment.size, 16);
    let data = world
        .db
        .read(|tx| tx.read_draft_attachment_data(&draft.id, 0))
        .unwrap();
    assert_eq!(data.unwrap(), b"%PDF-1.4 contenu");
    let other = write_file(dir.path(), "notes", b"x");
    let again = attach_file(&world.ctx(), &draft.id, &other).unwrap();
    assert_eq!(again.attachments[1].mime_type, "application/octet-stream");
}

/// Au-delà de 25 Mo : `invalid_input.parameter`, rien n'est joint ; 25 Mo pile passent
#[test]
fn attachment_over_25_mb_is_refused_and_exactly_25_mb_is_accepted() {
    let world = World::seeded();
    let draft = blank(&world);
    let dir = tempfile::tempdir().unwrap();
    let too_big = dir.path().join("gros.bin");
    File::create(&too_big)
        .unwrap()
        .set_len(MAX_ATTACHMENT_BYTES + 1)
        .unwrap();
    let error = attach_file(&world.ctx(), &draft.id, &too_big).unwrap_err();
    assert_eq!(error.code, "invalid_input.parameter");
    let stored = world
        .db
        .read(|tx| tx.get_draft(&draft.id))
        .unwrap()
        .unwrap();
    assert!(stored.attachments.is_empty());
    let limit = dir.path().join("limite.bin");
    File::create(&limit)
        .unwrap()
        .set_len(MAX_ATTACHMENT_BYTES)
        .unwrap();
    let accepted = attach_file(&world.ctx(), &draft.id, &limit).unwrap();
    assert_eq!(accepted.attachments[0].size, MAX_ATTACHMENT_BYTES);
}

/// Un fichier absent donne `local_storage.io` ; un dossier est refusé comme paramètre invalide
#[test]
fn missing_file_and_directory_are_refused() {
    let world = World::seeded();
    let draft = blank(&world);
    let dir = tempfile::tempdir().unwrap();
    let missing = attach_file(&world.ctx(), &draft.id, &dir.path().join("absent.txt")).unwrap_err();
    assert_eq!(missing.code, "local_storage.io");
    let directory = attach_file(&world.ctx(), &draft.id, dir.path()).unwrap_err();
    assert_eq!(directory.code, "invalid_input.parameter");
    assert!(!missing.message.contains(dir.path().to_str().unwrap()));
}

/// Retirer une pièce jointe par son rang garde les autres dans l'ordre ; un rang inconnu est refusé
#[test]
fn remove_attachment_by_index_keeps_the_others_in_order() {
    let world = World::seeded();
    let draft = blank(&world);
    let dir = tempfile::tempdir().unwrap();
    for name in ["a.txt", "b.txt", "c.txt"] {
        let path = write_file(dir.path(), name, name.as_bytes());
        attach_file(&world.ctx(), &draft.id, &path).unwrap();
    }
    let after_first = remove_attachment(&world.ctx(), &draft.id, 1).unwrap();
    let names: Vec<&str> = after_first
        .attachments
        .iter()
        .map(|a| a.filename.as_str())
        .collect();
    assert_eq!(names, ["a.txt", "c.txt"]);
    let after_second = remove_attachment(&world.ctx(), &draft.id, 1).unwrap();
    assert_eq!(after_second.attachments.len(), 1);
    let error = remove_attachment(&world.ctx(), &draft.id, 5).unwrap_err();
    assert_eq!(error.code, "invalid_input.parameter");
}

/// Pas de pièce jointe sur un brouillon planifié
#[test]
fn attachments_cannot_change_on_a_scheduled_draft() {
    let world = World::seeded();
    let mut draft = blank(&world);
    draft.state = DraftState::Scheduled {
        at: Timestamp::from_millis(1),
    };
    world.db.write(|tx| tx.update_draft(&draft)).unwrap();
    let dir = tempfile::tempdir().unwrap();
    let path = write_file(dir.path(), "a.txt", b"a");
    let error = attach_file(&world.ctx(), &draft.id, &path).unwrap_err();
    assert_eq!(error.code, "incompatible_state.draft_not_editable");
    let error = remove_attachment(&world.ctx(), &draft.id, 0).unwrap_err();
    assert_eq!(error.code, "incompatible_state.draft_not_editable");
}

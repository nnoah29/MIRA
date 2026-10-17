#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Pièce jointe : fichiers spéciaux refusés sans blocage, lien symbolique vers un fichier ordinaire accepté.

mod support;
mod support_actions;
mod support_mirror;

use std::process::Command;

use mira_core::drafts::attach_file;
use support_actions::drafts::{blank, write_file};
use support_actions::world::World;

/// Un tube nommé est refusé tout de suite, sans jamais être ouvert (la lecture ne se bloque pas)
#[test]
fn regression_named_pipe_is_refused_without_blocking() {
    let world = World::seeded();
    let draft = blank(&world);
    let dir = tempfile::tempdir().unwrap();
    let pipe = dir.path().join("tube");
    let status = Command::new("mkfifo").arg(&pipe).status().unwrap();
    assert!(status.success());
    let error = attach_file(&world.ctx(), &draft.id, &pipe).unwrap_err();
    assert_eq!(error.code, "invalid_input.parameter");
}

/// Un lien symbolique vers un fichier ordinaire est joint sous le nom du lien
#[test]
fn symlink_to_a_regular_file_is_attached_under_the_link_name() {
    let world = World::seeded();
    let draft = blank(&world);
    let dir = tempfile::tempdir().unwrap();
    let target = write_file(dir.path(), "cible.txt", b"contenu");
    let link = dir.path().join("lien.txt");
    std::os::unix::fs::symlink(&target, &link).unwrap();
    let updated = attach_file(&world.ctx(), &draft.id, &link).unwrap();
    assert_eq!(updated.attachments[0].filename, "lien.txt");
    assert_eq!(updated.attachments[0].size, 7);
}

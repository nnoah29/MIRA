//! Brouillons de test : neuf, prêt à envoyer, relu en base, fichier à joindre.

use std::path::{Path, PathBuf};

use mira_core::drafts::{DraftEdit, create_blank, edit_draft};
use mira_vocabulary::{AccountId, Draft};

use super::mail::entry;
use super::world::World;

/// Brouillon neuf d'acc-1
pub fn blank(world: &World) -> Draft {
    create_blank(&world.ctx(), &AccountId::from_raw("acc-1"), None).unwrap()
}

/// Brouillon d'acc-1 avec un destinataire
pub fn ready_draft(world: &World) -> Draft {
    let draft = blank(world);
    let edit = DraftEdit {
        to: Some(vec![entry("bob@example.org")]),
        ..DraftEdit::default()
    };
    edit_draft(&world.ctx(), &draft.id, edit).unwrap()
}

/// Relit le brouillon en base
pub fn stored(world: &World, draft: &Draft) -> Draft {
    world
        .db
        .read(|tx| tx.get_draft(&draft.id))
        .unwrap()
        .unwrap()
}

/// Écrit un fichier de test dans le dossier donné et rend son chemin
pub fn write_file(dir: &Path, name: &str, content: &[u8]) -> PathBuf {
    let path = dir.join(name);
    std::fs::write(&path, content).unwrap();
    path
}

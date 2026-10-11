//! Modification des drapeaux d'un message.

use super::data::Mailbox;

/// Manière de combiner les drapeaux demandés avec les drapeaux existants.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum FlagMode {
    Add,
    Remove,
    Replace,
}

/// Convertit des drapeaux en chaînes possédées
pub(crate) fn owned(flags: &[&str]) -> Vec<String> {
    flags.iter().map(|flag| (*flag).to_string()).collect()
}

/// Nouveaux drapeaux selon le mode
fn combine(current: &[String], mode: FlagMode, wanted: &[String]) -> Vec<String> {
    let mut flags: Vec<String> = match mode {
        FlagMode::Replace => Vec::new(),
        _ => current.to_vec(),
    };
    for flag in wanted {
        let position = flags
            .iter()
            .position(|known| known.eq_ignore_ascii_case(flag));
        match (mode, position) {
            (FlagMode::Remove, Some(index)) => {
                flags.remove(index);
            }
            (FlagMode::Remove, None) | (_, Some(_)) => {}
            (_, None) => flags.push(flag.clone()),
        }
    }
    flags
}

impl Mailbox {
    /// Applique la modification ; renvoie le nouveau MODSEQ si les drapeaux ont changé
    pub(crate) fn change_flags(
        &mut self,
        uid: u32,
        mode: FlagMode,
        wanted: &[String],
    ) -> Option<u64> {
        let current = self
            .entries
            .iter()
            .find(|entry| entry.uid == uid)?
            .flags
            .clone();
        let mut next = combine(&current, mode, wanted);
        let mut sorted_current = current;
        sorted_current.sort();
        next.sort();
        if next == sorted_current {
            return None;
        }
        let modseq = self.bump_modseq();
        let entry = self.entry_mut(uid)?;
        entry.flags = next;
        entry.modseq = modseq;
        Some(modseq)
    }
}

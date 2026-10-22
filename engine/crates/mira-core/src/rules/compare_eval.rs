//! Conditions numériques : nombre de destinataires et taille.

use super::context::MessageContext;
use mira_vocabulary::Comparison;
use std::cmp::Ordering;

/// Le nombre total de destinataires (À, Cc, Cci) se compare à `n`.
pub fn recipient_count(ctx: &MessageContext, cmp: Comparison, n: u32) -> bool {
    let summary = &ctx.summary;
    let total = summary.to.len() + summary.cc.len() + summary.bcc.len();
    compare(cmp, total as u64, u64::from(n))
}

/// La taille du message en octets se compare à `bytes`.
pub fn size(ctx: &MessageContext, cmp: Comparison, bytes: u64) -> bool {
    compare(cmp, ctx.summary.size, bytes)
}

/// Applique le sens de comparaison à deux entiers.
fn compare(cmp: Comparison, left: u64, right: u64) -> bool {
    let wanted = match cmp {
        Comparison::Less => Ordering::Less,
        Comparison::Equal => Ordering::Equal,
        Comparison::Greater => Ordering::Greater,
    };
    left.cmp(&right) == wanted
}

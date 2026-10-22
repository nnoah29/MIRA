//! Conditions « moi en À » et « moi en Cc seulement ».

use super::context::MessageContext;
use mira_vocabulary::AddressEntry;

/// Une adresse du compte figure parmi les destinataires À.
pub fn me_in_to(ctx: &MessageContext) -> bool {
    lists_own_address(ctx, &ctx.summary.to)
}

/// Une adresse du compte figure en Cc et aucune en À.
pub fn me_in_cc_only(ctx: &MessageContext) -> bool {
    lists_own_address(ctx, &ctx.summary.cc) && !me_in_to(ctx)
}

/// Vrai si une adresse du compte figure dans la liste, casse ignorée.
fn lists_own_address(ctx: &MessageContext, entries: &[AddressEntry]) -> bool {
    entries.iter().any(|entry| {
        ctx.own_addresses
            .iter()
            .any(|own| own.as_str().eq_ignore_ascii_case(entry.address.trim()))
    })
}

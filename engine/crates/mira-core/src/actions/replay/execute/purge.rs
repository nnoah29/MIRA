//! Capacités effectives de la session et suppression ciblée de messages de la boîte sélectionnée.

use mira_vocabulary::{MiraError, ProviderQuirks};

use crate::capabilities::{
    EffectiveCapabilities, ExpungeMethod, choose_expunge, parse_capabilities,
};
use crate::imap::{ImapClient, ImapFlag};

/// Capacités sur lesquelles on peut compter : annonce du serveur moins les exceptions du service
pub(crate) fn effective(client: &ImapClient, quirks: &ProviderQuirks) -> EffectiveCapabilities {
    EffectiveCapabilities::compute(&parse_capabilities(client.capabilities()), quirks)
}

/// Marque les messages `\Deleted` puis les purge par UID ; sans UIDPLUS, ils restent marqués (10.14)
pub(crate) async fn purge_selected(
    client: &mut ImapClient,
    capabilities: &EffectiveCapabilities,
    uids: &[u32],
) -> Result<(), MiraError> {
    if uids.is_empty() {
        return Ok(());
    }
    client.add_flags(uids, &[ImapFlag::Deleted]).await?;
    match choose_expunge(capabilities) {
        ExpungeMethod::UidExpunge => client.expunge_uids(uids).await,
        ExpungeMethod::NoPurge => Ok(()),
    }
}

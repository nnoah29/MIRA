//! Démarrage de l'écoute IDLE sur une seconde connexion, ou repli sur le relevé périodique (10.14).

use mira_vocabulary::{LogLevel, MailboxRole};

use super::runtime::AccountRuntime;
use crate::capabilities::{EffectiveCapabilities, Realtime, choose_realtime};
use crate::sync::realtime::{IdleWatcher, start_idle_watcher};
use crate::sync::session::open_connection;

/// Journalise le repli sur le relevé périodique et ne rend aucune écoute.
fn fall_back_to_polling(runtime: &AccountRuntime, reason: &str) -> Option<IdleWatcher> {
    runtime
        .ctx
        .diagnose(LogLevel::Info, "sync.realtime", reason);
    None
}

/// Nom de la réception, une fois connue du miroir.
fn inbox_name(runtime: &AccountRuntime) -> Option<String> {
    let account = runtime.ctx.account.id.clone();
    let found = runtime
        .ctx
        .db
        .read(|tx| tx.find_mailbox_by_role(&account, MailboxRole::Inbox))
        .ok()?;
    found.map(|inbox| inbox.name)
}

/// Ouvre une seconde connexion en IDLE sur la réception, si le serveur et le budget le permettent.
pub async fn start_idle(
    runtime: &AccountRuntime,
    capabilities: &EffectiveCapabilities,
) -> Option<IdleWatcher> {
    if choose_realtime(capabilities) != Realtime::Idle {
        return fall_back_to_polling(runtime, "IDLE indisponible : relevé périodique");
    }
    let spare_left = runtime.budget.available() >= 2;
    let permit = runtime.budget.try_acquire().filter(|_| spare_left);
    let Some(permit) = permit else {
        return fall_back_to_polling(runtime, "budget de connexions : relevé périodique");
    };
    let inbox = inbox_name(runtime)?;
    let credentials = runtime.credentials.as_ref();
    let connection = open_connection(&runtime.ctx, permit, credentials)
        .await
        .ok()?;
    start_idle_watcher(connection, &inbox, runtime.config.idle_renewal)
        .await
        .ok()
}

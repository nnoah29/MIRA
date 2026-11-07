//! Traitement d'un lot d'arrivées : décision pour chacune, lecture des en-têtes, composition par compte, remise au récepteur.

use std::collections::BTreeMap;
use std::ops::Deref;
use std::sync::Arc;

use mira_assembly::mira_vocabulary::EngineSettings;

use super::pending::Pending;
use super::{Arrival, Decision, LocalClock, NotificationSink, Texts, compose, decide, gather};
use crate::Host;

/// Référence partageable vers un hôte : `Arc<Host>` (démon) ou `&'static Host` (pont).
pub trait HostRef: Deref<Target = Host> + Send + Sync + 'static {}

impl<T: Deref<Target = Host> + Send + Sync + 'static> HostRef for T {}

/// Ce qu'il faut pour notifier : l'hôte à lire, le récepteur, l'horloge locale et les textes.
pub(super) struct Notifier<H: HostRef> {
    pub host: H,
    pub sink: Arc<dyn NotificationSink>,
    pub clock: Arc<dyn LocalClock>,
    pub texts: Texts,
}

impl<H: HostRef> Notifier<H> {
    /// Notifie le lot : réglages relus à ce moment, arrivées retenues regroupées par compte.
    pub(super) async fn flush(&self, batch: Vec<Pending>) {
        let Some(settings) = gather::settings(&self.host).await else {
            return;
        };
        let groups = self.kept(&settings, batch).await;
        for arrivals in groups.into_values() {
            for notification in compose(&arrivals, &settings.notifications, &self.texts) {
                self.sink.show(notification);
            }
        }
    }

    /// Arrivées à notifier, regroupées par compte, dans l'ordre d'arrivée.
    async fn kept(
        &self,
        settings: &EngineSettings,
        batch: Vec<Pending>,
    ) -> BTreeMap<String, Vec<Arrival>> {
        let active = self.host.presence().is_active();
        let hour = self.clock.hour();
        let mut groups: BTreeMap<String, Vec<Arrival>> = BTreeMap::new();
        for pending in batch {
            let verdict = decide(settings, pending.silenced, &pending.account, active, hour);
            if verdict == Decision::Skip {
                continue;
            }
            if let Some(arrival) = gather::arrival(&self.host, &pending).await {
                groups
                    .entry(arrival.account.clone())
                    .or_default()
                    .push(arrival);
            }
        }
        groups
    }
}

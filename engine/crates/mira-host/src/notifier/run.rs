//! Boucle du notifieur : écoute les arrivées, les regroupe sur une courte fenêtre, puis les notifie.

use std::sync::Arc;
use std::time::Duration;

use mira_assembly::events::EventSubscription;
use mira_assembly::mira_vocabulary::EventFilter;
use tokio::time::{Instant, timeout_at};

use super::flush::{HostRef, Notifier};
use super::pending::Pending;
use super::{LocalClock, NotificationSink, Texts};
use crate::events_feed::latest_seq;

/// Prochaine arrivée annoncée, ou rien quand le moteur se ferme.
async fn next_pending(subscription: &mut EventSubscription) -> Option<Pending> {
    while let Some(event) = subscription.next().await {
        if let Some(pending) = Pending::from_event(&event) {
            return Some(pending);
        }
    }
    None
}

/// Ajoute au lot les arrivées qui suivent pendant `window`.
async fn gather_window(
    subscription: &mut EventSubscription,
    batch: &mut Vec<Pending>,
    window: Duration,
) {
    let deadline = Instant::now() + window;
    while let Ok(Some(event)) = timeout_at(deadline, subscription.next()).await {
        batch.extend(Pending::from_event(&event));
    }
}

/// Écoute les événements du moteur à partir de maintenant et remet les notifications au récepteur, jusqu'à la fermeture du moteur.
pub async fn run_notifier<H: HostRef>(
    host: H,
    sink: Arc<dyn NotificationSink>,
    clock: Arc<dyn LocalClock>,
    texts: Texts,
    window: Duration,
) {
    let Some(engine) = host.engine() else { return };
    let filter = EventFilter {
        after: latest_seq(&engine).await,
        ..EventFilter::default()
    };
    let mut subscription = engine.subscribe(filter);
    drop(engine);
    let notifier = Notifier {
        host,
        sink,
        clock,
        texts,
    };
    while let Some(first) = next_pending(&mut subscription).await {
        let mut batch = vec![first];
        gather_window(&mut subscription, &mut batch, window).await;
        notifier.flush(batch).await;
    }
}

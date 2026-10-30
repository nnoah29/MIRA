//! Échange réseau d'une génération : envoi, lecture des morceaux, décodage SSE et transmission.

use mira_vocabulary::WireFormat;

use super::ending::Ending;
use super::guard::Guard;
use super::prepare::ChatPlan;
use super::progress::Progress;
use crate::ai::sse::{SseEvent, SseReader};
use crate::ai::transport::{AiStreamBody, TransportLimits, post_stream};
use crate::ai::wire::dispatch::{decode_event, encode};

/// Envoie la requête et suit la réponse jusqu'à sa fin, son échec ou son interruption.
pub(super) async fn converse(
    plan: &ChatPlan,
    guard: &Guard<'_>,
    progress: &mut Progress,
) -> Ending {
    let call = encode(&plan.request, &plan.model, &plan.descriptor);
    let limits = TransportLimits::default();
    let opening = post_stream(&plan.base_url, &plan.descriptor, &plan.key, &call, &limits);
    match guard.wait(opening).await {
        Err(interrupt) => interrupt.into(),
        Ok(Err(error)) => Ending::Failed(error),
        Ok(Ok(body)) => pump(body, plan.descriptor.format, guard, progress).await,
    }
}

/// Lit le flux jusqu'au bout ; rendre la main détruit le corps, ce qui ferme la connexion.
async fn pump(
    mut body: AiStreamBody,
    format: WireFormat,
    guard: &Guard<'_>,
    progress: &mut Progress,
) -> Ending {
    let mut reader = SseReader::new();
    loop {
        let chunk = match guard.wait(body.next_chunk()).await {
            Err(interrupt) => return interrupt.into(),
            Ok(Err(error)) => return Ending::Failed(error),
            Ok(Ok(None)) => break,
            Ok(Ok(Some(chunk))) => chunk,
        };
        if let Err(ending) = forward(reader.push(&chunk), format, guard, progress).await {
            return ending;
        }
    }
    drop(body);
    match forward(reader.finish(), format, guard, progress).await {
        Ok(()) => progress.end_of_stream(),
        Err(ending) => ending,
    }
}

/// Décode les événements, retient l'usage et l'arrêt, transmet le texte au lecteur.
async fn forward(
    events: Vec<SseEvent>,
    format: WireFormat,
    guard: &Guard<'_>,
    progress: &mut Progress,
) -> Result<(), Ending> {
    for event in events {
        for delta in decode_event(format, &event).map_err(Ending::Failed)? {
            for delta in progress.absorb(delta) {
                guard.send(delta).await?;
            }
        }
    }
    Ok(())
}

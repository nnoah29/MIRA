//! Tours de génération : le modèle répond, demande des outils, le moteur les exécute, jusqu'à huit tours.

use mira_vocabulary::StopReason;

use super::ending::Ending;
use super::guard::Guard;
use super::prepare::ChatPlan;
use super::progress::Progress;
use super::stream::converse;
use super::tool_step::run_tools;

const MAX_ROUNDS: usize = 8;

/// Mène les tours ; un huitième tour qui demande encore des outils termine sans les exécuter.
pub(super) async fn run_rounds(
    plan: &mut ChatPlan,
    guard: &Guard<'_>,
    progress: &mut Progress,
) -> Ending {
    for round in 1..=MAX_ROUNDS {
        let ending = converse(plan, guard, progress).await;
        let Ending::Finished(StopReason::ToolUse) = ending else {
            return ending;
        };
        let calls = progress.take_calls();
        if calls.is_empty() || plan.request.tools.is_empty() {
            return Ending::Finished(StopReason::EndTurn);
        }
        if round == MAX_ROUNDS {
            return Ending::Finished(StopReason::MaxToolRounds);
        }
        if let Err(ending) = run_tools(plan, guard, progress, calls).await {
            return ending;
        }
        progress.close_round();
    }
    Ending::Finished(StopReason::MaxToolRounds)
}

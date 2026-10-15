//! Page Maintenance : volumes du miroir, dernier compte rendu et rappel des opérations.

use mira_assembly::storage::repositories::maintenance::StorageStats;
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::text::Line;
use ratatui::widgets::Paragraph;

use crate::state::AppState;
use crate::theme;
use crate::view::safe_text::display_safe;

/// Volumes du miroir, un par ligne (mêmes libellés que `mira maintenance stats`).
fn stats_lines(stats: &StorageStats) -> Vec<Line<'static>> {
    vec![
        Line::from(format!("Boîtes         : {}", stats.mailboxes)),
        Line::from(format!("Messages       : {}", stats.messages)),
        Line::from(format!("Fils           : {}", stats.threads)),
        Line::from(format!(
            "Sources brutes : {} ({} octets, {} compressés)",
            stats.raw_sources, stats.raw_bytes, stats.compressed_bytes
        )),
        Line::from(format!(
            "Pièces jointes : {} ({} en cache)",
            stats.attachments, stats.attachments_cached
        )),
    ]
}

/// Dessine les volumes (ou « chargement »), le dernier compte rendu et les touches de la page.
pub fn render_maintenance(frame: &mut Frame, area: Rect, state: &AppState) {
    let maintenance = &state.settings.maintenance;
    let mut lines = match &maintenance.stats {
        Some(stats) => stats_lines(stats),
        None => vec![Line::from("Chargement des volumes…")],
    };
    lines.push(Line::from(""));
    if let Some(report) = &maintenance.report {
        lines.push(Line::styled(display_safe(report), theme::unread()));
        lines.push(Line::from(""));
    }
    lines.push(Line::from(
        "s volumes · c intégrité · p vider le cache de pièces jointes",
    ));
    lines.push(Line::from(
        "i reconstruire l'index · d exporter un diagnostic · x exporter le profil",
    ));
    frame.render_widget(Paragraph::new(lines), area);
}

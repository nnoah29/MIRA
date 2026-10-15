//! Résumé de la confirmation d'envoi : total, un résumé par champ (coupes signalées), objet raccourci, touches.

use mira_assembly::mira_vocabulary::AddressEntry;

use crate::state::compose::{ComposeState, Field};
use crate::view::safe_text::display_safe;

const ADDRESS_CHARS: usize = 40;
const SUBJECT_CHARS: usize = 50;
const KEYS: &str = "Entrée ou y : envoyer · Échap ou n : annuler";

/// Raccourcit à `max` caractères, la coupe marquée par « … ».
fn ellipsize(text: &str, max: usize) -> String {
    if text.chars().count() <= max {
        return text.to_string();
    }
    let kept: String = text.chars().take(max.saturating_sub(1)).collect();
    format!("{kept}…")
}

/// « À : a, b (+N autres) » : les adresses qui tiennent, neutralisées, puis le nombre de celles qui ne sont pas montrées.
fn listed(label: &str, entries: &[AddressEntry]) -> Option<String> {
    let first = entries.first()?;
    let mut shown = vec![ellipsize(&display_safe(&first.address), ADDRESS_CHARS)];
    let mut used = shown[0].chars().count();
    for entry in entries.iter().skip(1) {
        let address = display_safe(&entry.address);
        used += 2 + address.chars().count();
        if used > ADDRESS_CHARS {
            break;
        }
        shown.push(address);
    }
    let mut line = format!("{label} : {}", shown.join(", "));
    let rest = entries.len() - shown.len();
    if rest > 0 {
        line.push_str(&format!(" (+{rest} autres)"));
    }
    Some(line)
}

/// Lignes de la confirmation, chacune avec son rang de sacrifice (le plus grand disparaît d'abord sur un écran bas).
fn ranked_summary(compose: &ComposeState, from: &str) -> Vec<(u8, String)> {
    let to = compose.recipients(Field::To);
    let cc = compose.recipients(Field::Cc);
    let bcc = compose.recipients(Field::Bcc);
    let total = to.len() + cc.len() + bcc.len();
    let mut lines = vec![(3, format!("Envoyer ce message à {total} destinataire(s) ?"))];
    lines.push((4, format!("De : {}", ellipsize(from, ADDRESS_CHARS))));
    lines.extend(listed(Field::To.label(), &to).map(|line| (5, line)));
    lines.extend(listed(Field::Cc.label(), &cc).map(|line| (6, line)));
    if !bcc.is_empty() {
        let label = Field::Bcc.label();
        lines.push((7, format!("{label} : {} destinataire(s)", bcc.len())));
    }
    let subject = display_safe(&compose.text(Field::Subject));
    lines.push((2, format!("Objet : {}", ellipsize(&subject, SUBJECT_CHARS))));
    lines.push((1, KEYS.to_string()));
    lines
}

/// Toutes les lignes de la confirmation, chacune assez courte pour la fenêtre : rien n'est coupé sans être signalé.
pub fn send_summary(compose: &ComposeState, from: &str) -> Vec<String> {
    ranked_summary(compose, from)
        .into_iter()
        .map(|(_, line)| line)
        .collect()
}

/// Lignes qui tiennent en `rows` : destinataires détaillés puis en-tête sacrifiés avant l'objet et les touches.
pub fn fit_summary(compose: &ComposeState, from: &str, rows: usize) -> Vec<String> {
    let mut lines = ranked_summary(compose, from);
    while lines.len() > rows {
        let Some(drop) = (0..lines.len()).max_by_key(|&index| lines[index].0) else {
            break;
        };
        lines.remove(drop);
    }
    lines.into_iter().map(|(_, line)| line).collect()
}

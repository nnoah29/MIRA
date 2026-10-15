//! Fenêtre d'ajout de compte : en-tête du moteur, champs, erreur ; elle suit le champ qui a le focus si l'écran est bas.

use ratatui::Frame;
use ratatui::layout::{Position, Rect};
use ratatui::text::Line;
use ratatui::widgets::{Block, Clear, Paragraph};

use super::account_form_info::{footer_lines, header_lines};
use super::account_form_rows::{LABEL_WIDTH, field_row, security_label};
use super::centered;
use crate::state::AppState;
use crate::state::account_form::{AccountForm, FormField};
use crate::theme;
use crate::view::layout;

const WIDTH: u16 = 80;

/// Lignes de la fenêtre et position (ligne, colonne) du curseur dans ces lignes.
struct Content {
    lines: Vec<Line<'static>>,
    cursor: Option<(usize, usize)>,
}

/// Chiffrement à rappeler sous ce champ, pour les ports des serveurs saisis.
fn security_line(form: &AccountForm, field: FormField) -> Option<Line<'static>> {
    let security = match field {
        FormField::ImapPort => form.imap_security,
        FormField::SmtpPort => form.smtp_security,
        FormField::Address
        | FormField::Name
        | FormField::Method
        | FormField::ImapHost
        | FormField::SmtpHost
        | FormField::Password => return None,
    };
    let text = format!("{:<LABEL_WIDTH$}{}", "", security_label(security));
    Some(Line::styled(text, theme::dim()))
}

/// Construit toutes les lignes : en-tête, champs visibles, pied.
fn content(form: &AccountForm, width: usize) -> Content {
    let mut lines = header_lines(form);
    lines.push(Line::from(""));
    let mut cursor = None;
    for field in form.visible_fields() {
        let row = field_row(form, field, width);
        if let Some(column) = row.cursor {
            cursor = Some((lines.len(), column));
        }
        lines.push(row.line);
        lines.extend(security_line(form, field));
    }
    let footer = footer_lines(form);
    if !footer.is_empty() {
        lines.push(Line::from(""));
        lines.extend(footer);
    }
    Content { lines, cursor }
}

/// Dessine la fenêtre d'ajout de compte au-dessus de l'écran, au-dessus de la barre d'état.
pub fn render_account_form(frame: &mut Frame, state: &AppState) {
    let Some(form) = state.account_form.form() else {
        return;
    };
    let body = layout::body(frame.area());
    let width = body.width.min(WIDTH);
    let content = content(form, usize::from(width.saturating_sub(2)));
    let rows = u16::try_from(content.lines.len()).unwrap_or(u16::MAX);
    let area = centered(body, width, rows.saturating_add(2));
    frame.render_widget(Clear, area);
    let block = Block::bordered()
        .title("Ajouter un compte")
        .border_style(theme::border(true));
    let inner = block.inner(area);
    frame.render_widget(block, area);
    draw_content(frame, inner, content);
}

/// Dessine les lignes qui tiennent, en gardant visible la ligne du curseur, puis place le curseur.
fn draw_content(frame: &mut Frame, inner: Rect, content: Content) {
    let height = usize::from(inner.height);
    let focus_row = content.cursor.map_or(0, |(row, _)| row);
    let skip = (focus_row + 1).saturating_sub(height);
    let visible: Vec<Line<'static>> = content.lines.into_iter().skip(skip).take(height).collect();
    frame.render_widget(Paragraph::new(visible), inner);
    let Some((row, column)) = content.cursor else {
        return;
    };
    if inner.width == 0 || inner.height == 0 {
        return;
    }
    let x = u16::try_from(column)
        .unwrap_or(u16::MAX)
        .min(inner.width - 1);
    let y = u16::try_from(row - skip)
        .unwrap_or(u16::MAX)
        .min(inner.height - 1);
    frame.set_cursor_position(Position::new(inner.x + x, inner.y + y));
}

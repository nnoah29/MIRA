//! Fenêtre verticale du corps : lignes visibles et première ligne qui garde le curseur à l'écran.

/// Lignes du haut occupées par l'expéditeur et les champs d'en-tête (De, À, Cc, Cci, Objet).
pub const HEADER_ROWS: u16 = 5;

/// Première ligne à afficher pour que le curseur reste dans les `rows` lignes visibles.
pub fn visible_top(top: usize, cursor_line: usize, rows: usize) -> usize {
    let rows = rows.max(1);
    if cursor_line < top {
        cursor_line
    } else if cursor_line >= top + rows {
        cursor_line + 1 - rows
    } else {
        top
    }
}

/// Lignes de texte du corps pour un terminal de cette hauteur (barre d'état, en-têtes et bordures retirés).
pub fn body_rows(height: u16) -> usize {
    let chrome = 1 + HEADER_ROWS + 2;
    usize::from(height.saturating_sub(chrome)).max(1)
}

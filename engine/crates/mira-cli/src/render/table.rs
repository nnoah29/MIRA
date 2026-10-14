//! Tableau texte simple : colonnes alignées à gauche, séparées par deux espaces.

/// Largeur de chaque colonne : le plus long entre l'en-tête et les cellules.
fn column_widths(headers: &[&str], rows: &[Vec<String>]) -> Vec<usize> {
    let mut widths: Vec<usize> = headers
        .iter()
        .map(|header| header.chars().count())
        .collect();
    for row in rows {
        for (index, cell) in row.iter().enumerate() {
            if let Some(width) = widths.get_mut(index) {
                *width = (*width).max(cell.chars().count());
            }
        }
    }
    widths
}

/// Écrit une ligne alignée, sans espace en fin de ligne.
fn format_line(cells: &[String], widths: &[usize]) -> String {
    let padded: Vec<String> = cells
        .iter()
        .zip(widths)
        .map(|(cell, width)| format!("{cell:<width$}"))
        .collect();
    padded.join("  ").trim_end().to_string()
}

/// Rend un tableau avec en-tête ; sans ligne, le texte « (aucun) ».
pub fn render_table(headers: &[&str], rows: &[Vec<String>]) -> String {
    if rows.is_empty() {
        return "(aucun)".to_string();
    }
    let widths = column_widths(headers, rows);
    let titles: Vec<String> = headers.iter().map(ToString::to_string).collect();
    let mut lines = vec![format_line(&titles, &widths)];
    lines.extend(rows.iter().map(|row| format_line(row, &widths)));
    lines.join("\n")
}

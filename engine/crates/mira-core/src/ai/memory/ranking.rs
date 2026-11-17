//! Similarité cosinus et classement des souvenirs les plus proches d'une requête.

/// Similarité cosinus ; `None` si les dimensions diffèrent ou si un vecteur est de norme nulle
pub fn cosine(a: &[f32], b: &[f32]) -> Option<f32> {
    if a.len() != b.len() {
        return None;
    }
    let dot: f32 = a.iter().zip(b).map(|(x, y)| x * y).sum();
    let norm_a = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm_a == 0.0 || norm_b == 0.0 {
        return None;
    }
    Some(dot / (norm_a * norm_b))
}

/// Les `k` plus proches dont le score atteint `min_score`, du meilleur au moins bon ; égalité : identifiant croissant
pub fn top_k(
    query: &[f32],
    items: Vec<(String, Vec<f32>)>,
    k: usize,
    min_score: f32,
) -> Vec<(String, f32)> {
    let mut scored: Vec<(String, f32)> = items
        .into_iter()
        .filter_map(|(id, vector)| cosine(query, &vector).map(|score| (id, score)))
        .filter(|(_, score)| *score >= min_score)
        .collect();
    scored.sort_by(|a, b| b.1.total_cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    scored.truncate(k);
    scored
}

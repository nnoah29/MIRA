//! Nettoyage du texte reconnu avant son insertion dans un champ.

/// Normalise une phrase reconnue : apostrophes droites, blancs de tête et de fin retirés, blancs multiples réduits.
String normalizeTranscript(String raw) =>
    raw.replaceAll('’', "'").replaceAll(RegExp(r'\s+'), ' ').trim();

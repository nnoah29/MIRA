//! Détection d'un texte qui annonce une pièce jointe (français et anglais), pour avertir avant l'envoi si aucune n'est jointe.

const _start = r'(?<![\p{L}\p{N}])';
const _end = r'(?![\p{L}\p{N}])';

const _phrases = [
  r'pi[eè]ces?\s+jointes?', // i18n:ignore
  r'ci[\s-]?joint(?:e|s|es)?', // i18n:ignore
  r'ci[\s-]?attach[ée](?:e|s|es)?', // i18n:ignore
  r'(?:je|nous)\s+(?:(?:vous|te|t’|t\x27)\s*)?(?:joins|joignons)', // i18n:ignore
  r'j[’\x27]ai\s+joint', // i18n:ignore
  r'en\s+annexe', // i18n:ignore
  r'annex[ée](?:e|s|es)?', // i18n:ignore
  r'p\.?\s?j\.?', // i18n:ignore
  r'attach(?:ed|ments?)', // i18n:ignore
  r'enclosed', // i18n:ignore
];

final _mention = RegExp(
  '$_start(?:${_phrases.join('|')})$_end',
  caseSensitive: false,
  unicode: true,
);

/// Vrai si le texte parle d'une pièce jointe (« ci-joint », « pièce jointe », « attached »…).
bool mentionsAttachment(String text) => _mention.hasMatch(text);

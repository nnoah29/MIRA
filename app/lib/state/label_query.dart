import '../data/models.dart';

/// Libellé visé par `@:nom` en tête de la saisie (`key` nul s'il n'y en a pas) et texte qui reste à chercher.
typedef LabelQuery = ({String? key, String rest});

const _accents = {
  'à': 'a', 'â': 'a', 'ä': 'a', 'ç': 'c', 'é': 'e', 'è': 'e', 'ê': 'e',
  'ë': 'e', 'î': 'i', 'ï': 'i', 'ô': 'o', 'ö': 'o', 'ù': 'u', 'û': 'u',
  'ü': 'u', 'ÿ': 'y', //
};

/// Minuscules sans accent, caractère pour caractère : les positions restent celles du texte d'origine.
String _fold(String s) =>
    s.toLowerCase().split('').map((c) => _accents[c] ?? c).join();

/// Reconnaît `@:nom` (espaces permis dans le nom, casse et accents ignorés) ; un préfixe unique suffit.
LabelQuery parseLabelQuery(String raw, Map<String, LabelDef> defs) {
  final text = raw.trimLeft();
  if (!text.startsWith('@:')) return (key: null, rest: raw);
  final body = text.substring(2);
  final folded = _fold(body);
  String? best;
  var bestLen = 0;
  for (final e in defs.entries) {
    final name = _fold(e.value.n);
    if (!folded.startsWith(name) || name.length <= bestLen) continue;
    if (body.length == name.length || body[name.length] == ' ') {
      best = e.key;
      bestLen = name.length;
    }
  }
  if (best != null) {
    return (key: best, rest: body.substring(bestLen).trim());
  }
  final typed = folded.trim();
  final byPrefix = [
    for (final e in defs.entries)
      if (typed.isNotEmpty && _fold(e.value.n).startsWith(typed)) e.key,
  ];
  return byPrefix.length == 1
      ? (key: byPrefix.single, rest: '')
      : (key: null, rest: raw);
}

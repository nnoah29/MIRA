//! Messages cités par l'IA dans sa réponse : le repère `[mail:ID]` quitte le texte, l'identifiant devient une carte.

/// Texte affiché sans les repères, et identifiants cités dans l'ordre de leur première mention.
class Mentions {
  final String text;
  final List<String> ids;
  const Mentions(this.text, this.ids);
}

final _marker = RegExp(r'[ \t]*\[mail:([^\]\s]+)\]');
final _partial = RegExp(r'[ \t]*\[(m(a(i(l(:[^\]\s]*)?)?)?)?)?$');

/// Retire de [raw] les repères complets, et un repère commencé à la fin (flux en cours) ; rend les identifiants cités.
Mentions readMentions(String raw) {
  final ids = <String>[];
  final text = raw.replaceAllMapped(_marker, (m) {
    final id = m.group(1)!;
    if (!ids.contains(id)) ids.add(id);
    return '';
  });
  return Mentions(text.replaceFirst(_partial, ''), ids);
}

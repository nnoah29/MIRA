//! Réponse de l'IA rendue en simple texte : gras, italique, code en ligne, listes ; jamais de HTML ni de lien actif.

import 'package:flutter/material.dart';

import '../theme/app_theme.dart';

final _inline = RegExp(
  r'\*\*(?<b>[^*\n]+?)\*\*|`(?<c>[^`\n]+)`|\*(?<i>[^\s*][^*\n]*?)\*|(?<![\w])_(?<u>[^\s_][^_\n]*?)_(?![\w])',
);
final _bullet = RegExp(r'^(\s*)[-*•]\s+(.*)$');
final _numbered = RegExp(r'^(\s*)(\d{1,3})[.)]\s+(.*)$');

/// Texte d'une ligne de liste (puce ou numéro) en retrait, ou la ligne telle quelle.
(String, String) _listLine(String line) {
  final bullet = _bullet.firstMatch(line);
  if (bullet != null) return ('  •  ', bullet.group(2)!);
  final number = _numbered.firstMatch(line);
  if (number != null) return ('  ${number.group(2)}.  ', number.group(3)!);
  return ('', line);
}

/// Morceaux d'une ligne : texte, gras, italique, code ; un marquage non fermé reste tel quel.
List<InlineSpan> _lineSpans(String line, TextStyle base, Pal p) {
  final out = <InlineSpan>[];
  var at = 0;
  for (final m in _inline.allMatches(line)) {
    if (m.start > at) out.add(TextSpan(text: line.substring(at, m.start)));
    out.add(_styled(m, base, p));
    at = m.end;
  }
  if (at < line.length) out.add(TextSpan(text: line.substring(at)));
  return out;
}

TextSpan _styled(RegExpMatch m, TextStyle base, Pal p) {
  if (m.namedGroup('b') != null) {
    final bold = Fonts.sansWeight(
      base,
      FontWeight.w500,
    ).copyWith(color: p.text);
    return TextSpan(text: m.namedGroup('b'), style: bold);
  }
  if (m.namedGroup('c') != null) {
    final code = Fonts.mono(
      size: (base.fontSize ?? 15) - 1,
      color: p.text,
      ls: 0,
    );
    return TextSpan(text: m.namedGroup('c'), style: code);
  }
  final text = m.namedGroup('i') ?? m.namedGroup('u');
  return TextSpan(
    text: text,
    style: base.copyWith(fontStyle: FontStyle.italic),
  );
}

/// Réponse de l'IA en morceaux de texte stylés ; le HTML et les liens restent du texte brut.
List<InlineSpan> chatSpans(String text, TextStyle base, Pal p) {
  final out = <InlineSpan>[];
  final lines = text.split('\n');
  for (var i = 0; i < lines.length; i++) {
    final (marker, rest) = _listLine(lines[i]);
    if (marker.isNotEmpty) out.add(TextSpan(text: marker));
    out.addAll(_lineSpans(rest, base, p));
    if (i < lines.length - 1) out.add(const TextSpan(text: '\n'));
  }
  return out;
}

/// Le même texte sans marquage (pour l'insérer dans un mail) ; les listes gardent leur puce.
String plainFromMarkdown(String text) =>
    text.split('\n').map(_plainLine).join('\n');

String _plainLine(String line) {
  final (marker, rest) = _listLine(line);
  final plain = rest.replaceAllMapped(_inline, _unmarked);
  return marker.isEmpty ? plain : '${marker.trim()} $plain';
}

String _unmarked(Match match) {
  final m = match as RegExpMatch;
  final found = ['b', 'c', 'i', 'u'].map(m.namedGroup).nonNulls;
  return found.isEmpty ? '' : found.first;
}

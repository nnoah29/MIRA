import '../data/models.dart';

String _escape(String s) =>
    s.replaceAll('&', '&amp;').replaceAll('<', '&lt;').replaceAll('>', '&gt;');

String _span(Span s) {
  var h = _escape(s.text).replaceAll('\n', '<br>');
  if (s.code) h = '<code>$h</code>';
  if (s.bold) h = '<b>$h</b>';
  if (s.italic) h = '<i>$h</i>';
  if (s.underline) h = '<u>$h</u>';
  return h;
}

/// HTML d'un message rédigé : paragraphes et citations.
String blocksToHtml(List<Block> blocks) => [
  for (final b in blocks)
    switch (b) {
      Para p when p.loose => p.spans.map(_span).join(),
      Para p =>
        '<p>${p.spans.isEmpty ? '<br>' : p.spans.map(_span).join()}</p>',
      Quote q => '<blockquote>${blocksToHtml(q.children)}</blockquote>',
    },
].join('\n');

/// Texte brut d'un message rédigé ; les citations sont préfixées par « > ».
String blocksToText(List<Block> blocks) => [
  for (final b in blocks)
    switch (b) {
      Para p => p.text,
      Quote q => blocksToText(
        q.children,
      ).split('\n').map((l) => '> $l').join('\n'),
    },
].join('\n');

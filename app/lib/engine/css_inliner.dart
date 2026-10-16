import 'package:csslib/parser.dart' as css;
import 'package:csslib/visitor.dart' as vis;
import 'package:html/dom.dart' as dom;
import 'package:html/parser.dart' as html_parser;

/// Reporte les règles des balises `<style>` dans l'attribut `style` des éléments visés.
///
/// Les règles viennent avant le style en ligne, qui garde la priorité. Les règles
/// `@media`, pseudo-classes et sélecteurs non gérés sont ignorés sans erreur.
String inlineStyles(String html) {
  if (!html.toLowerCase().contains('<style')) return html;
  final doc = html_parser.parse(html);
  final rules = <MapEntry<String, String>>[];
  for (final style in doc.querySelectorAll('style')) {
    rules.addAll(_rulesOf(style.text));
  }
  for (final rule in rules) {
    _apply(doc, rule.key, rule.value);
  }
  return doc.outerHtml;
}

List<MapEntry<String, String>> _rulesOf(String sheet) {
  final out = <MapEntry<String, String>>[];
  try {
    final parsed = css.parse(sheet);
    for (final top in parsed.topLevels) {
      if (top is! vis.RuleSet) continue;
      final decls = _declarations(top);
      if (decls.isEmpty) continue;
      for (final sel
          in top.selectorGroup?.selectors ?? const <vis.Selector>[]) {
        out.add(MapEntry((sel.span?.text ?? '').trim(), decls));
      }
    }
  } catch (_) {
    return out;
  }
  return out;
}

String _declarations(vis.RuleSet rule) => [
  for (final d in rule.declarationGroup.declarations)
    if (d is vis.Declaration && d.span.text.isNotEmpty) '${d.span.text};',
].join();

void _apply(dom.Document doc, String selector, String declarations) {
  if (selector.contains(':') || selector.isEmpty) return;
  try {
    for (final e in doc.querySelectorAll(selector)) {
      e.attributes['style'] = '$declarations${e.attributes['style'] ?? ''}';
    }
  } catch (_) {
    return;
  }
}

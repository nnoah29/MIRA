import 'models.dart';

/// Mini-parseur du sous-ensemble HTML utilisé par les messages :
/// p, strong, em, code, br, a, span.at, blockquote.
List<Block> parseHtml(String html) {
  final root = <Block>[];
  final stack = <List<Block>>[root];
  List<Span>? para;
  var paraLoose = false;
  var bold = 0, italic = 0, underline = 0, code = 0, at = 0, link = 0;
  final atStack = <bool>[];

  void flush() {
    if (para != null) {
      stack.last.add(Para(para!, loose: paraLoose));
      para = null;
      paraLoose = false;
    }
  }

  void addText(String t) {
    if (t.isEmpty) return;
    if (para == null) {
      para = [];
      paraLoose = true;
    }
    para!.add(
      Span(
        t,
        bold: bold > 0,
        italic: italic > 0,
        underline: underline > 0,
        code: code > 0,
        at: at > 0,
        link: link > 0,
      ),
    );
  }

  final re = RegExp(r'<(/?)(\w+)([^>]*)>|([^<]+)');
  for (final m in re.allMatches(html)) {
    if (m.group(4) != null) {
      addText(_unescape(m.group(4)!));
      continue;
    }
    final close = m.group(1) == '/';
    final tag = m.group(2)!.toLowerCase();
    final attrs = m.group(3) ?? '';
    switch (tag) {
      case 'p':
        flush();
        if (close) break;
        para = [];
        paraLoose = false;
      case 'br':
        if (para == null) {
          para = [];
          paraLoose = true;
        }
        para!.add(const Span('\n'));
      case 'strong':
        bold += close ? -1 : 1;
      case 'em':
        italic += close ? -1 : 1;
      case 'u':
        underline += close ? -1 : 1;
      case 'code':
        code += close ? -1 : 1;
      case 'a':
        link += close ? -1 : 1;
      case 'span':
        if (close) {
          if (atStack.isNotEmpty && atStack.removeLast()) at--;
        } else {
          final isAt = attrs.contains('class="at"');
          atStack.add(isAt);
          if (isAt) at++;
        }
      case 'blockquote':
        flush();
        if (close) {
          if (stack.length > 1) {
            final kids = stack.removeLast();
            stack.last.add(Quote(kids));
          }
        } else {
          stack.add([]);
        }
    }
  }
  flush();
  while (stack.length > 1) {
    final kids = stack.removeLast();
    stack.last.add(Quote(kids));
  }
  return root;
}

String _unescape(String s) => s
    .replaceAll('&nbsp;', ' ')
    .replaceAll('&lt;', '<')
    .replaceAll('&gt;', '>')
    .replaceAll('&quot;', '"')
    .replaceAll('&#39;', "'")
    .replaceAll('&amp;', '&');

String _blocksPlain(List<Block> bs) {
  final out = <String>[];
  for (final b in bs) {
    switch (b) {
      case Para():
        out.add(b.spans.map((s) => s.text).join(' '));
      case Quote():
        out.add(_blocksPlain(b.children));
    }
  }
  return out.join(' ');
}

/// Texte brut (équivalent de `plain()` : balises remplacées par des espaces).
String plainOf(List<Block> bs) =>
    _blocksPlain(bs).replaceAll(RegExp(r'\s+'), ' ').trim();

String plainMsgs(Iterable<Msg> ms) => ms.map((m) => plainOf(m.body)).join(' ');

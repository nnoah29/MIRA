import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:google_fonts/google_fonts.dart';
import 'package:mira/theme/app_theme.dart';
import 'package:mira/ui/chat_markdown.dart';

List<TextSpan> spansOf(String text) {
  const base = TextStyle(fontSize: 15);
  return chatSpans(text, base, Pal.dark).cast<TextSpan>();
}

String flat(String text) => spansOf(text).map((s) => s.text).join();

TextSpan only(String text, String piece) =>
    spansOf(text).firstWhere((s) => s.text == piece);

void main() {
  setUpAll(() {
    TestWidgetsFlutterBinding.ensureInitialized();
    GoogleFonts.config.allowRuntimeFetching = false;
  });

  testWidgets('surrounding_blank_lines_are_dropped', (tester) async {
    expect(flat('\n\nBonjour\nle monde\n\n'), 'Bonjour\nle monde');
    expect(flat('  \n  Bonjour'), 'Bonjour');
    expect(plainFromMarkdown('\n\nBonjour\n'), 'Bonjour');
  });

  testWidgets('runs_of_blank_lines_collapse_to_one', (tester) async {
    expect(flat('un\n\n\n\ndeux'), 'un\n\ndeux');
    expect(plainFromMarkdown('un\n\n\n\ndeux'), 'un\n\ndeux');
  });

  testWidgets('bold_italic_code_spans', (tester) async {
    await tester.runAsync(Fonts.preload);
    const text = 'un **gras**, un *penché* et du `code`';
    expect(flat(text), 'un gras, un penché et du code');
    expect(only(text, 'gras').style?.fontWeight, FontWeight.w500);
    expect(only(text, 'penché').style?.fontStyle, FontStyle.italic);
    expect(only(text, 'code').style?.fontFamily, contains('JetBrains'));
  });

  testWidgets('bullet_and_numbered_lists', (tester) async {
    await tester.runAsync(Fonts.preload);
    expect(flat('- un\n* deux'), '  •  un\n  •  deux');
    expect(flat('1. a\n2) b'), '  1.  a\n  2.  b');
  });

  testWidgets('html_tags_stay_literal_text', (tester) async {
    await tester.runAsync(Fonts.preload);
    const text = '<b>fort</b> <script>alert(1)</script> <img src=x>';
    expect(flat(text), text);
    expect(spansOf(text).every((s) => s.recognizer == null), isTrue);
  });

  testWidgets('links_are_plain_text', (tester) async {
    await tester.runAsync(Fonts.preload);
    const text = 'voir [ici](https://example.org) ou https://example.org';
    expect(flat(text), text);
    expect(spansOf(text).every((s) => s.recognizer == null), isTrue);
  });

  testWidgets('unclosed_markup_does_not_throw', (tester) async {
    await tester.runAsync(Fonts.preload);
    for (final text in [
      '**ouvert',
      '`code',
      '*a',
      '_x',
      '**a** **b',
      '***',
      '',
    ]) {
      expect(() => spansOf(text), returnsNormally, reason: text);
      expect(flat(text).replaceAll('*', ''), text.replaceAll('*', ''));
    }
  });

  test('plain_text_for_a_reply_drops_markup_and_keeps_bullets', () {
    expect(
      plainFromMarkdown('**Bonjour** `Bob`,\n- un *point*\n2. deux'),
      'Bonjour Bob,\n• un point\n2. deux',
    );
  });
}

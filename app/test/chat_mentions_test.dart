import 'package:flutter_test/flutter_test.dart';
import 'package:mira/state/chat_context.dart';
import 'package:mira/state/chat_mentions.dart';

void main() {
  test('removes_complete_markers_and_returns_ids_in_order', () {
    final m = readMentions('Voici [mail:m2] puis [mail:m1], et [mail:m2].');
    expect(m.text, 'Voici puis, et.');
    expect(m.ids, ['m2', 'm1']);
  });

  test('hides_an_incomplete_marker_at_the_end_of_the_stream', () {
    for (final tail in ['[', '[m', '[mai', '[mail:', '[mail:m1']) {
      final m = readMentions('Voici $tail');
      expect(m.text, 'Voici', reason: tail);
      expect(m.ids, isEmpty, reason: tail);
    }
  });

  test('keeps_other_brackets_and_text_untouched', () {
    final m = readMentions('Tableau [1] et [IBAN_1] puis [mail] seul.');
    expect(m.text, 'Tableau [1] et [IBAN_1] puis [mail] seul.');
    expect(m.ids, isEmpty);
  });

  test('keeps_line_structure_around_markers', () {
    final m = readMentions('- [mail:a] Retour\n- [mail:b] Plan\n');
    expect(m.text, '- Retour\n- Plan\n');
    expect(m.ids, ['a', 'b']);
  });

  test('system_prompt_asks_to_cite_messages_with_their_id', () {
    expect(chatSystemPrompt(), contains('[mail:ID]'));
  });
}

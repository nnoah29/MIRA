import 'package:flutter_test/flutter_test.dart';
import 'package:mira/state/chat_mentions.dart';

void main() {
  test('accepts_the_id_prefix_copied_from_the_tool_output', () {
    final m = readMentions('Voici [mail:id=m1] et [mail:id=m2].');
    expect(m.text, 'Voici et.');
    expect(m.ids, ['m1', 'm2']);
  });

  test('accepts_spaces_and_any_letter_case_in_the_marker', () {
    final m = readMentions('Voir [Mail: m1 ] et [MAIL:m2].');
    expect(m.text, 'Voir et.');
    expect(m.ids, ['m1', 'm2']);
  });

  test('accepts_several_ids_in_one_marker', () {
    final m = readMentions('Ces deux [mail:a, b] sont urgents.');
    expect(m.text, 'Ces deux sont urgents.');
    expect(m.ids, ['a', 'b']);
  });

  test('hides_a_tolerant_marker_still_being_streamed', () {
    for (final tail in ['[mail: ', '[mail:id=', '[Mail:m1, m2', '[MAI']) {
      final m = readMentions('Voici $tail');
      expect(m.text, 'Voici', reason: tail);
      expect(m.ids, isEmpty, reason: tail);
    }
  });

  test('keeps_ids_that_contain_an_equal_sign', () {
    final m = readMentions('Voir [mail:abc==].');
    expect(m.ids, ['abc==']);
  });
}

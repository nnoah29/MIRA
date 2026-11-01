import 'package:flutter_test/flutter_test.dart';
import 'package:mira/state/chat_conversation.dart';

void main() {
  test('title_flattens_whitespace_and_trims', () {
    expect(
      conversationTitle('  Bonjour\n\n  le   monde \t'),
      'Bonjour le monde',
    );
  });

  test('title_is_cut_at_80_characters_counted_like_the_engine', () {
    expect(conversationTitle('a' * 120).runes.length, 80);
    expect(conversationTitle('😀' * 81).runes.length, 80);
    expect(conversationTitle('é' * 80), 'é' * 80);
  });

  test('blank_question_gets_a_default_title', () {
    expect(conversationTitle('  \n '), 'Nouvelle conversation');
  });

  test('ids_look_like_uuid_v4_and_differ', () {
    final a = newConversationId();
    final b = newConversationId();
    expect(
      a,
      matches(
        RegExp(
          r'^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$',
        ),
      ),
    );
    expect(a, isNot(b));
  });
}

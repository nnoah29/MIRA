import 'package:flutter_test/flutter_test.dart';
import 'package:mira/state/chat_message.dart';
import 'package:mira/state/chat_message_json.dart';

ChatMsg _rich() {
  final m = ChatMsg(
    role: 'ai',
    text: 'Réponse [mail:m1] é 😀',
    shown: 'partiel',
    draft: ['Bonjour', 'Merci'],
    refs: ['m1', 'm2'],
    fromEngine: true,
    failed: true,
    note: 'Génération arrêtée.',
    maskedCount: 3,
  );
  m.tools
    ..add(
      ToolActivity(
        id: 't1',
        name: 'search_messages',
        summary: 'facture',
        state: ToolState.ok,
        results: 4,
      ),
    )
    ..add(
      ToolActivity(
        id: 't2',
        name: 'get_message',
        state: ToolState.denied,
        deniedKind: 'mail_message',
      ),
    );
  m.sources
    ..add('m1')
    ..add('m2');
  return m;
}

void main() {
  test('round_trip_keeps_every_field', () {
    final text = encodeChat([ChatMsg(role: 'me', text: 'Question ?'), _rich()]);
    final back = decodeChat(text, chatPayloadVersion)!;
    expect(back, hasLength(2));
    expect(back[0].role, 'me');
    expect(back[0].text, 'Question ?');
    final m = back[1];
    expect(m.role, 'ai');
    expect(m.text, 'Réponse [mail:m1] é 😀');
    expect(m.draft, ['Bonjour', 'Merci']);
    expect(m.refs, ['m1', 'm2']);
    expect(m.sources, ['m1', 'm2']);
    expect(m.fromEngine, isTrue);
    expect(m.failed, isTrue);
    expect(m.note, 'Génération arrêtée.');
    expect(m.maskedCount, 3);
    expect(m.tools, hasLength(2));
    expect(m.tools[0].id, 't1');
    expect(m.tools[0].name, 'search_messages');
    expect(m.tools[0].summary, 'facture');
    expect(m.tools[0].state, ToolState.ok);
    expect(m.tools[0].results, 4);
    expect(m.tools[1].state, ToolState.denied);
    expect(m.tools[1].deniedKind, 'mail_message');
  });

  test('decoded_messages_are_done_and_shown', () {
    final partial = ChatMsg(role: 'ai', done: false, text: 'Texte', shown: '');
    final back = decodeChat(encodeChat([partial]), chatPayloadVersion)!;
    expect(back.single.done, isTrue);
    expect(back.single.shown, 'Texte');
  });

  test('unknown_version_gives_null', () {
    final text = encodeChat([ChatMsg(role: 'me', text: 'x')]);
    expect(decodeChat(text, 9), isNull);
  });

  test('corrupted_json_gives_null', () {
    expect(decodeChat('{pas du json', chatPayloadVersion), isNull);
    expect(decodeChat('', chatPayloadVersion), isNull);
  });

  test('wrong_shape_gives_null', () {
    for (final text in [
      '{"messages": 3}',
      '[]',
      '{}',
      '{"messages":[{"role":1,"text":"x"}]}',
      '{"messages":[{"role":"robot","text":"x"}]}',
      '{"messages":[{"role":"me"}]}',
    ]) {
      expect(decodeChat(text, chatPayloadVersion), isNull, reason: text);
    }
  });
}

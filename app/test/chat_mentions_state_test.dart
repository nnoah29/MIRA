import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:mira/engine/ai_admin.dart';
import 'package:mira/state/assistant.dart';
import 'package:mira/state/chat_context.dart';
import 'package:mira/state/chat_controller.dart';
import 'package:mira/state/chat_message.dart';

import 'fake_ai_engine.dart';

const noMail = ChatContextInput();

Map<String, dynamic> text(String s) => {'type': 'text', 'text': s};

Map<String, dynamic> finished(List<String> sources) => {
  'type': 'tool_finished',
  'id': 'c1',
  'state': 'ok',
  'results': sources.length,
  'sources': sources,
  'denied_kind': null,
};

void main() {
  late FakeAiEngine engine;
  late ChatController chat;

  setUp(() {
    engine = FakeAiEngine()
      ..providers.add({
        'id': 'openai',
        'display_name': 'OpenAI',
        'base_url': 'https://api.openai.com/v1',
        'base_url_editable': false,
      })
      ..connections.add({
        'id': 'ai-1',
        'provider': 'openai',
        'label': 'Mon OpenAI',
        'base_url': null,
        'default_model': 'm1',
        'is_default': true,
        'has_key': true,
        'consent_at': 1,
        'models': null,
      });
    chat = ChatController(
      admin: () => AiAdmin(engine),
      welcome: () => ChatMsg(role: 'ai', text: 'Bienvenue'),
      demoAnswer: (q) => Answer('démo : $q'),
      after: (ms, run) => Timer(Duration.zero, run),
    );
  });

  test('cited_messages_become_refs_and_leave_the_text', () async {
    engine.manual = true;
    final sending = chat.send('Urgent ?', noMail);
    await pumpEventQueue();
    engine.live!.add(finished(['m1', 'm2', 'm3']));
    await pumpEventQueue();
    engine.live!.add(text('Voici [mai'));
    await pumpEventQueue();
    expect(chat.chat.last.shown, 'Voici');
    engine.live!
      ..add(text('l:m1] et [mail:m2].'))
      ..add({'type': 'done', 'reason': 'end_turn'})
      ..close();
    await sending;
    final reply = chat.chat.last;
    expect(reply.text, 'Voici et.');
    expect(reply.refs, ['m1', 'm2']);
  });

  test('an_id_no_tool_returned_is_removed_but_never_shown_as_a_card', () async {
    engine.manual = true;
    final sending = chat.send('Urgent ?', noMail);
    await pumpEventQueue();
    engine.live!
      ..add(finished(['m1']))
      ..add(text('Voir [mail:m1] et [mail:zz].'))
      ..add({'type': 'done', 'reason': 'end_turn'})
      ..close();
    await sending;
    final reply = chat.chat.last;
    expect(reply.text, 'Voir et.');
    expect(reply.refs, ['m1']);
  });

  test('without_citation_there_are_no_refs', () async {
    engine.manual = true;
    final sending = chat.send('Urgent ?', noMail);
    await pumpEventQueue();
    engine.live!
      ..add(finished(['m1']))
      ..add(text('Rien de précis.'))
      ..add({'type': 'done', 'reason': 'end_turn'})
      ..close();
    await sending;
    expect(chat.chat.last.refs, isNull);
    expect(chat.chat.last.sources, ['m1']);
  });
}

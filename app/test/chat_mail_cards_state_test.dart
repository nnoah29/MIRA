import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:mira/engine/ai_admin.dart';
import 'package:mira/state/assistant.dart';
import 'package:mira/state/chat_context.dart';
import 'package:mira/state/chat_controller.dart';
import 'package:mira/state/chat_message.dart';
import 'package:mira/state/chat_message_json.dart';

import 'fake_ai_engine.dart';

const noMail = ChatContextInput();

Map<String, dynamic> finished(List<String> sources) => {
  'type': 'tool_finished',
  'id': 'c1',
  'state': 'ok',
  'results': sources.length,
  'sources': sources,
  'denied_kind': null,
};

Map<String, dynamic> shown(List<Map<String, dynamic>> cards, {String? title}) =>
    {
      'type': 'mail_cards_shown',
      'id': 'c2',
      'cards': {'title': title, 'cards': cards},
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

  test('the_cards_event_is_kept_on_the_reply_with_title_and_reasons', () async {
    engine.manual = true;
    final sending = chat.send('Importants ?', noMail);
    await pumpEventQueue();
    engine.live!
      ..add(finished(['m1', 'm2']))
      ..add(
        shown([
          {'id': 'm1', 'reason': 'échéance vendredi'},
          {'id': 'm2', 'reason': null},
        ], title: 'Mails importants'),
      )
      ..add({'type': 'text', 'text': 'Voilà.'})
      ..add({'type': 'done', 'reason': 'end_turn'})
      ..close();
    await sending;
    final lists = chat.chat.last.cards;
    expect(lists, hasLength(1));
    expect(lists.first.title, 'Mails importants');
    expect(lists.first.cards.map((c) => c.id), ['m1', 'm2']);
    expect(lists.first.cards.first.reason, 'échéance vendredi');
    expect(lists.first.cards.last.reason, isNull);
  });

  test('a_second_event_for_the_same_call_replaces_the_first', () async {
    engine.manual = true;
    final sending = chat.send('Importants ?', noMail);
    await pumpEventQueue();
    engine.live!
      ..add(
        shown([
          {'id': 'm1'},
        ]),
      )
      ..add(
        shown([
          {'id': 'm2'},
        ]),
      )
      ..add({'type': 'done', 'reason': 'end_turn'})
      ..close();
    await sending;
    expect(chat.chat.last.cards, hasLength(1));
    expect(chat.chat.last.cards.first.cards.single.id, 'm2');
  });

  test('cards_survive_the_saved_conversation_round_trip', () async {
    engine.manual = true;
    final sending = chat.send('Importants ?', noMail);
    await pumpEventQueue();
    engine.live!
      ..add(finished(['m1']))
      ..add(
        shown([
          {'id': 'm1', 'reason': 'urgent'},
        ], title: 'Urgent'),
      )
      ..add({'type': 'done', 'reason': 'end_turn'})
      ..close();
    await sending;
    final payload = encodeChat(chat.chat.skip(1).toList());
    final back = decodeChat(payload, chatPayloadVersion)!;
    final cards = back.last.cards.single;
    expect(cards.title, 'Urgent');
    expect(cards.cards.single.id, 'm1');
    expect(cards.cards.single.reason, 'urgent');
  });

  test('system_prompt_asks_for_the_show_mails_tool_to_present_lists', () {
    expect(chatSystemPrompt(), contains('show_mails'));
    expect(chatSystemPrompt(), contains('[mail:ID]'));
  });
}

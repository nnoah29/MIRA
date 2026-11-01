import 'dart:convert';

import 'package:flutter_test/flutter_test.dart';
import 'package:mira/engine/engine_client.dart';
import 'package:mira/state/chat_controller.dart';

import 'chat_history_support.dart';

List<dynamic> messagesOf(Map<String, dynamic> save) =>
    (jsonDecode(save['payload'] as String) as Map)['messages'] as List;

void main() {
  test('one_save_per_finished_turn', () async {
    final r = rig();
    r.engine.scripts
      ..add(reply('Bonjour'))
      ..add(reply('Encore'));
    await r.chat.send('Salut', noContext);
    expect(r.saves, hasLength(1));
    await r.chat.send('Et ensuite ?', noContext);
    expect(r.saves, hasLength(2));
    expect(r.saves[1]['id'], r.saves[0]['id']);
    expect(r.saves[0]['payload_version'], 1);
    expect(messagesOf(r.saves[1]), hasLength(4));
    expect(r.chat.conversationId, r.saves[0]['id']);
    expect(r.chat.saveFailed, isFalse);
  });

  test('no_save_while_generating', () async {
    final r = rig();
    r.engine.manual = true;
    final sending = r.chat.send('Salut', noContext);
    await pumpEventQueue();
    r.engine.live!
      ..add({'type': 'started', 'operation': 'op-1'})
      ..add({'type': 'text', 'text': 'Bon'});
    await pumpEventQueue();
    expect(r.saves, isEmpty);
    r.engine.live!
      ..add({'type': 'done', 'reason': 'end_turn'})
      ..close();
    await sending;
    expect(r.saves, hasLength(1));
  });

  test('no_save_in_demo', () async {
    final r = rig(demo: true);
    await r.chat.send('Salut', noContext);
    await pumpEventQueue();
    expect(r.saves, isEmpty);
    expect(r.chat.conversationId, isNull);
  });

  test('no_save_when_history_disabled', () async {
    final r = rig();
    r.engine.history.enabled = false;
    await r.history.load();
    r.engine.scripts.add(reply('Bonjour'));
    await r.chat.send('Salut', noContext);
    expect(r.saves, isEmpty);
    expect(r.chat.saveFailed, isFalse);
  });

  test('no_save_when_only_welcome', () async {
    final r = rig(consent: false);
    r.engine.scripts.add([
      {
        'type': 'error',
        'error': {
          'code': 'incompatible_state.ai_consent_required',
          'message': 'x',
        },
      },
    ]);
    await r.chat.send('Salut', noContext);
    expect(r.chat.chat, hasLength(1));
    expect(r.saves, isEmpty);
  });

  test('disabling_mid_run_skips_that_turn', () async {
    final r = rig();
    r.engine.manual = true;
    final sending = r.chat.send('Salut', noContext);
    await pumpEventQueue();
    r.engine.live!.add({'type': 'started', 'operation': 'op-1'});
    await r.history.setEnabled(false);
    r.engine.live!
      ..add({'type': 'text', 'text': 'Bonjour'})
      ..add({'type': 'done', 'reason': 'end_turn'})
      ..close();
    await sending;
    expect(r.saves, isEmpty);
  });

  test('title_is_the_first_question_and_welcome_is_not_in_payload', () async {
    final r = rig();
    r.engine.scripts.add(reply('Bonjour'));
    await r.chat.send('  Résume\nmes   mails  ', noContext);
    expect(r.saves.single['title'], 'Résume mes mails');
    final messages = messagesOf(r.saves.single);
    expect(messages.map((m) => (m as Map)['role']), ['me', 'ai']);
    expect(jsonEncode(messages).contains('Bienvenue'), isFalse);
  });

  test('save_failure_does_not_break_chat_and_retries', () async {
    final r = rig();
    r.engine
      ..failOpName = 'save_ai_conversation'
      ..failOp = const EngineException('server.ai_unavailable', '')
      ..scripts.add(reply('Bonjour'))
      ..scripts.add(reply('Encore'));
    await r.chat.send('Salut', noContext);
    expect(r.chat.saveFailed, isTrue);
    expect(r.chat.chat.last.text, 'Bonjour');
    expect(r.chat.busy, isFalse);
    r.engine.failOp = null;
    await r.chat.send('Et ensuite ?', noContext);
    expect(r.chat.saveFailed, isFalse);
    expect(messagesOf(r.saves.last), hasLength(4));
  });

  test('engine_refusal_resyncs_without_warning', () async {
    final r = rig();
    await r.history.load();
    r.engine.history.enabled = false;
    r.engine.scripts.add(reply('Bonjour'));
    await r.chat.send('Salut', noContext);
    await pumpEventQueue();
    expect(r.saves, hasLength(1));
    expect(r.chat.saveFailed, isFalse);
    expect(r.history.enabled, isFalse);
  });
}

import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:mira/engine/ai_admin.dart';
import 'package:mira/engine/engine_client.dart';
import 'package:mira/state/assistant.dart';
import 'package:mira/state/chat_context.dart';
import 'package:mira/state/chat_controller.dart';
import 'package:mira/state/chat_message.dart';

import 'fake_ai_engine.dart';

const noMail = ChatContextInput();

Map<String, dynamic> text(String s) => {'type': 'text', 'text': s};

Map<String, dynamic> started(String op) => {'type': 'started', 'operation': op};

Map<String, dynamic> done(Object reason) => {'type': 'done', 'reason': reason};

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
      })
      ..manual = true;
    chat = ChatController(
      admin: () => AiAdmin(engine),
      welcome: () => ChatMsg(role: 'ai', text: 'Bienvenue'),
      demoAnswer: (q) => Answer('démo : $q'),
      after: (ms, run) => Timer(Duration.zero, run),
    );
  });

  test('stream_ending_without_done_keeps_text_and_resets_busy', () async {
    final sending = chat.send('Salut', noMail);
    await pumpEventQueue();
    engine.live!
      ..add(text('Début'))
      ..close();
    await sending;
    expect(chat.busy, isFalse);
    expect(chat.chat.last.text, 'Début');
    expect(chat.chat.last.done, isTrue);
  });

  test('error_mid_stream_fails_the_reply_and_resets_busy', () async {
    final sending = chat.send('Salut', noMail);
    await pumpEventQueue();
    engine.live!
      ..add(text('Début'))
      ..addError(const EngineException('server.ai_unavailable', 'x'));
    await sending;
    expect(chat.busy, isFalse);
    expect(chat.chat.last.failed, isTrue);
    expect(chat.chat.last.text, contains('Début'));
    expect(chat.chat.last.text, contains('Service d’IA indisponible'));
  });

  test('error_line_mid_stream_fails_the_reply', () async {
    final sending = chat.send('Salut', noMail);
    await pumpEventQueue();
    engine.live!.add({
      'type': 'error',
      'error': {'code': 'server.ai_bad_response', 'message': 'x'},
    });
    await sending;
    expect(chat.busy, isFalse);
    expect(chat.chat.last.text, contains('Réponse inattendue'));
  });

  test('stop_before_started_honours_the_pending_cancel', () async {
    final sending = chat.send('Salut', noMail);
    await pumpEventQueue();
    await chat.stop();
    await sending;
    expect(chat.busy, isFalse);
    expect(chat.chat.last.done, isTrue);
    expect(engine.requestsOf('cancel_operation'), isEmpty);
    expect(engine.cancelledStreams, 0);
    engine.live!.add(started('op-5'));
    await pumpEventQueue();
    expect(engine.requestsOf('cancel_operation').single['operation'], 'op-5');
    expect(engine.cancelledStreams, 1);
  });

  test('clear_during_generation_cancels_and_stays_quiet_afterwards', () async {
    final sending = chat.send('Salut', noMail);
    await pumpEventQueue();
    engine.live!
      ..add(started('op-3'))
      ..add(text('Début'));
    await pumpEventQueue();
    var notified = 0;
    chat.addListener(() => notified++);
    chat.clear();
    await sending;
    await pumpEventQueue();
    expect(engine.requestsOf('cancel_operation').single['operation'], 'op-3');
    expect(engine.cancelledStreams, 1);
    expect(chat.busy, isFalse);
    expect(chat.chat.map((m) => m.text), ['Bienvenue']);
    final after = notified;
    engine.live!.add(text('tard'));
    await pumpEventQueue();
    expect(notified, after);
    expect(chat.chat.length, 1);
  });

  test('reset_before_started_cancels_once_the_operation_is_known', () async {
    final sending = chat.send('Salut', noMail);
    await pumpEventQueue();
    chat.reset();
    await sending;
    expect(chat.busy, isFalse);
    engine.live!.add(started('op-8'));
    await pumpEventQueue();
    expect(engine.requestsOf('cancel_operation').single['operation'], 'op-8');
    expect(chat.chat.map((m) => m.text), ['Bienvenue']);
  });

  test('second_send_while_busy_is_ignored', () async {
    final first = chat.send('Un', noMail);
    await pumpEventQueue();
    await chat.send('Deux', noMail);
    expect(engine.chatStarts, hasLength(1));
    expect(chat.chat.where((m) => m.role == 'me'), hasLength(1));
    engine.live!
      ..add(done('end_turn'))
      ..close();
    await first;
  });

  test('stopped_empty_reply_is_not_content_and_not_sent_back', () async {
    final sending = chat.send('Un', noMail);
    await pumpEventQueue();
    engine.live!.add(started('op-1'));
    await pumpEventQueue();
    await chat.stop();
    await sending;
    final reply = chat.chat.last;
    expect(reply.note, 'Génération arrêtée.');
    expect(reply.failed, isTrue);
    expect(reply.text, isEmpty);
    engine.manual = false;
    engine.scripts.add([text('ok'), done('end_turn')]);
    await chat.send('Deux', noMail);
    final sent = engine.chatStarts.last['request']['messages'] as List;
    expect(sent.map((m) => m['text']), ['Deux']);
  });

  test('truncation_note_stays_out_of_the_text_sent_back', () async {
    final sending = chat.send('Un', noMail);
    await pumpEventQueue();
    engine.live!
      ..add(text('Abc'))
      ..add(done('max_tokens'));
    await sending;
    final reply = chat.chat.last;
    expect(reply.text, 'Abc');
    expect(reply.note, contains('tronquée'));
    expect(reply.failed, isFalse);
    engine.manual = false;
    engine.scripts.add([text('ok'), done('end_turn')]);
    await chat.send('Deux', noMail);
    final sent = engine.chatStarts.last['request']['messages'] as List;
    expect(sent.map((m) => m['text']), ['Un', 'Abc', 'Deux']);
  });

  test('loopback_check_rejects_lookalike_hosts', () {
    expect(isLoopbackUrl('http://127.0.0.1:11434/v1'), isTrue);
    expect(isLoopbackUrl('http://localhost:1234/v1'), isTrue);
    expect(isLoopbackUrl('http://[::1]:8080/v1'), isTrue);
    expect(isLoopbackUrl('http://127.evil.com/v1'), isFalse);
    expect(isLoopbackUrl('http://localhost.evil.com/v1'), isFalse);
    expect(isLoopbackUrl(null), isFalse);
  });
}

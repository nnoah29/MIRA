import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:mira/engine/ai_admin.dart';
import 'package:mira/state/assistant.dart';
import 'package:mira/state/chat_context.dart';
import 'package:mira/state/chat_controller.dart';
import 'package:mira/state/chat_message.dart';

import 'fake_ai_engine.dart';

const noMail = ChatContextInput();

Map<String, dynamic> connection({
  bool isDefault = true,
  bool hasKey = true,
  String? baseUrl,
  bool consent = true,
}) => {
  'id': 'ai-1',
  'provider': 'openai',
  'label': 'Mon OpenAI',
  'base_url': baseUrl,
  'default_model': 'm1',
  'is_default': isDefault,
  'has_key': hasKey,
  'consent_at': consent ? 1 : null,
  'models': null,
};

Map<String, dynamic> done(Object reason) => {'type': 'done', 'reason': reason};

Map<String, dynamic> text(String s) => {'type': 'text', 'text': s};

Map<String, dynamic> started([String op = 'op-1']) => {
  'type': 'started',
  'operation': op,
};

void main() {
  late FakeAiEngine engine;
  late ChatController chat;

  ChatController build({bool withAdmin = true}) => ChatController(
    admin: () => withAdmin ? AiAdmin(engine) : null,
    welcome: () => ChatMsg(role: 'ai', text: 'Bienvenue'),
    demoAnswer: (q) => Answer('démo : $q'),
    after: (ms, run) => Timer(Duration.zero, run),
  );

  setUp(() {
    engine = FakeAiEngine()
      ..providers.add({
        'id': 'openai',
        'display_name': 'OpenAI',
        'base_url': 'https://api.openai.com/v1',
        'base_url_editable': false,
      })
      ..connections.add(connection());
    chat = build();
  });

  test('streams_text_into_the_last_message_progressively', () async {
    engine.manual = true;
    final sending = chat.send('Salut', noMail);
    await pumpEventQueue();
    engine.live!
      ..add(started())
      ..add(text('Bon'));
    await pumpEventQueue();
    expect(chat.chat.last.shown, 'Bon');
    expect(chat.chat.last.done, isFalse);
    engine.live!.add(text('jour'));
    await pumpEventQueue();
    expect(chat.chat.last.shown, 'Bonjour');
    engine.live!
      ..add(done('end_turn'))
      ..close();
    await sending;
    expect(chat.chat.last.text, 'Bonjour');
    expect(chat.chat.last.done, isTrue);
    expect(chat.chat.last.fromEngine, isTrue);
  });

  test('busy_true_during_stream_false_after_done', () async {
    engine.manual = true;
    final sending = chat.send('Salut', noMail);
    await pumpEventQueue();
    expect(chat.busy, isTrue);
    engine.live!
      ..add(text('ok'))
      ..add(done('end_turn'));
    await sending;
    expect(chat.busy, isFalse);
    expect(chat.demo, isFalse);
  });

  test('stop_cancels_operation_and_keeps_partial_text', () async {
    engine.manual = true;
    final sending = chat.send('Salut', noMail);
    await pumpEventQueue();
    engine.live!
      ..add(started('op-9'))
      ..add(text('Début'));
    await pumpEventQueue();
    await chat.stop();
    await sending;
    expect(engine.requestsOf('cancel_operation').single['operation'], 'op-9');
    expect(chat.chat.last.text, 'Début');
    expect(chat.chat.last.done, isTrue);
    expect(chat.busy, isFalse);
    expect(engine.cancelledStreams, 1);
  });

  test('failed_done_shows_translated_error_and_keeps_history', () async {
    engine.scripts.add([
      started(),
      text('Début'),
      done({
        'failed': {'code': 'server.ai_unavailable', 'retry_after_secs': null},
      }),
    ]);
    await chat.send('Salut', noMail);
    expect(chat.chat.map((m) => m.role), ['ai', 'me', 'ai']);
    expect(chat.chat.last.text, contains('Début'));
    expect(chat.chat.last.text, contains('Service d’IA indisponible'));
    expect(chat.chat.last.failed, isTrue);
    engine.scripts.add([text('Re'), done('end_turn')]);
    await chat.send('Encore', noMail);
    final sent = engine.chatStarts.last['request']['messages'] as List;
    expect(sent.map((m) => m['text']), ['Encore']);
  });

  test('retry_after_is_shown_for_rate_limit', () async {
    engine.scripts.add([
      done({
        'failed': {'code': 'server.ai_rate_limited', 'retry_after_secs': 7},
      }),
    ]);
    await chat.send('Salut', noMail);
    expect(chat.chat.last.text, contains('Réessaie dans 7 s.'));
  });

  test('no_connection_uses_demo_answer_with_banner', () async {
    engine.connections.clear();
    await chat.refresh();
    expect(chat.demo, isTrue);
    await chat.send('Salut', noMail);
    await pumpEventQueue();
    await pumpEventQueue();
    expect(chat.chat.last.text, 'démo : Salut');
    expect(chat.chat.last.fromEngine, isFalse);
    expect(chat.chat.last.done, isTrue);
    expect(engine.chatStarts, isEmpty);
    expect(chat.busy, isFalse);
  });

  test('without_an_engine_the_demo_answers_synchronously_started', () async {
    final demo = build(withAdmin: false);
    expect(demo.demo, isTrue);
    await demo.send('Salut', noMail);
    expect(demo.busy, isTrue);
    await pumpEventQueue();
    expect(demo.chat.last.text, 'démo : Salut');
  });

  test('a_key_less_service_is_usable_only_on_loopback', () async {
    engine.connections
      ..clear()
      ..add(connection(hasKey: false, baseUrl: 'https://ia.example.org/v1'));
    await chat.refresh();
    expect(chat.demo, isTrue);
    engine.connections
      ..clear()
      ..add(connection(hasKey: false, baseUrl: 'http://127.0.0.1:11434/v1'));
    await chat.refresh();
    expect(chat.demo, isFalse);
    engine.connections
      ..clear()
      ..add(connection(isDefault: false));
    await chat.refresh();
    expect(chat.demo, isTrue);
  });

  test('consent_required_sets_needsConsent_then_resend_works', () async {
    engine.connections
      ..clear()
      ..add(connection(consent: false));
    engine.scripts.add([
      {
        'type': 'error',
        'error': {
          'code': 'incompatible_state.ai_consent_required',
          'message': 'x',
        },
      },
    ]);
    await chat.send('Salut', noMail);
    expect(chat.needsConsent('ai-1'), isTrue);
    expect(chat.consentProviderName, 'OpenAI');
    expect(chat.chat.length, 1);
    expect(chat.busy, isFalse);
    engine.scripts.add([text('Bonjour'), done('end_turn')]);
    await chat.grantConsent();
    expect(
      engine.requestsOf('acknowledge_ai_consent').single['connection'],
      'ai-1',
    );
    expect(chat.needsConsent('ai-1'), isFalse);
    expect(chat.chat.map((m) => m.text), ['Bienvenue', 'Salut', 'Bonjour']);
  });

  test('declining_consent_keeps_the_question_and_sends_nothing', () async {
    engine.scripts.add([
      {
        'type': 'error',
        'error': {
          'code': 'incompatible_state.ai_consent_required',
          'message': 'x',
        },
      },
    ]);
    await chat.send('Salut', noMail);
    chat.declineConsent();
    expect(chat.needsConsent('ai-1'), isFalse);
    expect(chat.chat[1].text, 'Salut');
    expect(chat.chat.last.text, contains('rien n’a été transmis'));
    expect(engine.requestsOf('acknowledge_ai_consent'), isEmpty);
  });

  test('empty_answer_shows_empty_notice', () async {
    engine.scripts.add([started(), done('end_turn')]);
    await chat.send('Salut', noMail);
    expect(chat.chat.last.text, 'Le modèle n’a rien répondu.');
    expect(chat.chat.last.failed, isTrue);
  });

  test('clear_keeps_only_the_welcome_message', () async {
    engine.scripts.add([text('a'), done('end_turn')]);
    await chat.send('Salut', noMail);
    chat.clear();
    expect(chat.chat.map((m) => m.text), ['Bienvenue']);
  });
}

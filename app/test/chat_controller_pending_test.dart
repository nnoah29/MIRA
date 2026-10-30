import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:mira/engine/ai_admin.dart';
import 'package:mira/engine/engine_client.dart';
import 'package:mira/state/assistant.dart';
import 'package:mira/state/chat_context.dart';
import 'package:mira/state/chat_controller.dart';
import 'package:mira/state/chat_message.dart';

import 'fake_ai_engine.dart';

const mail = ChatContextInput(subject: 'Projet', body: 'Bonjour.');

const consentError = {
  'type': 'error',
  'error': {'code': 'incompatible_state.ai_consent_required', 'message': 'x'},
};

const contentError = {
  'type': 'error',
  'error': {
    'code': 'incompatible_state.ai_content_not_allowed',
    'message': 'x',
  },
};

const okReply = [
  {'type': 'text', 'text': 'Voici.'},
  {'type': 'done', 'reason': 'end_turn'},
];

class HeldEngine extends FakeAiEngine {
  final gate = Completer<void>();
  String? holdOp;

  @override
  Future<Map<String, dynamic>> call(Map<String, dynamic> request) async {
    if ((request['request'] as Map)['op'] == holdOp) await gate.future;
    return super.call(request);
  }
}

void main() {
  late HeldEngine engine;
  late ChatController chat;

  setUp(() {
    engine = HeldEngine()
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
        'consent_at': null,
        'models': null,
        'allowed_kinds': <String>[],
      });
    chat = ChatController(
      admin: () => AiAdmin(engine),
      welcome: () => ChatMsg(role: 'ai', text: 'Bienvenue'),
      demoAnswer: (q) => Answer('démo : $q'),
      after: (ms, run) => Timer(Duration.zero, run),
    );
  });

  test('send_clears_stale_pending_state', () async {
    engine.scripts.addAll([
      [consentError],
      [contentError],
      okReply,
    ]);
    await chat.send('Première', mail);
    expect(chat.consentConnectionId, 'ai-1');
    await chat.send('Seconde', mail);
    expect(chat.consentConnectionId, isNull);
    expect(chat.contentConnectionId, 'ai-1');
    await chat.send('Troisième', mail);
    expect(chat.contentConnectionId, isNull);
  });

  test('busy_stays_true_during_grant', () async {
    engine.scripts.addAll([
      [consentError],
      okReply,
    ]);
    await chat.send('Question', mail);
    engine.holdOp = 'acknowledge_ai_consent';
    final granting = chat.grantConsent();
    await pumpEventQueue();
    expect(chat.busy, isTrue);
    await chat.send('Autre', mail);
    expect(engine.chatStarts, hasLength(1));
    engine.gate.complete();
    await granting;
    expect(engine.chatStarts, hasLength(2));
    expect(chat.busy, isFalse);
  });

  test('busy_stays_true_while_content_is_granted', () async {
    engine.scripts.addAll([
      [contentError],
      okReply,
    ]);
    await chat.send('Question', mail);
    engine.holdOp = 'set_ai_allowed_content';
    final granting = chat.grantContent();
    await pumpEventQueue();
    expect(chat.busy, isTrue);
    engine.gate.complete();
    await granting;
    expect(chat.busy, isFalse);
  });

  test('a_failed_grant_releases_busy', () async {
    engine.scripts.add([consentError]);
    await chat.send('Question', mail);
    engine.failOpName = 'acknowledge_ai_consent';
    engine.failOp = const EngineException('server.ai_unavailable', '');
    await chat.grantConsent();
    expect(chat.busy, isFalse);
    expect(chat.chat.last.failed, isTrue);
  });

  test(
    'unticked_mail_resends_without_context_then_next_send_asks_content',
    () async {
      engine.scripts.addAll([
        [consentError],
        okReply,
        [contentError],
      ]);
      await chat.send('Question', mail);
      await chat.grantConsent(const []);
      final resent = (engine.chatStarts[1]['request'] as Map)['context'];
      expect(resent, isEmpty);
      expect(chat.contentConnectionId, isNull);
      await chat.send('Suite', mail);
      expect(chat.contentConnectionId, 'ai-1');
    },
  );
}

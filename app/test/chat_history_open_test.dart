import 'dart:convert';

import 'package:flutter_test/flutter_test.dart';
import 'package:mira/engine/ai_models.dart';
import 'package:mira/engine/engine_client.dart';
import 'package:mira/state/chat_controller.dart';
import 'package:mira/state/chat_message.dart';

import 'chat_history_support.dart';

List<ChatMsg> sample() => [
  ChatMsg(role: 'me', text: 'Q'),
  ChatMsg(role: 'ai', text: 'R', fromEngine: true),
];

void main() {
  test('open_replaces_thread_and_keeps_id', () async {
    final r = rig();
    seedConversation(r.engine, 'c-1', sample());
    final error = await r.chat.open('c-1');
    expect(error, isNull);
    expect(r.chat.chat.map((m) => m.text), ['Bienvenue', 'Q', 'R']);
    expect(r.chat.chat.skip(1).every((m) => m.done), isTrue);
    expect(r.chat.conversationId, 'c-1');
  });

  test('next_turn_after_open_saves_into_same_conversation', () async {
    final r = rig();
    seedConversation(r.engine, 'c-1', sample());
    await r.chat.open('c-1');
    r.engine.scripts.add(reply('Suite'));
    await r.chat.send('Et après ?', noContext);
    expect(r.saves.single['id'], 'c-1');
    final payload = jsonDecode(r.saves.single['payload'] as String) as Map;
    expect(payload['messages'], hasLength(4));
    expect(r.engine.history.conversations, hasLength(1));
  });

  test('open_is_ignored_while_busy', () async {
    final r = rig();
    seedConversation(r.engine, 'c-1', sample());
    r.engine.manual = true;
    final sending = r.chat.send('Salut', noContext);
    await pumpEventQueue();
    expect(r.chat.busy, isTrue);
    expect(await r.chat.open('c-1'), isNull);
    expect(r.engine.requestsOf('get_ai_conversation'), isEmpty);
    expect(r.chat.conversationId, isNull);
    r.engine.live!
      ..add({'type': 'done', 'reason': 'end_turn'})
      ..close();
    await sending;
  });

  test('unreadable_payload_keeps_thread_and_returns_error_text', () async {
    final r = rig();
    seedConversation(r.engine, 'bad', const [], version: 9);
    seedConversation(r.engine, 'worse', const [], payload: '{pas du json');
    for (final id in ['bad', 'worse']) {
      expect(await r.chat.open(id), 'Conversation illisible');
    }
    expect(r.chat.chat, hasLength(1));
    expect(r.chat.conversationId, isNull);
  });

  test('engine_error_on_get_returns_translated_text', () async {
    final r = rig();
    r.engine
      ..failOpName = 'get_ai_conversation'
      ..failOp = const EngineException('server.ai_unavailable', '');
    expect(await r.chat.open('c-1'), 'Service d’IA indisponible');
    expect(r.chat.chat, hasLength(1));
  });

  test('start_new_gives_fresh_id_and_keeps_old_saved', () async {
    final r = rig();
    r.engine.scripts
      ..add(reply('Un'))
      ..add(reply('Deux'));
    await r.chat.send('Première', noContext);
    final first = r.chat.conversationId;
    r.chat.startNew();
    expect(r.chat.chat, hasLength(1));
    expect(r.chat.conversationId, isNull);
    await r.chat.send('Seconde', noContext);
    expect(r.chat.conversationId, isNot(first));
    expect(r.engine.history.conversations, hasLength(2));
  });

  test('reset_forgets_conversation_id', () async {
    final r = rig();
    r.engine.scripts.add(reply('Un'));
    await r.chat.send('Première', noContext);
    expect(r.chat.conversationId, isNotNull);
    r.chat.reset();
    expect(r.chat.conversationId, isNull);
    expect(r.chat.saveFailed, isFalse);
  });

  test('pending_action_cards_of_a_restored_conversation_expire', () async {
    final r = rig();
    final reply = ChatMsg(
      role: 'ai',
      text: 'Je te propose ça.',
      fromEngine: true,
    );
    final proposal = AiActionProposal(
      id: 'c1',
      action: 'trash',
      messages: const ['m1'],
    );
    reply.actions
      ..add(ProposedAction(proposal))
      ..add(
        ProposedAction(
          AiActionProposal(id: 'c2', action: 'archive', messages: const ['m2']),
          status: 'done',
        ),
      );
    seedConversation(r.engine, 'old', [
      ChatMsg(role: 'me', text: 'Range'),
      reply,
    ]);
    expect(await r.chat.open('old'), isNull);
    final restored = r.chat.chat.last.actions;
    expect(restored.map((a) => a.status), ['dismissed', 'done']);
  });
}

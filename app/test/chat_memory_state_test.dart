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

Map<String, dynamic> proposed(
  String action,
  String text, {
  String? id,
  String kind = 'preference',
  String call = 'c1',
}) => {
  'type': 'memory_proposed',
  'id': call,
  'proposal': {'action': action, 'text': text, 'kind': kind, 'id': id},
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
    engine.memory.settings = {
      'enabled': true,
      'embedder': {'kind': 'local'},
    };
    chat = ChatController(
      admin: () => AiAdmin(engine),
      welcome: () => ChatMsg(role: 'ai', text: 'Bienvenue'),
      demoAnswer: (q) => Answer('démo : $q'),
      after: (ms, run) => Timer(Duration.zero, run),
    );
  });

  Future<void> receive(List<Map<String, dynamic>> lines) async {
    engine.manual = true;
    final sending = chat.send('Retiens ça', noMail);
    await pumpEventQueue();
    for (final line in lines) {
      engine.live!.add(line);
    }
    engine.live!
      ..add({'type': 'done', 'reason': 'end_turn'})
      ..close();
    await sending;
    await pumpEventQueue();
  }

  test('a_remember_proposal_waits_pending_and_writes_nothing', () async {
    await receive([proposed('remember', 'Je signe « Noah »')]);
    final item = chat.chat.last.memories.single;
    expect((item.status, item.proposal.action), ('pending', 'remember'));
    expect(item.proposal.text, 'Je signe « Noah »');
    expect(engine.memory.records, isEmpty);
  });

  test('confirming_remember_writes_the_memory_and_marks_it_done', () async {
    await receive([proposed('remember', 'Je signe « Noah »')]);
    final item = chat.chat.last.memories.single;
    await chat.settleMemory(item, apply: true);
    expect(item.status, 'done');
    expect(engine.memory.records.single['text'], 'Je signe « Noah »');
    expect(engine.memory.records.single['kind'], 'preference');
  });

  test('dismissing_never_writes', () async {
    await receive([proposed('remember', 'x')]);
    final item = chat.chat.last.memories.single;
    await chat.settleMemory(item, apply: false);
    expect(item.status, 'dismissed');
    expect(engine.memory.records, isEmpty);
    await chat.settleMemory(item, apply: true);
    expect(item.status, 'dismissed');
  });

  test('a_forget_proposal_resolves_the_memory_text_then_forgets_it', () async {
    engine.memory.records.add({
      'id': 'mem-9',
      'text': 'Mon chat est Mochi',
      'kind': 'fact',
      'source': 'user',
      'created_at': 5,
      'model_id': 'embed',
    });
    await receive([proposed('forget', 'obsolète', id: 'mem-9', kind: 'fact')]);
    final item = chat.chat.last.memories.single;
    expect(item.target, 'Mon chat est Mochi');
    await chat.settleMemory(item, apply: true);
    expect(item.status, 'done');
    expect(engine.memory.records, isEmpty);
  });

  test('a_forget_of_an_unknown_memory_cannot_be_confirmed', () async {
    await receive([proposed('forget', 'x', id: 'absent', kind: 'fact')]);
    final item = chat.chat.last.memories.single;
    expect((item.target, item.missing), (null, true));
    await chat.settleMemory(item, apply: true);
    expect(item.status, 'pending');
  });

  test('a_disabled_memory_keeps_the_card_pending_with_an_error', () async {
    await receive([proposed('remember', 'x')]);
    engine.memory.settings = {
      'enabled': false,
      'embedder': {'kind': 'local'},
    };
    final item = chat.chat.last.memories.single;
    await chat.settleMemory(item, apply: true);
    expect(item.status, 'pending');
    expect(item.error, isNotNull);
  });

  test('a_second_event_for_the_same_call_replaces_the_first', () async {
    await receive([proposed('remember', 'un'), proposed('remember', 'deux')]);
    expect(chat.chat.last.memories.map((m) => m.proposal.text), ['deux']);
  });

  test('proposals_and_their_status_survive_the_saved_round_trip', () async {
    await receive([proposed('remember', 'un', call: 'c1')]);
    await chat.settleMemory(chat.chat.last.memories.single, apply: false);
    final back = decodeChat(
      encodeChat(chat.chat.skip(1).toList()),
      chatPayloadVersion,
    )!;
    final item = back.last.memories.single;
    expect((item.status, item.proposal.text), ('dismissed', 'un'));
  });
}

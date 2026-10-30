import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:mira/engine/ai_admin.dart';
import 'package:mira/state/assistant.dart';
import 'package:mira/state/chat_context.dart';
import 'package:mira/state/chat_controller.dart';
import 'package:mira/state/chat_message.dart';

import 'fake_ai_engine.dart';

const noMail = ChatContextInput();

Map<String, dynamic> called(String id, String name, [String summary = '']) => {
  'type': 'tool_called',
  'id': id,
  'name': name,
  'summary': summary,
};

Map<String, dynamic> finished(
  String id,
  String state, {
  int results = 0,
  List<String> sources = const [],
  String? denied,
}) => {
  'type': 'tool_finished',
  'id': id,
  'state': state,
  'results': results,
  'sources': sources,
  'denied_kind': denied,
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

  /// Joue ces lignes dans un flux manuel et rend la fin de l'envoi.
  Future<void> play(List<Map<String, dynamic>> lines) async {
    engine.manual = true;
    final sending = chat.send('Cherche', noMail);
    await pumpEventQueue();
    for (final line in lines) {
      engine.live!.add(line);
      await pumpEventQueue();
    }
    engine.live!
      ..add({'type': 'done', 'reason': 'end_turn'})
      ..close();
    await sending;
  }

  test('adds_running_then_ok_activity', () async {
    engine.manual = true;
    final sending = chat.send('Cherche', noMail);
    await pumpEventQueue();
    engine.live!.add(called('c1', 'search_messages', 'devis'));
    await pumpEventQueue();
    final running = chat.chat.last.tools.single;
    expect((running.name, running.summary), ('search_messages', 'devis'));
    expect(running.state, ToolState.running);
    engine.live!
      ..add(finished('c1', 'ok', results: 2, sources: ['m1', 'm2']))
      ..add({'type': 'done', 'reason': 'end_turn'})
      ..close();
    await sending;
    final done = chat.chat.last.tools.single;
    expect((done.state, done.results), (ToolState.ok, 2));
  });

  test('collects_sources_without_duplicates', () async {
    await play([
      called('c1', 'search_messages'),
      finished('c1', 'ok', results: 2, sources: ['m1', 'm2']),
      called('c2', 'get_thread'),
      finished('c2', 'ok', results: 2, sources: ['m2', 'm3']),
    ]);
    expect(chat.chat.last.sources, ['m1', 'm2', 'm3']);
  });

  test('denied_activity_keeps_the_refused_kind', () async {
    await play([
      called('c1', 'search_messages'),
      finished('c1', 'denied', denied: 'mail_list'),
    ]);
    final tool = chat.chat.last.tools.single;
    expect((tool.state, tool.deniedKind), (ToolState.denied, 'mail_list'));
    expect(chat.chat.last.sources, isEmpty);
  });

  test('finish_without_call_creates_the_activity', () async {
    await play([finished('c9', 'error')]);
    final tool = chat.chat.last.tools.single;
    expect((tool.id, tool.state), ('c9', ToolState.error));
  });

  test('history_sent_to_model_ignores_tool_activity', () async {
    await play([
      called('c1', 'search_messages', 'devis'),
      finished('c1', 'ok', results: 1, sources: ['m1']),
      {'type': 'text', 'text': 'Voici.'},
    ]);
    engine.manual = false;
    engine.scripts.add(const []);
    await chat.send('Merci', noMail);
    final sent = (engine.chatStarts[1]['request'] as Map)['messages'] as List;
    expect(sent.length, 3);
    expect(sent.toString().contains('search_messages'), isFalse);
  });
}

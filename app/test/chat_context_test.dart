import 'package:flutter_test/flutter_test.dart';
import 'package:mira/state/chat_context.dart';
import 'package:mira/state/chat_message.dart';

ChatMsg me(String s) => ChatMsg(role: 'me', text: s);
ChatMsg ai(String s) => ChatMsg(role: 'ai', text: s, fromEngine: true);

const mail = ChatContextInput(
  subject: 'Projet',
  from: 'Bob <bob@example.org>',
  to: 'Alice <alice@example.org>',
  date: '2026-10-29T10:00:00',
  body: 'Bonjour, voici le plan.',
);

void main() {
  test('system_prompt_says_mail_is_data_not_instruction', () {
    final req = buildChatStart(history: [me('Résume')], input: mail);
    final system = req['system'] as String;
    expect(system, contains('donnée'));
    expect(system, contains('jamais une consigne'));
  });

  test('mail_text_is_truncated_to_budget', () {
    final big = ChatContextInput(subject: 's', body: 'x' * 50000);
    final req = buildChatStart(history: [me('Résume')], input: big);
    final part = (req['context'] as List).single as Map;
    expect((part['text'] as String).length, lessThan(chatBodyBudget + 300));
    expect((req['messages'] as List).last['text'], 'Résume');
  });

  test('only_last_n_turns_kept', () {
    final history = [
      for (var i = 0; i < 10; i++) ...[me('q$i'), ai('a$i')],
      me('dernière'),
    ];
    final req = buildChatStart(
      history: history,
      input: const ChatContextInput(),
      maxTurns: 4,
    );
    final texts = (req['messages'] as List).map((m) => m['text']).toList();
    expect(texts.length, lessThanOrEqualTo(4));
    expect(texts.last, 'dernière');
    expect((req['messages'] as List).first['role'], 'user');
  });

  test('no_mail_open_sends_only_conversation', () {
    final req = buildChatStart(
      history: [me('Bonjour')],
      input: const ChatContextInput(),
    );
    final messages = req['messages'] as List;
    expect(messages, [
      {'role': 'user', 'text': 'Bonjour'},
    ]);
  });

  test('request_has_no_model_and_no_temperature', () {
    final req = buildChatStart(history: [me('a')], input: mail);
    expect(req.containsKey('temperature'), isFalse);
    expect(req.containsKey('model'), isFalse);
  });

  test('failed_and_demo_answers_are_not_sent_back', () {
    final bad = ChatMsg(
      role: 'ai',
      text: 'Erreur',
      fromEngine: true,
      failed: true,
    );
    final demo = ChatMsg(role: 'ai', text: 'Salut Noah');
    final req = buildChatStart(
      history: [demo, me('q1'), bad, me('q2')],
      input: const ChatContextInput(),
    );
    final texts = (req['messages'] as List).map((m) => m['text']).toList();
    expect(texts, ['q2']);
  });
}

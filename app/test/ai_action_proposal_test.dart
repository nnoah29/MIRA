import 'package:flutter_test/flutter_test.dart';
import 'package:mira/engine/ai_admin.dart';
import 'package:mira/state/chat_controller.dart';
import 'package:mira/state/chat_message.dart';
import 'package:mira/state/chat_message_json.dart';

import 'chat_history_support.dart';
import 'fake_ai_engine.dart';

Map<String, dynamic> actionLine({
  String id = 'c1',
  String action = 'archive',
}) => {
  'type': 'action_proposed',
  'id': id,
  'proposal': {
    'action': action,
    'messages': ['m1', 'm2'],
    'reason': 'Newsletters déjà lues',
  },
};

List<Map<String, dynamic>> turn(List<Map<String, dynamic>> middle) => [
  {'type': 'started', 'operation': 'op-1'},
  {
    'type': 'tool_called',
    'id': 'c1',
    'name': 'propose_mailbox_action',
    'summary': 'Newsletters',
  },
  ...middle,
  {
    'type': 'tool_finished',
    'id': 'c1',
    'state': 'ok',
    'results': 1,
    'sources': [],
  },
  {'type': 'text', 'text': 'À toi de confirmer.'},
  {'type': 'done', 'reason': 'end_turn'},
];

void main() {
  test('chat_parses_action_proposed', () async {
    final engine = FakeAiEngine()..scripts.add(turn([actionLine()]));
    final events = await AiAdmin(engine)
        .chat(const {}, connection: 'ai-1')
        .toList();
    final p = events.whereType<AiActionProposal>().single;
    expect(p.id, 'c1');
    expect(p.action, 'archive');
    expect(p.messages, ['m1', 'm2']);
    expect(p.reason, 'Newsletters déjà lues');
  });

  test('the_proposal_is_attached_as_a_pending_action', () async {
    final r = rig();
    r.engine.scripts.add(turn([actionLine()]));
    await r.chat.send('archive mes newsletters', noContext);
    final reply = r.chat.chat.last;
    expect(reply.actions, hasLength(1));
    expect(reply.actions.single.proposal.action, 'archive');
    expect(reply.actions.single.status, 'pending');
  });

  test('a_second_proposal_for_the_same_call_replaces_the_first', () async {
    final r = rig();
    r.engine.scripts.add(turn([actionLine(), actionLine(action: 'trash')]));
    await r.chat.send('range', noContext);
    expect(r.chat.chat.last.actions.map((a) => a.proposal.action), ['trash']);
  });

  test('status_survives_the_saved_conversation', () {
    final reply = ChatMsg(role: 'ai', text: 'Prêt', fromEngine: true);
    final item = ProposedAction(AiActionProposal.fromLine(actionLine()))
      ..status = 'done';
    reply.actions.add(item);
    final back = decodeChat(encodeChat([reply]), chatPayloadVersion)!.single;
    expect(back.actions.single.status, 'done');
    expect(back.actions.single.proposal.messages, ['m1', 'm2']);
    expect(back.actions.single.proposal.reason, 'Newsletters déjà lues');
  });

  test('a_conversation_saved_without_actions_still_loads', () {
    final back = decodeChat(
      '{"messages":[{"role":"ai","text":"x"}]}',
      chatPayloadVersion,
    )!;
    expect(back.single.actions, isEmpty);
  });
}

import 'package:flutter_test/flutter_test.dart';
import 'package:intl/date_symbol_data_local.dart';
import 'package:mira/engine/ai_admin.dart';
import 'package:mira/state/chat_context.dart';
import 'package:mira/state/chat_controller.dart';
import 'package:mira/state/chat_message.dart';
import 'package:mira/state/chat_message_json.dart';

import 'chat_history_support.dart';
import 'fake_ai_engine.dart';

Map<String, dynamic> proposalLine({String id = 'c1', String kind = 'new'}) => {
  'type': 'draft_proposed',
  'id': id,
  'proposal': {
    'kind': kind,
    'to': [
      {'name': 'Mon boss', 'address': 'boss@example.org'},
    ],
    'cc': null,
    'subject': 'Fin de stage',
    'body': 'Bonjour,\n\nMon stage finit mercredi.',
    'selection': null,
    'reply_to': 'm-1',
    'identity': null,
  },
};

List<Map<String, dynamic>> turn(List<Map<String, dynamic>> middle) => [
  {'type': 'started', 'operation': 'op-1'},
  {
    'type': 'tool_called',
    'id': 'c1',
    'name': 'compose_draft',
    'summary': 'Fin de stage',
  },
  ...middle,
  {
    'type': 'tool_finished',
    'id': 'c1',
    'state': 'ok',
    'results': 1,
    'sources': [],
  },
  {'type': 'text', 'text': 'Le brouillon est prêt.'},
  {'type': 'done', 'reason': 'end_turn'},
];

void main() {
  setUpAll(() => initializeDateFormatting('fr_FR'));

  test('chat_parses_draft_proposed_and_forwards_the_mode', () async {
    final engine = FakeAiEngine()..scripts.add(turn([proposalLine()]));
    final events = await AiAdmin(engine)
        .chat(
          {'messages': const []},
          connection: 'ai-1',
          mode: {'kind': 'composer', 'selection': true},
        )
        .toList();
    final proposal = events.whereType<AiDraftProposal>().single;
    expect(proposal.id, 'c1');
    expect(proposal.kind, 'new');
    expect(proposal.to!.single.name, 'Mon boss');
    expect(proposal.to!.single.address, 'boss@example.org');
    expect(proposal.cc, isNull);
    expect(proposal.subject, 'Fin de stage');
    expect(proposal.body, startsWith('Bonjour'));
    expect(proposal.replyTo, 'm-1');
    expect(engine.chatStarts.single['mode'], {
      'kind': 'composer',
      'selection': true,
    });
  });

  test('without_a_mode_the_start_has_none_and_missing_fields_parse', () async {
    final engine = FakeAiEngine();
    final line = proposalLine(kind: 'edit');
    (line['proposal'] as Map)
      ..remove('to')
      ..remove('reply_to')
      ..['subject'] = null;
    engine.scripts.add([line]);
    final events = await AiAdmin(engine).chat(const {}).toList();
    final proposal = events.single as AiDraftProposal;
    expect(proposal.kind, 'edit');
    expect(proposal.to, isNull);
    expect(proposal.replyTo, isNull);
    expect(proposal.subject, isNull);
    expect(engine.chatStarts.single.containsKey('mode'), isFalse);
  });

  test('proposal_is_attached_to_the_reply_next_to_its_tool_chip', () async {
    final r = rig();
    r.engine.scripts.add(turn([proposalLine()]));
    await r.chat.send('écris à mon boss', noContext);
    final reply = r.chat.chat.last;
    expect(reply.drafts, hasLength(1));
    expect(reply.drafts.single.subject, 'Fin de stage');
    expect(reply.tools.single.name, 'compose_draft');
    expect(reply.text, 'Le brouillon est prêt.');
  });

  test('a_second_proposal_for_the_same_call_replaces_the_first', () async {
    final r = rig();
    final second = proposalLine();
    (second['proposal'] as Map)['subject'] = 'Fin de stage (v2)';
    r.engine.scripts.add(turn([proposalLine(), second]));
    await r.chat.send('écris', noContext);
    expect(r.chat.chat.last.drafts.map((d) => d.subject), [
      'Fin de stage (v2)',
    ]);
  });

  test('drafts_survive_the_saved_conversation', () async {
    final reply = ChatMsg(role: 'ai', text: 'Prêt', fromEngine: true);
    final event = AiDraftProposal.fromLine(proposalLine());
    reply.drafts.add(event);
    final back = decodeChat(encodeChat([reply]), chatPayloadVersion)!.single;
    expect(back.drafts, hasLength(1));
    expect(back.drafts.single.id, 'c1');
    expect(back.drafts.single.to!.single.address, 'boss@example.org');
    expect(back.drafts.single.body, event.body);
  });

  test('the_system_prompt_gives_today_s_date_and_mentions_the_draft_tool', () {
    final prompt = chatSystemPrompt(DateTime(2026, 11, 1));
    expect(prompt, contains('2026'));
    expect(prompt, contains('novembre'));
    expect(prompt, contains('brouillon'));
    expect(prompt, isNot(contains('proposes seulement du texte')));
    expect(prompt, contains('[mail:ID]'));
  });
}

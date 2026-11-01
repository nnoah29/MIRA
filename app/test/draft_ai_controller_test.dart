import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:intl/date_symbol_data_local.dart';
import 'package:mira/data/models.dart';
import 'package:mira/engine/ai_admin.dart';
import 'package:mira/state/draft_ai_controller.dart';
import 'package:mira/state/rich_controller.dart';
import 'package:mira/state/store.dart';

import 'fake_ai_engine.dart';

const signature = 'Cordialement,\nNoah';

Map<String, dynamic> proposal(
  Map<String, Object?> fields, {
  String id = 'c1',
}) => {
  'type': 'draft_proposed',
  'id': id,
  'proposal': {
    'kind': 'edit',
    'to': null,
    'cc': null,
    'subject': null,
    'body': null,
    'selection': null,
    'reply_to': null,
    'identity': null,
    ...fields,
  },
};

List<Map<String, dynamic>> turn(
  List<Map<String, dynamic>> middle, {
  String text = '',
}) => [
  {'type': 'started', 'operation': 'op-9'},
  ...middle,
  if (text.isNotEmpty) {'type': 'text', 'text': text},
  {'type': 'done', 'reason': 'end_turn'},
];

ComposeState draft({
  String body = '\n\n$signature',
  String subject = '',
  String mode = 'new',
  String sig = signature,
}) {
  final c = ComposeState(
    mode: mode,
    to: [],
    cc: [],
    bcc: [],
    atts: [],
    from: 'moi@example.org',
  );
  c.subject.text = subject;
  c.body.setContent(body, RichController.gapFlags(body));
  c.sigText = sig;
  return c;
}

void main() {
  setUpAll(() => initializeDateFormatting('fr_FR'));
  late FakeAiEngine engine;
  late ComposeState c;
  late int touched;
  late String? connectionId;

  DraftAiController make() => DraftAiController(
    admin: () => AiAdmin(engine),
    connection: () async => connectionId,
    compose: () => c,
    onChanged: () => touched++,
  );

  setUp(() {
    engine = FakeAiEngine();
    c = draft();
    touched = 0;
    connectionId = 'ai-1';
  });

  test('whole_draft_instruction_fills_subject_recipients_and_body_before_the_signature', () async {
    engine.scripts.add(
      turn([
        proposal({
          'subject': 'Fin de stage',
          'to': [
            {'name': 'Mon boss', 'address': 'boss@example.org'},
          ],
          'body': 'Bonjour,\n\nMon stage se termine mercredi.',
        }),
      ]),
    );
    final ai = make();
    await ai.run('dis à mon boss que mon stage finit mercredi');
    expect(c.subject.text, 'Fin de stage');
    expect(c.to.single.email, 'boss@example.org');
    expect(c.to.single.name, 'Mon boss');
    expect(
      c.body.text,
      'Bonjour,\n\nMon stage se termine mercredi.\n\n$signature',
    );
    expect(
      c.body.selection.baseOffset,
      'Bonjour,\n\nMon stage se termine mercredi.'.length,
    );
    expect(ai.canUndo, isTrue);
    expect(ai.busy, isFalse);
    expect(touched, greaterThan(0));
    final start = engine.chatStarts.single;
    expect(start['mode'], {'kind': 'composer', 'selection': false});
    final request = start['request'] as Map;
    expect(
      (request['messages'] as List).single['text'],
      contains('mon stage finit mercredi'),
    );
    final part = (request['context'] as List).single as Map;
    expect(part['kind'], 'mail_message');
    expect(request['system'], contains('edit_draft'));
    expect(request['system'], contains('novembre'));
  });

  test('undo_restores_text_flags_subject_and_recipients_exactly', () async {
    c.subject.text = 'Avant';
    c.to = [const Person('Ana', 'ana@example.org')];
    c.body.setContent(
      'Texte gras\n\n$signature',
      List<int>.generate(
        'Texte gras\n\n$signature'.length,
        (i) => i < 5 ? 1 : 0,
      ),
    );
    final textBefore = c.body.text;
    final flagsBefore = List<int>.from(c.body.flags);
    engine.scripts.add(
      turn([
        proposal({
          'subject': 'Après',
          'to': [
            {'name': null, 'address': 'zoe@example.org'},
          ],
          'body': 'Neuf',
        }),
      ]),
    );
    final ai = make();
    await ai.run('réécris');
    expect(c.subject.text, 'Après');
    expect(c.to.single.email, 'zoe@example.org');
    ai.undo();
    expect(c.subject.text, 'Avant');
    expect(c.to.single.email, 'ana@example.org');
    expect(c.body.text, textBefore);
    expect(c.body.flags, flagsBefore);
    expect(ai.canUndo, isFalse);
    expect(touched, greaterThanOrEqualTo(2));
  });

  test('selection_replacement_changes_only_the_selected_range', () async {
    const text = 'Salut Marie, merci pour tout.\n\nCordialement';
    c = draft(body: text);
    final start = text.indexOf('merci pour tout');
    final selected = DraftSelection(
      start,
      start + 'merci pour tout'.length,
      'merci pour tout',
    );
    engine.scripts.add(
      turn([
        proposal({'selection': 'un grand merci'}),
      ]),
    );
    final ai = make();
    await ai.run('reformule', selection: selected);
    expect(c.body.text, 'Salut Marie, un grand merci.\n\nCordialement');
    expect(engine.chatStarts.single['mode'], {
      'kind': 'composer',
      'selection': true,
    });
    final request = engine.chatStarts.single['request'] as Map;
    expect(
      (request['messages'] as List).single['text'],
      contains('merci pour tout'),
    );
    ai.undo();
    expect(c.body.text, text);
  });

  test(
    'a_moved_selection_is_found_again_and_a_changed_one_is_refused',
    () async {
      const text = 'Salut Marie, merci pour tout.';
      c = draft(body: text);
      final from = text.indexOf('merci');
      final selected = DraftSelection(from, from + 5, 'merci');
      engine.manual = true;
      final ai = make();
      final running = ai.run('reformule', selection: selected);
      await pumpEventQueue();
      c.body.value = c.body.value.copyWith(
        text: 'Bonjour. $text',
        selection: const TextSelection.collapsed(offset: 0),
      );
      engine.live!
        ..add({'type': 'started', 'operation': 'op-9'})
        ..add(proposal({'selection': 'grand merci'}))
        ..add({'type': 'done', 'reason': 'end_turn'})
        ..close();
      await running;
      expect(c.body.text, 'Bonjour. Salut Marie, grand merci pour tout.');
      final before = c.body.text;
      engine.manual = false;
      engine.scripts.add(
        turn([
          proposal({'selection': 'xx'}),
        ]),
      );
      await ai.run('encore', selection: DraftSelection(0, 5, 'absent'));
      expect(c.body.text, before);
      expect(ai.error, isNotNull);
    },
  );

  test('no_proposal_shows_the_ai_reply_and_changes_nothing', () async {
    engine.scripts.add(turn(const [], text: 'Quelle est son adresse ?'));
    final ai = make();
    final before = c.body.text;
    await ai.run('écris à mon boss');
    expect(ai.reply, 'Quelle est son adresse ?');
    expect(c.body.text, before);
    expect(ai.canUndo, isFalse);
    expect(ai.error, isNull);
  });

  test('an_engine_error_is_translated_and_nothing_changes', () async {
    engine.scripts.add([
      {
        'type': 'error',
        'error': {
          'code': 'incompatible_state.ai_consent_required',
          'message': 'x',
        },
      },
    ]);
    final ai = make();
    await ai.run('écris');
    expect(ai.error, 'Autorise d’abord l’envoi de texte à ce service d’IA');
    expect(ai.busy, isFalse);
    expect(ai.canUndo, isFalse);
  });

  test('without_a_usable_connection_nothing_starts', () async {
    connectionId = null;
    final ai = make();
    await ai.run('écris');
    expect(ai.error, isNotNull);
    expect(engine.chatStarts, isEmpty);
    expect(ai.busy, isFalse);
  });

  test('stop_cancels_the_generation_and_keeps_the_draft', () async {
    engine.manual = true;
    final before = c.body.text;
    final ai = make();
    final running = ai.run('écris');
    await pumpEventQueue();
    engine.live!.add({'type': 'started', 'operation': 'op-9'});
    await pumpEventQueue();
    await ai.stop();
    engine.live!.add(proposal({'body': 'trop tard'}));
    await running;
    expect(ai.busy, isFalse);
    expect(c.body.text, before);
    expect(engine.requestsOf('cancel_operation'), isNotEmpty);
  });

  test('only_the_last_proposal_of_a_turn_is_applied', () async {
    engine.scripts.add(
      turn([
        proposal({'body': 'Premier jet'}),
        proposal({'body': 'Version finale'}, id: 'c2'),
      ]),
    );
    final ai = make();
    await ai.run('écris');
    expect(c.body.text, 'Version finale\n\n$signature');
  });

  test('forwarded_text_after_the_signature_is_kept_and_shown_to_the_ai', () async {
    const marker = '---------- Message transféré ----------';
    final text =
        '\n\n$signature\n$marker\nDe : Ana\n\nContenu transféré important';
    c = draft(body: text, mode: 'fwd');
    final bold = c.body.text.indexOf('Contenu');
    final flags = List<int>.generate(text.length, (i) => i >= bold ? 1 : 0);
    c.body.setContent(text, flags);
    engine.scripts.add(
      turn([
        proposal({'body': 'Voici un mail à lire.'}),
      ]),
    );
    final ai = make();
    await ai.run('ajoute une phrase d’intro');
    expect(
      c.body.text,
      'Voici un mail à lire.\n\n$signature\n$marker\nDe : Ana\n\nContenu transféré important',
    );
    final at = c.body.text.indexOf('Contenu');
    expect(c.body.flags[at], 1);
    expect(c.body.flags[0], 0);
    final part =
        ((engine.chatStarts.single['request'] as Map)['context'] as List).single
            as Map;
    expect(part['text'], contains('Contenu transféré important'));
    ai.undo();
    expect(c.body.text, text);
    expect(c.body.flags, flags);
  });

  test('a_forward_without_signature_keeps_the_forwarded_part', () async {
    const marker = '---------- Message transféré ----------';
    final text = 'Mon intro\n\n$marker\nDe : Ana\n\nContenu transféré';
    c = draft(body: text, mode: 'fwd', sig: '');
    engine.scripts.add(
      turn([
        proposal({'body': 'Nouvelle intro'}),
      ]),
    );
    await make().run('réécris l’intro');
    expect(
      c.body.text,
      'Nouvelle intro\n\n$marker\nDe : Ana\n\nContenu transféré',
    );
  });

  test('the_summary_names_the_changed_fields_with_real_addresses', () async {
    engine.scripts.add(
      turn([
        proposal({
          'subject': 'Fin de stage',
          'to': [
            {'name': 'Support Banque', 'address': 'evil@x.com'},
          ],
          'cc': [
            {'name': null, 'address': 'rh@example.org'},
          ],
          'body': 'Texte',
        }),
      ]),
    );
    final ai = make();
    await ai.run('écris');
    expect(ai.summary, contains('evil@x.com'));
    expect(ai.summary, contains('rh@example.org'));
    expect(ai.summary, contains('Fin de stage'));
    ai.undo();
    expect(ai.summary, isNull);
  });
}

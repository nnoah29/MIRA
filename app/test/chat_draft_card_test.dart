import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mira/engine/ai_admin.dart';
import 'package:mira/state/chat_controller.dart';
import 'package:mira/state/store.dart';
import 'package:mira/ui/step_animator.dart';

import 'ai_panel_harness.dart';

Finder key(String k) => find.byKey(ValueKey(k));

Map<String, dynamic> proposal({
  String kind = 'new',
  String? replyTo,
  String? subject = 'Fin de stage',
  List<Map<String, dynamic>>? to,
  String? identity,
}) => {
  'type': 'draft_proposed',
  'id': 'c1',
  'proposal': {
    'kind': kind,
    'to':
        to ??
        [
          {'name': 'Mon boss', 'address': 'boss@example.org'},
        ],
    'cc': null,
    'subject': subject,
    'body': 'Bonjour,\n\nMon stage finit mercredi.',
    'selection': null,
    'reply_to': replyTo,
    'identity': identity,
  },
};

/// Panneau ouvert, une réponse de l'IA portant cette proposition.
Future<MailStore> withProposal(
  WidgetTester tester,
  Map<String, dynamic> line,
) async {
  final engine = withDefaultConnection()
    ..scripts.add([
      {'type': 'started', 'operation': 'op-1'},
      line,
      {'type': 'text', 'text': 'Le brouillon est prêt.'},
      {'type': 'done', 'reason': 'end_turn'},
    ]);
  final store = await pumpPanel(tester, engine);
  await tester.pump(const Duration(milliseconds: 300));
  await store.chatCtl.send('écris à mon boss', noContext);
  await tester.pump(const Duration(milliseconds: 300));
  return store;
}

void main() {
  setUpAll(() => StepAnimator.frameCap = null);

  testWidgets('card_shows_recipient_subject_and_excerpt', (tester) async {
    final store = await withProposal(tester, proposal());
    final card = key('chat-draft-c1');
    expect(card, findsOneWidget);
    expect(
      find.descendant(of: card, matching: find.textContaining('Mon boss')),
      findsOneWidget,
    );
    expect(
      find.descendant(of: card, matching: find.textContaining('Fin de stage')),
      findsOneWidget,
    );
    expect(
      find.descendant(
        of: card,
        matching: find.textContaining('Mon stage finit mercredi'),
      ),
      findsOneWidget,
    );
    await close(tester, store);
  });

  testWidgets('open_button_opens_a_prefilled_new_message_with_one_signature', (
    tester,
  ) async {
    final store = await withProposal(tester, proposal());
    await tester.tap(key('chat-draft-open-c1'));
    await tester.pump(const Duration(milliseconds: 300));
    final c = store.c!;
    expect(store.view, PaneView.compose);
    expect(c.mode, 'new');
    expect(c.subject.text, 'Fin de stage');
    expect(c.to.single.email, 'boss@example.org');
    expect(c.body.text, startsWith('Bonjour,\n\nMon stage finit mercredi.'));
    expect('Cordialement'.allMatches(c.body.text), hasLength(1));
    expect(c.body.text.trimRight(), endsWith(store.sig.trim()));
    await close(tester, store);
  });

  testWidgets('reply_target_opens_a_reply_with_the_quote', (tester) async {
    final store = await withProposal(tester, proposal(subject: null));
    final original = store.mails.first;
    store.chatCtl.chat.last.drafts
      ..clear()
      ..add(
        AiDraftProposal.fromLine(
          proposal(replyTo: original.id, subject: null, to: const []),
        ),
      );
    store.openProposal(store.chatCtl.chat.last.drafts.single);
    final c = store.c!;
    expect(c.mode, 'reply');
    expect(c.replyTo, original.id);
    expect(c.quote, isNotNull);
    expect(c.subject.text, startsWith('Re:'));
    expect(c.to, isNotEmpty);
    await close(tester, store);
  });

  testWidgets('unknown_reply_target_falls_back_to_a_new_message', (
    tester,
  ) async {
    final store = await withProposal(tester, proposal(replyTo: 'inconnu'));
    store.openProposal(store.chatCtl.chat.last.drafts.single);
    expect(store.c!.mode, 'new');
    expect(store.c!.replyTo, isNull);
    expect(store.c!.subject.text, 'Fin de stage');
    await close(tester, store);
  });

  testWidgets('edit_proposals_and_wrong_identities_are_handled', (
    tester,
  ) async {
    final store = await withProposal(
      tester,
      proposal(kind: 'edit', identity: 'nobody@nowhere.test'),
    );
    expect(key('chat-draft-c1'), findsNothing);
    store.openProposal(store.chatCtl.chat.last.drafts.single);
    expect(store.c!.from, isNotEmpty);
    expect(store.c!.from, isNot('nobody@nowhere.test'));
    await close(tester, store);
  });

  testWidgets('card_always_shows_the_real_address_next_to_the_name', (
    tester,
  ) async {
    final store = await withProposal(
      tester,
      proposal(
        to: [
          {'name': 'Support Banque', 'address': 'evil@x.com'},
        ],
      ),
    );
    final card = key('chat-draft-c1');
    expect(
      find.descendant(
        of: card,
        matching: find.textContaining('Support Banque <evil@x.com>'),
      ),
      findsOneWidget,
    );
    await close(tester, store);
  });
}

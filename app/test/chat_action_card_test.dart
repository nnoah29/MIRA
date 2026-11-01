import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mira/engine/ai_admin.dart';
import 'package:mira/state/chat_controller.dart';
import 'package:mira/state/store.dart';
import 'package:mira/ui/step_animator.dart';

import 'ai_panel_harness.dart';

Finder key(String k) => find.byKey(ValueKey(k));

Future<void> settle(WidgetTester tester) async {
  await tester.pump(const Duration(milliseconds: 300));
  await tester.pump(const Duration(milliseconds: 50));
}

/// Panneau ouvert ; l'IA « a lu » ces messages (sources) puis propose cette action.
Future<MailStore> withAction(
  WidgetTester tester, {
  required String action,
  required List<String> Function(MailStore) ids,
  List<String> Function(MailStore)? read,
}) async {
  final engine = withDefaultConnection();
  final store = await pumpPanel(tester, engine);
  await tester.pump(const Duration(milliseconds: 300));
  engine.scripts.add([
    {'type': 'started', 'operation': 'op-1'},
    {
      'type': 'tool_called',
      'id': 'c1',
      'name': 'propose_mailbox_action',
      'summary': 'x',
    },
    {
      'type': 'tool_finished',
      'id': 'r1',
      'state': 'ok',
      'results': 2,
      'sources': (read ?? ids)(store),
    },
    {
      'type': 'action_proposed',
      'id': 'c1',
      'proposal': {
        'action': action,
        'messages': ids(store),
        'reason': 'Ils sont lus',
      },
    },
    {'type': 'text', 'text': 'À toi de confirmer.'},
    {'type': 'done', 'reason': 'end_turn'},
  ]);
  await store.chatCtl.send('range ma boîte', noContext);
  await settle(tester);
  return store;
}

void main() {
  setUpAll(() => StepAnimator.frameCap = null);

  testWidgets('card_shows_the_count_the_reason_and_the_subjects', (
    tester,
  ) async {
    final store = await withAction(
      tester,
      action: 'archive',
      ids: (s) => [s.mails[0].id, s.mails[1].id],
    );
    final card = key('chat-action-c1');
    expect(card, findsOneWidget);
    expect(
      find.descendant(of: card, matching: find.textContaining('2 messages')),
      findsOneWidget,
    );
    expect(
      find.descendant(of: card, matching: find.textContaining('Ils sont lus')),
      findsOneWidget,
    );
    expect(
      find.descendant(
        of: card,
        matching: find.textContaining(store.mails[0].subject),
      ),
      findsOneWidget,
    );
    await close(tester, store);
  });

  testWidgets('confirm_applies_the_action_through_the_app_and_marks_it_done', (
    tester,
  ) async {
    final store = await withAction(
      tester,
      action: 'archive',
      ids: (s) => [s.mails[0].id, s.mails[1].id],
    );
    final (a, b) = (store.mails[0], store.mails[1]);
    await tester.tap(key('chat-action-confirm-c1'));
    await settle(tester);
    expect(a.folder, 'archive');
    expect(b.folder, 'archive');
    expect(key('chat-action-done-c1'), findsOneWidget);
    expect(key('chat-action-confirm-c1'), findsNothing);
    expect(store.chatCtl.chat.last.actions.single.status, 'done');
    await close(tester, store);
  });

  testWidgets('ignore_changes_nothing', (tester) async {
    final store = await withAction(
      tester,
      action: 'trash',
      ids: (s) => [s.mails[0].id],
    );
    final mail = store.mails[0];
    final folder = mail.folder;
    await tester.tap(key('chat-action-dismiss-c1'));
    await settle(tester);
    expect(mail.folder, folder);
    expect(key('chat-action-dismissed-c1'), findsOneWidget);
    expect(store.chatCtl.chat.last.actions.single.status, 'dismissed');
    await close(tester, store);
  });

  testWidgets('ids_the_ai_never_read_are_not_touched', (tester) async {
    final store = await withAction(
      tester,
      action: 'trash',
      ids: (s) => [s.mails[0].id, s.mails[1].id],
      read: (s) => [s.mails[0].id],
    );
    final (read, unread) = (store.mails[0], store.mails[1]);
    final before = unread.folder;
    expect(
      find.descendant(
        of: key('chat-action-c1'),
        matching: find.textContaining('1 message'),
      ),
      findsOneWidget,
    );
    await tester.tap(key('chat-action-confirm-c1'));
    await settle(tester);
    expect(read.folder, 'trash');
    expect(unread.folder, before);
    await close(tester, store);
  });

  testWidgets('read_unread_and_star_actions_change_the_flags', (tester) async {
    final store = await withAction(
      tester,
      action: 'mark_unread',
      ids: (s) => [s.mails[2].id],
    );
    final mail = store.mails[2];
    mail.unread = false;
    await tester.tap(key('chat-action-confirm-c1'));
    await settle(tester);
    expect(mail.unread, isTrue);
    expect(
      store.applyActionProposal(
        AiActionProposal(id: 'x', action: 'star', messages: [mail.id]),
      ),
      1,
    );
    expect(mail.starred, isTrue);
    expect(
      store.applyActionProposal(
        AiActionProposal(id: 'y', action: 'unstar', messages: [mail.id]),
      ),
      1,
    );
    expect(mail.starred, isFalse);
    expect(
      store.applyActionProposal(
        AiActionProposal(id: 'z', action: 'mark_read', messages: [mail.id]),
      ),
      1,
    );
    expect(mail.unread, isFalse);
    await close(tester, store);
  });

  testWidgets('with_no_resolvable_message_the_card_cannot_be_confirmed', (
    tester,
  ) async {
    final store = await withAction(
      tester,
      action: 'archive',
      ids: (s) => ['inconnu-1'],
      read: (s) => const [],
    );
    expect(key('chat-action-c1'), findsOneWidget);
    expect(key('chat-action-confirm-c1'), findsNothing);
    expect(key('chat-action-dismiss-c1'), findsOneWidget);
    await close(tester, store);
  });

  testWidgets('an_unknown_action_cannot_be_confirmed', (tester) async {
    final store = await withAction(
      tester,
      action: 'delete_forever',
      ids: (s) => [s.mails[0].id],
    );
    expect(key('chat-action-c1'), findsOneWidget);
    expect(key('chat-action-confirm-c1'), findsNothing);
    expect(key('chat-action-dismiss-c1'), findsOneWidget);
    await close(tester, store);
  });
}

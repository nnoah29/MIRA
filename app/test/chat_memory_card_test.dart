import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mira/engine/ai_admin.dart';
import 'package:mira/state/chat_message.dart';
import 'package:mira/theme/app_theme.dart';
import 'package:mira/ui/chat_memory_card.dart';
import 'package:mira/ui/chat_tool_labels.dart';

AiMemoryProposal proposal(String action, {String? id, String kind = 'fact'}) =>
    AiMemoryProposal(
      id: 'c1',
      action: action,
      text: action == 'remember' ? 'TEXTE-A-RETENIR' : 'RAISON-D-OUBLI',
      kind: kind,
      memoryId: id,
    );

Future<void> show(
  WidgetTester tester,
  ProposedMemory item, {
  VoidCallback? onConfirm,
  VoidCallback? onDismiss,
}) => tester.pumpWidget(
  MaterialApp(
    theme: buildTheme(false),
    home: Scaffold(
      body: ChatMemoryCard(
        item,
        onConfirm: onConfirm ?? () {},
        onDismiss: onDismiss ?? () {},
      ),
    ),
  ),
);

void main() {
  testWidgets('a_remember_card_shows_the_text_and_asks_to_confirm', (t) async {
    var confirmed = 0;
    var dismissed = 0;
    await show(
      t,
      ProposedMemory(proposal('remember')),
      onConfirm: () => confirmed++,
      onDismiss: () => dismissed++,
    );
    expect(find.text('TEXTE-A-RETENIR'), findsOneWidget);
    expect(find.textContaining('RETENIR CECI'), findsOneWidget);
    await t.tap(find.byKey(const ValueKey('chat-memory-confirm-c1')));
    await t.tap(find.byKey(const ValueKey('chat-memory-dismiss-c1')));
    expect((confirmed, dismissed), (1, 1));
  });

  testWidgets('a_forget_card_shows_the_memory_text_and_the_reason', (t) async {
    final item = ProposedMemory(
      proposal('forget', id: 'mem-1'),
      target: 'MON-CHAT-MOCHI',
    );
    await show(t, item);
    expect(find.text('MON-CHAT-MOCHI'), findsOneWidget);
    expect(find.textContaining('RAISON-D-OUBLI'), findsOneWidget);
    expect(
      find.byKey(const ValueKey('chat-memory-confirm-c1')),
      findsOneWidget,
    );
  });

  testWidgets('a_missing_memory_can_only_be_dismissed', (t) async {
    final item = ProposedMemory(proposal('forget', id: 'x'), missing: true);
    await show(t, item);
    expect(find.byKey(const ValueKey('chat-memory-confirm-c1')), findsNothing);
    expect(
      find.byKey(const ValueKey('chat-memory-dismiss-c1')),
      findsOneWidget,
    );
  });

  testWidgets('settled_cards_show_their_outcome_and_no_buttons', (t) async {
    final done = ProposedMemory(proposal('remember'), status: 'done');
    await show(t, done);
    expect(find.byKey(const ValueKey('chat-memory-done-c1')), findsOneWidget);
    expect(find.byKey(const ValueKey('chat-memory-confirm-c1')), findsNothing);
    final no = ProposedMemory(proposal('remember'), status: 'dismissed');
    await show(t, no);
    expect(
      find.byKey(const ValueKey('chat-memory-dismissed-c1')),
      findsOneWidget,
    );
  });

  testWidgets('a_failed_confirmation_shows_its_error', (t) async {
    final item = ProposedMemory(proposal('remember'))..error = 'ERREUR-X';
    await show(t, item);
    expect(find.text('ERREUR-X'), findsOneWidget);
    expect(
      find.byKey(const ValueKey('chat-memory-confirm-c1')),
      findsOneWidget,
    );
  });

  test('the_memory_tools_have_readable_labels', () {
    String label(String name) => toolAction(ToolActivity(id: 'c', name: name));
    expect(label('recall_memory'), 'Mémoire consultée');
    expect(label('remember'), 'Proposition de souvenir');
    expect(label('forget_memory'), 'Proposition d’oubli');
  });
}

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mira/engine/engine_mail_source.dart';
import 'package:mira/state/store.dart';
import 'package:shared_preferences/shared_preferences.dart';

import 'account_panel_test.dart' show AccountsEngine;
import 'onboarding_flow_test.dart' show pumpFlow, tapKey;
import 'onboarding_test.dart' show open;

Future<MailStore> emptyStore(WidgetTester tester) async {
  SharedPreferences.setMockInitialValues({});
  final prefs = await tester.runAsync(SharedPreferences.getInstance);
  final src = await tester.runAsync(
    () => EngineMailSource.load(AccountsEngine()),
  );
  return MailStore(prefs!, source: src);
}

Future<void> toAccountPage(WidgetTester tester) async {
  for (var i = 0; i < 3; i++) {
    await tapKey(tester, 'onb-next');
  }
}

Future<void> settle(WidgetTester tester) async {
  await tester.pump();
  await tester.pump(const Duration(seconds: 5));
}

Future<void> addYahoo(WidgetTester tester) async {
  await tester.tap(find.byKey(const ValueKey('provider-yahoo')));
  await tester.pump();
  await tester.enterText(find.byKey(const ValueKey('field-name')), 'Alice');
  await tester.enterText(
    find.byKey(const ValueKey('field-address')),
    'alice@yahoo.com',
  );
  await tester.enterText(find.byKey(const ValueKey('field-password')), 'pw');
  await tester.pump();
  await tester.ensureVisible(find.byKey(const ValueKey('btn-add')));
  await tester.tap(find.byKey(const ValueKey('btn-add')));
  await tester.pump();
  await tester.pump(const Duration(milliseconds: 600));
}

void main() {
  testWidgets('adding_an_account_moves_to_the_next_page', (tester) async {
    final store = await emptyStore(tester);
    await pumpFlow(tester, store: store);
    await toAccountPage(tester);
    expect(find.byKey(const ValueKey('onb-account-panel')), findsOneWidget);
    await addYahoo(tester);
    expect(find.byKey(const ValueKey('onb-account-panel')), findsNothing);
    expect(store.accounts.any((a) => a.email == 'alice@yahoo.com'), isTrue);
    await settle(tester);
  });

  testWidgets('later_link_moves_on_without_account', (tester) async {
    await pumpFlow(tester, store: await emptyStore(tester));
    await toAccountPage(tester);
    await tapKey(tester, 'onb-account-later');
    expect(find.byKey(const ValueKey('onb-account-panel')), findsNothing);
  });

  testWidgets('replay_with_accounts_shows_count_and_keeps_next', (
    tester,
  ) async {
    final store = await tester.runAsync(() => open(accounts: true));
    await pumpFlow(tester, store: store);
    await toAccountPage(tester);
    expect(find.text('Comptes déjà ajoutés : 1'), findsOneWidget);
    expect(find.byKey(const ValueKey('onb-account-panel')), findsNothing);
    await tapKey(tester, 'onb-next');
    expect(find.text('Comptes déjà ajoutés : 1'), findsNothing);
  });

  testWidgets('back_to_account_after_adding_shows_added_state', (tester) async {
    await pumpFlow(tester, store: await emptyStore(tester));
    await toAccountPage(tester);
    await addYahoo(tester);
    await tapKey(tester, 'onb-back');
    expect(find.text('Comptes déjà ajoutés : 1'), findsOneWidget);
    expect(find.byKey(const ValueKey('onb-account-panel')), findsNothing);
    await tapKey(tester, 'onb-account-add-another');
    expect(find.byKey(const ValueKey('onb-account-panel')), findsOneWidget);
    await settle(tester);
  });
}

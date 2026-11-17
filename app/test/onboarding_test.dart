import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:google_fonts/google_fonts.dart';
import 'package:mira/engine/engine_mail_source.dart';
import 'package:mira/state/store.dart';
import 'package:mira/theme/app_theme.dart';
import 'package:mira/ui/app_root.dart';
import 'package:provider/provider.dart';
import 'package:shared_preferences/shared_preferences.dart';

import 'app_test.dart' show teardown;
import 'engine_source_test.dart' show FakeEngine;

Future<void> pumpRoot(WidgetTester tester, MailStore store) async {
  GoogleFonts.config.allowRuntimeFetching = false;
  await tester.runAsync(Fonts.preload);
  tester.view.physicalSize = const Size(1200, 900);
  tester.view.devicePixelRatio = 1;
  addTearDown(tester.view.resetPhysicalSize);
  await tester.pumpWidget(
    ChangeNotifierProvider.value(
      value: store,
      child: MaterialApp(theme: buildTheme(true), home: const AppRoot()),
    ),
  );
  await tester.pump(const Duration(milliseconds: 400));
}

Future<MailStore> open({
  bool accounts = false,
  Map<String, Object> prefs = const {},
}) async {
  SharedPreferences.setMockInitialValues(prefs);
  final p = await SharedPreferences.getInstance();
  final fake = FakeEngine()..noAccounts = !accounts;
  final store = MailStore(p, source: await EngineMailSource.load(fake));
  await store.localLabelsReady;
  return store;
}

void main() {
  test('premier lancement sans compte : l’accueil s’ouvre', () async {
    expect((await open()).welcomeOpen, isTrue);
  });

  test('l’accueil ne s’ouvre pas s’il a déjà été vu', () async {
    expect((await open(prefs: {'welcomed': '1'})).welcomeOpen, isFalse);
  });

  test('l’accueil ne s’ouvre pas quand un compte existe', () async {
    expect((await open(accounts: true)).welcomeOpen, isFalse);
  });

  test('la démonstration n’a pas d’accueil', () async {
    SharedPreferences.setMockInitialValues({});
    final store = MailStore(await SharedPreferences.getInstance());
    expect(store.welcomeOpen, isFalse);
  });

  test('« Ajouter un compte » ferme l’accueil et ouvre l’ajout', () async {
    final store = await open();
    store.finishWelcome(addAccount: true);
    expect(store.welcomeOpen, isFalse);
    expect(store.wantsAddAccount, isTrue);
    expect(store.view, PaneView.settings);
    expect(store.prefs.getString('welcomed'), '1');
  });

  test('« Plus tard » ferme l’accueil sans ouvrir l’ajout', () async {
    final store = await open();
    store.finishWelcome(addAccount: false);
    expect(store.welcomeOpen, isFalse);
    expect(store.wantsAddAccount, isFalse);
  });

  test('l’accueil peut être rejoué', () async {
    final store = await open(prefs: {'welcomed': '1'});
    store.replayWelcome();
    expect(store.welcomeOpen, isTrue);
  });

  testWidgets('app_root_shows_flow_then_shell', (tester) async {
    final store = await tester.runAsync(open) as MailStore;
    await pumpRoot(tester, store);
    expect(find.text('TES MAILS. CHEZ TOI.'), findsOneWidget);
    await tester.tap(find.byKey(const ValueKey('onb-skip')));
    await tester.pump();
    await tester.pump(const Duration(milliseconds: 600));
    expect(store.welcomeOpen, isFalse);
    expect(find.text('TES MAILS. CHEZ TOI.'), findsNothing);
    await teardown(tester, store);
  });

  testWidgets('replay_restarts_at_first_page', (tester) async {
    final store = await tester.runAsync(open) as MailStore;
    await pumpRoot(tester, store);
    await tester.tap(find.byKey(const ValueKey('onb-next')));
    await tester.pump();
    await tester.pump(const Duration(milliseconds: 600));
    await tester.tap(find.byKey(const ValueKey('onb-skip')));
    await tester.pump();
    await tester.pump(const Duration(milliseconds: 600));
    store.replayWelcome();
    await tester.pump();
    await tester.pump(const Duration(milliseconds: 600));
    expect(find.text('TES MAILS. CHEZ TOI.'), findsOneWidget);
    await teardown(tester, store);
  });

  testWidgets('closing_before_the_end_does_not_mark_welcomed', (tester) async {
    final store = await tester.runAsync(open) as MailStore;
    await pumpRoot(tester, store);
    await tester.pumpWidget(const SizedBox());
    expect(store.prefs.getString('welcomed'), isNull);
    expect(store.welcomeOpen, isTrue);
    store.dispose();
  });
}

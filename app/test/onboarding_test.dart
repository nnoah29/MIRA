import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mira/engine/engine_mail_source.dart';
import 'package:mira/state/store.dart';
import 'package:mira/theme/app_theme.dart';
import 'package:mira/ui/about_settings.dart';
import 'package:mira/ui/welcome_page.dart';
import 'package:provider/provider.dart';
import 'package:shared_preferences/shared_preferences.dart';

import 'app_test.dart' show pumpApp, teardown;
import 'engine_source_test.dart' show FakeEngine;

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

  testWidgets('l’écran d’accueil mène à l’ajout de compte', (tester) async {
    final store = await pumpApp(tester);
    await tester.pumpWidget(
      ChangeNotifierProvider.value(
        value: store,
        child: MaterialApp(theme: buildTheme(true), home: const WelcomePage()),
      ),
    );
    expect(find.text('BIENVENUE DANS MIRA'), findsOneWidget);
    openLink = (_) async => true;
    await tester.tap(find.byKey(const ValueKey('welcome-add')));
    await tester.pump();
    expect(store.welcomeOpen, isFalse);
    expect(store.wantsAddAccount, isTrue);
    await teardown(tester, store);
  });
}

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:google_fonts/google_fonts.dart';
import 'package:intl/date_symbol_data_local.dart';
import 'package:mira/data/models.dart';
import 'package:mira/i18n/i18n.dart';
import 'package:mira/main.dart';
import 'package:mira/state/store.dart';
import 'package:mira/theme/app_theme.dart';
import 'package:provider/provider.dart';
import 'package:shared_preferences/shared_preferences.dart';

Future<MailStore> pumpApp(
  WidgetTester tester, {
  Size size = const Size(1500, 950),
  Map<String, Object> prefs = const {},
}) async {
  GoogleFonts.config.allowRuntimeFetching = false;
  SharedPreferences.setMockInitialValues(prefs);
  await initializeDateFormatting('fr_FR');
  await tester.runAsync(Fonts.preload);
  tester.view.physicalSize = size;
  tester.view.devicePixelRatio = 1;
  addTearDown(tester.view.resetPhysicalSize);
  final store = MailStore(await SharedPreferences.getInstance());
  await tester.pumpWidget(
    ChangeNotifierProvider.value(value: store, child: const MiraApp()),
  );
  await tester.pump(const Duration(milliseconds: 100));
  return store;
}

Future<void> teardown(WidgetTester tester, MailStore store) async {
  await tester.pumpWidget(const SizedBox());
  store.dispose();
}

void main() {
  testWidgets('démarrage : liste, premier message ouvert, compteurs', (
    tester,
  ) async {
    final s = await pumpApp(tester);
    expect(find.text('RÉCEPTION'), findsWidgets);
    expect(find.text('TON PLAN DE PROJET A ÉTÉ VALIDÉ'), findsOneWidget);
    expect(s.navCount('inbox'), 5); // le premier message ouvert est marqué lu
    expect(s.view, PaneView.read);
    await teardown(tester, s);
  });

  testWidgets('raccourcis clavier : j/k, e (archiver), ? (aide), Échap', (
    tester,
  ) async {
    final s = await pumpApp(tester);
    final first = s.open;
    await tester.sendKeyDownEvent(LogicalKeyboardKey.keyJ, character: 'j');
    await tester.sendKeyUpEvent(LogicalKeyboardKey.keyJ);
    await tester.pump();
    expect(s.open, isNot(first));
    await tester.sendKeyDownEvent(LogicalKeyboardKey.keyE, character: 'e');
    await tester.sendKeyUpEvent(LogicalKeyboardKey.keyE);
    await tester.pump();
    expect(s.toasts.any((t) => t.msg.contains('archivé')), isTrue);
    await tester.sendKeyDownEvent(LogicalKeyboardKey.slash, character: '?');
    await tester.sendKeyUpEvent(LogicalKeyboardKey.slash);
    await tester.pump();
    expect(s.showHelp, isTrue);
    await tester.sendKeyDownEvent(LogicalKeyboardKey.escape); // ferme l'aide
    await tester.sendKeyUpEvent(LogicalKeyboardKey.escape);
    await tester.pump();
    expect(s.showHelp, isFalse);
    await tester.sendKeyDownEvent(
      LogicalKeyboardKey.escape,
    ); // ferme le message
    await tester.sendKeyUpEvent(LogicalKeyboardKey.escape);
    await tester.pump();
    expect(s.open, isNull);
    await teardown(tester, s);
  });

  testWidgets('rédiger et envoyer un message', (tester) async {
    final s = await pumpApp(tester);
    await tester.tap(find.text('NOUVEAU MESSAGE').first);
    await tester.pump(const Duration(milliseconds: 100));
    expect(find.text('NOUVEAU MESSAGE'), findsWidgets);
    expect(s.c, isNotNull);

    // Destinataire (autocomplétion → puce), objet et corps
    final fields = find.byType(TextField);
    await tester.enterText(fields.at(1), 'awa@stockbot.africa');
    await tester.pump();
    await tester.testTextInput.receiveAction(TextInputAction.done);
    await tester.pump();
    await tester.pump();
    expect(s.c!.to.map((p) => p.email), ['awa@stockbot.africa']);
    await tester.enterText(
      find.widgetWithText(TextField, '').at(2).evaluate().isEmpty
          ? fields.at(3)
          : fields.at(3),
      'Démo de jeudi',
    );
    await tester.enterText(fields.at(4), 'Bonjour Awa');
    await tester.pump(const Duration(milliseconds: 100));
    expect(s.c!.subject.text, 'Démo de jeudi');
    expect(s.c!.body.text, 'Bonjour Awa');

    await tester.tap(find.text('ENVOYER'));
    await tester.pump(const Duration(milliseconds: 100));
    expect(s.c, isNull);
    final sent = s.mails.where(
      (m) => m.folder == 'sent' && m.subject == 'Démo de jeudi',
    );
    expect(sent.length, 1);
    expect(sent.single.to.single.name, 'Awa Diallo');
    expect(find.text('Message envoyé'), findsOneWidget);
    await teardown(tester, s);
  });

  testWidgets('envoi sans destinataire : message d’erreur et champ conservé', (
    tester,
  ) async {
    final s = await pumpApp(tester);
    await tester.tap(find.text('NOUVEAU MESSAGE').first);
    await tester.pump(const Duration(milliseconds: 100));
    await tester.tap(find.text('ENVOYER'));
    await tester.pump(const Duration(milliseconds: 400));
    expect(find.text('Ajoute au moins un destinataire'), findsOneWidget);
    expect(s.c, isNotNull);
    await teardown(tester, s);
  });

  testWidgets('paramètres : thème clair, densité compacte', (tester) async {
    final s = await pumpApp(tester);
    await tester.tap(find.text('PARAMÈTRES').first);
    await tester.pump(const Duration(milliseconds: 100));
    expect(find.text('APPARENCE'), findsOneWidget);
    await tester.tap(find.text('CLAIR'));
    await tester.pump(const Duration(milliseconds: 100));
    expect(s.theme, 'light');
    await tester.tap(find.text('COMPACTE'));
    await tester.pump();
    expect(s.dense, isTrue);
    expect(s.prefs.getString('theme'), 'light');
    await teardown(tester, s);
  });

  testWidgets('assistant : question « urgent » → liste de messages', (
    tester,
  ) async {
    final s = await pumpApp(tester);
    await tester.tap(find.text('ASSISTANT IA').first);
    await tester.pump(const Duration(milliseconds: 100));
    expect(find.textContaining('Salut Noah !'), findsOneWidget);
    await tester.tap(find.text('QU’EST-CE QUI EST URGENT ?'));
    await tester.pump(const Duration(seconds: 2));
    await tester.pump(const Duration(seconds: 3));
    expect(s.chat.length, 3);
    expect(s.chat.last.done, isTrue);
    expect(s.chat.last.refs, isNotEmpty);
    await teardown(tester, s);
  });

  testWidgets('mobile : liste → lecteur → retour système', (tester) async {
    final s = await pumpApp(tester, size: const Size(390, 844));
    expect(
      s.view,
      PaneView.empty,
    ); // pas d'ouverture automatique en vue étroite
    await tester.tap(find.text('Ton plan de projet a été validé'));
    await tester.pump(const Duration(milliseconds: 100));
    expect(s.view, PaneView.read);
    expect(find.text('TON PLAN DE PROJET A ÉTÉ VALIDÉ'), findsOneWidget);
    await tester.binding.handlePopRoute();
    await tester.pump();
    expect(s.view, PaneView.empty);
    await teardown(tester, s);
  });

  testWidgets('mobile : tiroir de navigation', (tester) async {
    final s = await pumpApp(tester, size: const Size(390, 844));
    expect(s.navOpen, isFalse);
    await tester.tap(find.byTooltip('Ouvrir le menu'));
    await tester.pump();
    await tester.pump(const Duration(milliseconds: 400));
    expect(s.navOpen, isTrue);
    await tester.tap(find.text('Archives'));
    await tester.pump(const Duration(milliseconds: 300));
    expect(s.navOpen, isFalse);
    expect(s.folder, 'archive');
    await teardown(tester, s);
  });

  testWidgets('comptes : menu déroulant, changement et ajout', (tester) async {
    final s = await pumpApp(tester);
    await tester.tap(find.byTooltip('Changer de compte'));
    await tester.pump(const Duration(milliseconds: 200));
    expect(find.text('Ajouter un compte'), findsOneWidget);
    await tester.tap(find.text('Toutes les boîtes'));
    await tester.pump(const Duration(milliseconds: 200));
    expect(s.unified, isTrue);
    await tester.tap(find.byTooltip('Changer de compte'));
    await tester.pump(const Duration(milliseconds: 200));
    await tester.tap(find.text('contact@stockbot.africa'));
    await tester.pump(const Duration(milliseconds: 200));
    expect(s.active.id, 'a3');

    await tester.tap(find.byTooltip('Changer de compte'));
    await tester.pump(const Duration(milliseconds: 200));
    await tester.tap(find.text('Ajouter un compte'));
    await tester.pump(const Duration(milliseconds: 300));
    expect(find.text('AJOUTER UN COMPTE'), findsOneWidget);
    await tester.enterText(find.byType(TextField).last, 'pas-un-mail');
    await tester.tap(find.text('AJOUTER'));
    await tester.pump();
    expect(find.text('Entre une adresse e-mail valide.'), findsOneWidget);
    await tester.enterText(find.byType(TextField).last, 'nouveau@mira.app');
    await tester.tap(find.text('AJOUTER'));
    await tester.pump(const Duration(milliseconds: 300));
    expect(s.active.email, 'nouveau@mira.app');
    await teardown(tester, s);
    me = Person(defaultAccounts[0].name, defaultAccounts[0].email);
  });

  testWidgets('paramètres : onglets et couleur principale', (tester) async {
    final s = await pumpApp(tester);
    await tester.tap(find.text('PARAMÈTRES').first);
    await tester.pump(const Duration(milliseconds: 100));
    expect(
      find.text('COULEUR PRINCIPALE'),
      findsNothing,
    ); // titre en casse normale
    expect(find.text('Couleur principale'), findsOneWidget);
    expect(find.text('Signature'), findsNothing); // autre onglet
    await tester.tap(find.byTooltip('Bleu'));
    await tester.pump(const Duration(milliseconds: 100));
    expect(s.accentHex, '3b82f6');
    expect(s.prefs.getString('accent'), '3b82f6');
    await tester.tap(find.byTooltip('Orange'));
    await tester.pump(const Duration(milliseconds: 100));
    expect(s.accentHex, isNull);
    await tester.tap(find.text('MESSAGERIE'));
    await tester.pump(const Duration(milliseconds: 100));
    expect(find.text('Signature'), findsOneWidget);
    await teardown(tester, s);
  });

  testWidgets(
    'destinataires : plusieurs adresses, suggestions larges, focus conservé',
    (tester) async {
      final s = await pumpApp(tester);
      await tester.tap(find.text('NOUVEAU MESSAGE').first);
      await tester.pump(const Duration(milliseconds: 100));
      final to = find.byType(TextField).at(1);
      await tester.enterText(to, 'ko');
      await tester.pump(const Duration(milliseconds: 100));
      // la liste de suggestions affiche l'adresse complète, sur une seule ligne
      final sug = find.text('kofi@stockbot.africa');
      expect(sug, findsOneWidget);
      expect(tester.getSize(sug).height, lessThan(24));
      await tester.testTextInput.receiveAction(TextInputAction.done);
      await tester.pump(const Duration(milliseconds: 200));
      await tester.pump(const Duration(milliseconds: 200));
      expect(s.c!.to.map((p) => p.email), ['kofi@stockbot.africa']);
      expect(
        s.c!.toFocus.hasFocus,
        isTrue,
      ); // on peut taper la seconde adresse tout de suite
      await tester.enterText(find.byType(TextField).at(1), 'awa');
      await tester.pump(const Duration(milliseconds: 100));
      await tester.testTextInput.receiveAction(TextInputAction.done);
      await tester.pump(const Duration(milliseconds: 200));
      await tester.pump(const Duration(milliseconds: 200));
      expect(s.c!.to.map((p) => p.email), [
        'kofi@stockbot.africa',
        'awa@stockbot.africa',
      ]);
      s.discard(); // arrête le minuteur d'enregistrement automatique
      await teardown(tester, s);
    },
  );

  testWidgets('lire une pièce jointe : PDF puis navigation entre images', (
    tester,
  ) async {
    final s = await pumpApp(tester);
    // PDF du premier message (déjà ouvert)
    await tester.tap(find.text('plan-validation.pdf'));
    await tester.pump(const Duration(milliseconds: 100));
    await tester.pump(const Duration(milliseconds: 300));
    expect(find.text('VALIDATION DU PLAN DE PROJET'), findsOneWidget);
    expect(find.text('Télécharger'.toUpperCase()), findsOneWidget);
    await tester.tap(find.byTooltip('Zoom avant (+)'));
    await tester.pump();
    expect(find.text('120 %'), findsOneWidget);
    await tester.tap(find.byTooltip('Fermer (Échap)'));
    await tester.pump(const Duration(milliseconds: 400));
    await tester.pump(const Duration(milliseconds: 400));
    expect(find.text('VALIDATION DU PLAN DE PROJET'), findsNothing);

    // message avec trois images
    final m = s.mails.firstWhere(
      (m) => m.subject == 'Idées pour la landing page',
    );
    s.openMail(m.id);
    await tester.pump(const Duration(milliseconds: 400));
    await tester.tap(find.text('landing-A.png'));
    await tester.pump(const Duration(milliseconds: 400));
    expect(find.textContaining('1 / 3'), findsOneWidget);
    await tester.tap(find.byTooltip('Suivant (→)'));
    await tester.pump(const Duration(milliseconds: 400));
    expect(find.textContaining('2 / 3'), findsOneWidget);
    await tester.tap(find.byTooltip('Fermer (Échap)'));
    await tester.pump(const Duration(milliseconds: 400));
    await tester.pump(const Duration(milliseconds: 400));
    await teardown(tester, s);
  });

  testWidgets('langue : passage en anglais, interface et réglage mémorisé', (
    tester,
  ) async {
    final s = await pumpApp(tester);
    await initializeDateFormatting('en_US');
    expect(
      find.text('Ton plan de projet a été validé'),
      findsWidgets,
    ); // contenu fictif : reste en français
    s.setLang(AppLang.en);
    await tester.pump(const Duration(milliseconds: 400));
    expect(find.text('INBOX'), findsWidgets);
    expect(find.text('Search my messages…'), findsOneWidget);
    expect(find.text('Starred'), findsOneWidget);
    expect(find.text('REPLY'), findsOneWidget);
    expect(s.prefs.getString('lang'), 'en');
    await tester.tap(find.text('SETTINGS').first);
    await tester.pump(const Duration(milliseconds: 400));
    expect(find.text('APPEARANCE'), findsOneWidget);
    expect(find.text('App language'), findsOneWidget);
    await tester.tap(find.text('FRANÇAIS'));
    await tester.pump(const Duration(milliseconds: 400));
    expect(appLang, AppLang.fr);
    expect(find.text('APPARENCE'), findsOneWidget);
    await teardown(tester, s);
  });
}

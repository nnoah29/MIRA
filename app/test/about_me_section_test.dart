import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mira/engine/ai_admin.dart';
import 'package:mira/state/about_me_state.dart';
import 'package:mira/theme/app_theme.dart';
import 'package:mira/ui/widgets.dart';
import 'package:mira/ui/about_me_section.dart';

import 'fake_ai_engine.dart';

Future<(FakeAiEngine, AboutMeState)> open(
  WidgetTester tester, {
  String text = '',
}) async {
  final engine = FakeAiEngine()..about.text = text;
  final state = AboutMeState(AiAdmin(engine));
  await state.load();
  await tester.pumpWidget(
    MaterialApp(
      theme: buildTheme(false),
      home: Scaffold(
        body: SingleChildScrollView(child: AboutMeSection(state: state)),
      ),
    ),
  );
  await tester.pumpAndSettle();
  return (engine, state);
}

Finder key(String k) => find.byKey(ValueKey(k));

void main() {
  testWidgets('le champ montre le texte enregistré et dit où il part', (
    tester,
  ) async {
    await open(tester, text: 'TEXTE-ENREGISTRE');
    expect(find.text('TEXTE-ENREGISTRE'), findsOneWidget);
    expect(key('about-me-note'), findsOneWidget);
    expect(find.textContaining('envoyé au service d’IA'), findsOneWidget);
  });

  testWidgets('enregistrer n’est possible qu’après une modification', (
    tester,
  ) async {
    final (engine, _) = await open(tester);
    expect(tester.widget<Btn>(key('about-me-save')).onTap, isNull);
    await tester.enterText(key('about-me-field'), 'Je suis médecin.');
    await tester.pump();
    await tester.tap(key('about-me-save'));
    await tester.pumpAndSettle();
    expect(engine.about.text, 'Je suis médecin.');
    expect(key('about-me-saved'), findsOneWidget);
  });

  testWidgets('le compteur passe en alerte au-delà de la limite', (
    tester,
  ) async {
    await open(tester);
    await tester.enterText(key('about-me-field'), 'é' * 1001);
    await tester.pump();
    expect(find.text('1001 / 1000'), findsOneWidget);
    expect(tester.widget<Btn>(key('about-me-save')).onTap, isNull);
  });
}

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mira/data/models.dart';
import 'package:mira/state/image_policy.dart';
import 'package:mira/state/store.dart';
import 'package:mira/theme/app_theme.dart';
import 'package:mira/ui/html_body.dart';
import 'package:mira/ui/image_rules_section.dart';
import 'package:provider/provider.dart';
import 'package:shared_preferences/shared_preferences.dart';

Future<MailStore> _store([Map<String, Object> initial = const {}]) async {
  SharedPreferences.setMockInitialValues(initial);
  return MailStore(await SharedPreferences.getInstance());
}

Widget _app(MailStore s, Widget child) => ChangeNotifierProvider.value(
  value: s,
  child: MaterialApp(
    theme: buildTheme(true),
    home: Scaffold(body: SingleChildScrollView(child: child)),
  ),
);

void main() {
  test('adresse exacte > domaine le plus précis > réglage général', () {
    final p = const ImagePolicy(fallback: ImageAction.never)
        .withRule(const ImageRule('exemple.com', ImageAction.always))
        .withRule(const ImageRule('promo.exemple.com', ImageAction.never))
        .withRule(const ImageRule('ami@promo.exemple.com', ImageAction.always));
    expect(p.decide('x@exemple.com'), ImageAction.always);
    expect(p.decide('x@news.exemple.com'), ImageAction.always);
    expect(p.decide('x@promo.exemple.com'), ImageAction.never);
    expect(p.decide('AMI@promo.exemple.com'), ImageAction.always);
    expect(p.decide('x@autre.org'), ImageAction.never);
    expect(p.decide('x@notexemple.com'), ImageAction.never);
  });

  test('saisies invalides refusées, valides normalisées', () {
    expect(normalizeMatch('  @Exemple.COM '), 'exemple.com');
    expect(normalizeMatch('Bob@Exemple.com'), 'bob@exemple.com');
    expect(normalizeMatch('pas un domaine'), isNull);
    expect(normalizeMatch('localhost'), isNull);
  });

  test(
    'politique enregistrée puis relue ; données illisibles → demander',
    () async {
      final s = await _store();
      s.setImageDefault(ImageAction.never);
      expect(s.addImageRule('promo.exemple.com', ImageAction.always), isTrue);
      expect(s.addImageRule('n’importe quoi', ImageAction.always), isFalse);
      final again = ImagePolicy.decode(s.prefs.getString('imgPolicy'));
      expect(again.fallback, ImageAction.never);
      expect(again.decide('a@promo.exemple.com'), ImageAction.always);
      expect(ImagePolicy.decode('{{{').fallback, ImageAction.ask);
    },
  );

  testWidgets('Paramètres : choisir, ajouter et supprimer une règle', (
    tester,
  ) async {
    final s = await _store();
    await tester.pumpWidget(_app(s, const ImageRulesSection()));
    await tester.tap(find.byKey(const ValueKey('default-always')));
    await tester.pump();
    expect(s.imagePolicy.fallback, ImageAction.always);
    await tester.enterText(
      find.byKey(const ValueKey('rule-input')),
      'pub.example.org',
    );
    await tester.tap(
      find.text('AJOUTER').evaluate().isEmpty
          ? find.text('Ajouter')
          : find.text('AJOUTER'),
    );
    await tester.pump();
    expect(s.imagePolicy.decide('x@pub.example.org'), ImageAction.never);
    expect(find.text('pub.example.org'), findsOneWidget);
    await tester.tap(find.byKey(const ValueKey('rule-pub.example.org-always')));
    await tester.pump();
    expect(s.imagePolicy.decide('x@pub.example.org'), ImageAction.always);
  });

  testWidgets(
    'le lecteur applique la règle : bandeau seulement en « demander »',
    (tester) async {
      Msg msg() =>
          Msg(
              from: const Person('B', 'b@pub.example.org'),
              date: 0,
              body: const [],
            )
            ..html = '<p>Salut</p><img src="https://x.test/a.png">'
            ..blocked = ['https://x.test/a.png'];
      final s = await _store();
      await tester.pumpWidget(
        _app(s, MailHtml(msg: msg(), base: const TextStyle())),
      );
      await tester.pump();
      expect(find.text('Afficher'), findsOneWidget);
      s.addImageRule('pub.example.org', ImageAction.never);
      await tester.pump();
      expect(find.text('Afficher'), findsNothing);
      expect(find.byType(Image), findsNothing);
    },
  );
}

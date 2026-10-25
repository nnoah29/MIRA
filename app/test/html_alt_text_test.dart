import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mira/data/models.dart';
import 'package:mira/state/store.dart';
import 'package:mira/theme/app_theme.dart';
import 'package:mira/ui/html_body.dart';
import 'package:provider/provider.dart';
import 'package:shared_preferences/shared_preferences.dart';

const _page = '''
<p><a href="https://x.org"><img alt="Ajouter à Google Agenda" src="https://cdn.example.org/g.png" width="200" height="44"></a></p>
<p><img alt="Logo bloqué" src="https://cdn.example.org/l.png" width="120"></p>
<p><img alt="" src="https://t.example.org/p.gif" width="1" height="1"></p>
<p><img alt="espion" src="https://t.example.org/q.gif" width="1" height="1"></p>
<p><img src="https://t.example.org/r.gif"></p>
''';

Future<void> _show(WidgetTester tester, {required bool blocked}) async {
  SharedPreferences.setMockInitialValues({});
  final store = MailStore(await SharedPreferences.getInstance());
  final msg = Msg(from: const Person('L', 'l@x.org'), date: 0, body: const [])
    ..html = _page
    ..blocked = blocked ? const ['https://cdn.example.org/g.png'] : const []
    ..inline = {};
  await tester.pumpWidget(
    ChangeNotifierProvider.value(
      value: store,
      child: MaterialApp(
        theme: buildTheme(false),
        home: Scaffold(
          body: SingleChildScrollView(
            child: MailHtml(msg: msg, base: const TextStyle(fontSize: 16)),
          ),
        ),
      ),
    ),
  );
  await tester.pump();
  await tester.runAsync(
    () => Future<void>.delayed(const Duration(milliseconds: 300)),
  );
  await tester.pump(const Duration(milliseconds: 700));
}

void main() {
  testWidgets('image distante bloquée : son texte alternatif la remplace', (
    tester,
  ) async {
    await _show(tester, blocked: true);
    expect(
      find.textContaining('Ajouter à Google Agenda', findRichText: true),
      findsOneWidget,
    );
    expect(
      find.textContaining('Logo bloqué', findRichText: true),
      findsOneWidget,
    );
    expect(find.byIcon(Icons.broken_image_outlined), findsNothing);
  });

  testWidgets(
    'image qui ne charge pas : texte alternatif, pas d’icône cassée',
    (tester) async {
      await _show(tester, blocked: false);
      expect(
        find.textContaining('Ajouter à Google Agenda', findRichText: true),
        findsOneWidget,
      );
      expect(find.byIcon(Icons.broken_image_outlined), findsNothing);
    },
  );

  testWidgets('un pixel espion n’affiche rien, même avec un texte alternatif', (
    tester,
  ) async {
    await _show(tester, blocked: true);
    expect(find.textContaining('espion', findRichText: true), findsNothing);
  });
}

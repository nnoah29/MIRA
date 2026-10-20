import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mira/data/models.dart';
import 'package:mira/engine/mail_layout.dart';
import 'package:mira/state/store.dart';
import 'package:mira/theme/app_theme.dart';
import 'package:mira/ui/html_body.dart';
import 'package:provider/provider.dart';
import 'package:shared_preferences/shared_preferences.dart';

const _alert =
    '<table width="100%" border="0" cellspacing="0" cellpadding="0">'
    '<tr><td align="center">'
    '<table border="0" cellspacing="0" cellpadding="0" '
    'style="padding-bottom: 20px; max-width: 516px; min-width: 220px;">'
    '<tr><td width="8" style="width: 8px;"></td><td>'
    '<div style="border: thin solid #dadce0; padding: 40px 20px;" align="center">'
    '<div style="color: rgba(0,0,0,0.87); line-height: 32px; text-align: center; '
    'word-break: break-word;"><div style="font-size: 24px;">'
    'Vous avez autorisé une application à accéder à vos données</div></div>'
    '</td><td width="8" style="width: 8px;"></td></tr></table>'
    '</td></tr></table>';

/// Rend `html` dans une fenêtre de 900 px et rend la boîte du texte cherché.
Future<Rect> _boxOf(WidgetTester tester, String html, String needle) async {
  SharedPreferences.setMockInitialValues({});
  final store = MailStore(await SharedPreferences.getInstance());
  final msg = Msg(from: const Person('G', 'g@x.org'), date: 0, body: const [])
    ..html = html
    ..blocked = const []
    ..inline = {};
  await tester.binding.setSurfaceSize(const Size(900, 1400));
  addTearDown(() => tester.binding.setSurfaceSize(null));
  await tester.pumpWidget(
    ChangeNotifierProvider.value(
      value: store,
      child: MaterialApp(
        theme: buildTheme(true),
        home: Scaffold(
          body: SingleChildScrollView(
            child: MailHtml(msg: msg, base: const TextStyle(fontSize: 16)),
          ),
        ),
      ),
    ),
  );
  await tester.pump();
  await tester.pump(const Duration(milliseconds: 600));
  return tester.getRect(find.textContaining(needle, findRichText: true).first);
}

void main() {
  testWidgets('un tableau sans largeur garde une colonne de texte lisible', (
    tester,
  ) async {
    final box = await _boxOf(
      tester,
      prepareMailHtml(_alert),
      'Vous avez autorisé',
    );
    expect(box.width, greaterThan(300));
    expect(box.height, lessThan(200));
  });
}

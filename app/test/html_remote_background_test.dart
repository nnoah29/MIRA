import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mira/data/models.dart';
import 'package:mira/state/store.dart';
import 'package:mira/theme/app_theme.dart';
import 'package:mira/ui/html_body.dart';
import 'package:mira/ui/mail_widget_factory.dart';
import 'package:provider/provider.dart';
import 'package:shared_preferences/shared_preferences.dart';

const _tracker =
    '<div style="background-image:url(https://pixel.test/u?id=1);width:50px;height:50px">x</div>';

Msg _blockedMsg() =>
    Msg(from: const Person('Bob', 'bob@x.org'), date: 0, body: const [])
      ..html = _tracker
      ..blocked = ['https://pixel.test/u']
      ..inline = {};

Future<void> _show(WidgetTester tester, Msg m) async {
  SharedPreferences.setMockInitialValues({'htmlMode': 'widgets'});
  final store = MailStore(await SharedPreferences.getInstance());
  await tester.pumpWidget(
    ChangeNotifierProvider.value(
      value: store,
      child: MaterialApp(
        theme: buildTheme(true),
        home: Scaffold(
          body: SingleChildScrollView(
            child: MailHtml(msg: m, base: const TextStyle(fontSize: 16)),
          ),
        ),
      ),
    ),
  );
  await tester.pump();
  await tester.runAsync(
    () => Future<void>.delayed(const Duration(milliseconds: 300)),
  );
  await tester.pump();
  await tester.pump(const Duration(milliseconds: 300));
}

bool _loadsRemoteBackground(WidgetTester tester) =>
    tester.widgetList<DecoratedBox>(find.byType(DecoratedBox)).any((box) {
      final decoration = box.decoration;
      return decoration is BoxDecoration &&
          decoration.image?.image is NetworkImage;
    });

void main() {
  testWidgets('arrière-plan CSS distant jamais chargé tant que bloqué', (
    tester,
  ) async {
    await _show(tester, _blockedMsg());
    expect(_loadsRemoteBackground(tester), isFalse);
  });

  test('la fabrique ne donne une image réseau que si autorisée', () {
    const url = 'https://pixel.test/u?id=1';
    expect(MailWidgetFactory().imageProviderFromNetwork(url), isNull);
    expect(
      MailWidgetFactory(remoteImages: true).imageProviderFromNetwork(url),
      isA<NetworkImage>(),
    );
  });
}

import 'dart:typed_data';
import 'dart:io';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mira/data/models.dart';
import 'package:mira/state/store.dart';
import 'package:mira/theme/app_theme.dart';
import 'package:mira/ui/download_all_button.dart';
import 'package:provider/provider.dart';
import 'package:shared_preferences/shared_preferences.dart';

Widget host(MailStore store, Widget child) => ChangeNotifierProvider.value(
  value: store,
  child: MaterialApp(
    theme: buildTheme(false),
    home: Scaffold(body: child),
  ),
);

Future<MailStore> newStore() async {
  SharedPreferences.setMockInitialValues({});
  return MailStore(await SharedPreferences.getInstance());
}

void main() {
  final list = [
    Attachment('a.txt', 'TXT', '1', bytes: Uint8List.fromList([1])),
    Attachment('b.txt', 'TXT', '1', bytes: Uint8List.fromList([2])),
  ];

  testWidgets('le dossier choisi reçoit toutes les pièces', (tester) async {
    final dir = Directory.systemTemp.createTempSync('mira-dl');
    addTearDown(() => dir.deleteSync(recursive: true));
    final store = await newStore();
    await tester.pumpWidget(
      host(store, DownloadAllButton(list, pickDirectory: () async => dir.path)),
    );
    await tester.tap(find.text('Tout télécharger'));
    for (var i = 0; i < 20 && dir.listSync().length < 2; i++) {
      await tester.runAsync(
        () => Future.delayed(const Duration(milliseconds: 50)),
      );
      await tester.pump();
    }
    expect(dir.listSync().map((e) => e.path.split('/').last).toSet(), {
      'a.txt',
      'b.txt',
    });
  });

  testWidgets('annuler le choix du dossier n écrit rien', (tester) async {
    var asked = 0;
    final store = await newStore();
    await tester.pumpWidget(
      host(
        store,
        DownloadAllButton(
          list,
          pickDirectory: () async {
            asked++;
            return null;
          },
        ),
      ),
    );
    await tester.tap(find.text('Tout télécharger'));
    await tester.pump();
    expect(asked, 1);
    expect(store.toasts, isEmpty);
  });
}

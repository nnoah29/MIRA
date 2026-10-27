import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mira/data/rule_import.dart';
import 'package:mira/i18n/i18n.dart';
import 'package:mira/theme/app_theme.dart';
import 'package:mira/ui/rules_import_dialog.dart';

/// Ouvre la boîte d'options ; le résultat est lu dans [result] à sa fermeture.
Future<void> _open(WidgetTester t, List<bool?> result) async {
  await t.pumpWidget(
    MaterialApp(
      theme: buildTheme(false),
      home: Builder(
        builder: (context) => TextButton(
          onPressed: () async =>
              result.add(await showRulesImportDialog(context)),
          child: const Text('ouvrir'),
        ),
      ),
    ),
  );
  await t.tap(find.text('ouvrir'));
  await t.pumpAndSettle();
}

RuleImportReport _report() => RuleImportReport([
  const RuleImportEntry(1, 'Lettres', 'imported'),
  const RuleImportEntry(2, 'Cassée', 'refused', reason: 'unknown_label'),
  const RuleImportEntry(
    3,
    'Doublon',
    'refused',
    reason: 'invalid_rule',
    code: 'invalid_input.rule_duplicate_in_file',
  ),
]);

void main() {
  tearDown(() => appLang = AppLang.fr);

  testWidgets('la case de remplacement est décochée par défaut', (t) async {
    final result = <bool?>[];
    await _open(t, result);
    final box = t.widget<Checkbox>(
      find.byKey(const ValueKey('import-replace')),
    );
    expect(box.value, false);
    await t.tap(find.byKey(const ValueKey('import-confirm')));
    await t.pumpAndSettle();
    expect(result, [false]);
  });

  testWidgets('cocher la case est transmis à l’import', (t) async {
    final result = <bool?>[];
    await _open(t, result);
    await t.tap(find.byKey(const ValueKey('import-replace')));
    await t.pumpAndSettle();
    await t.tap(find.byKey(const ValueKey('import-confirm')));
    await t.pumpAndSettle();
    expect(result, [true]);
  });

  testWidgets('annuler ne renvoie rien', (t) async {
    final result = <bool?>[];
    await _open(t, result);
    await t.tap(find.byKey(const ValueKey('import-cancel')));
    await t.pumpAndSettle();
    expect(result, [null]);
  });

  testWidgets('le rapport résume les sorts et détaille chaque refus', (
    t,
  ) async {
    await t.pumpWidget(
      MaterialApp(
        theme: buildTheme(false),
        home: Scaffold(body: RulesImportReportView(report: _report())),
      ),
    );
    expect(find.textContaining('1 importée'), findsOneWidget);
    expect(find.textContaining('2 refusées'), findsOneWidget);
    expect(find.textContaining('Cassée'), findsOneWidget);
    expect(find.textContaining('Libellé inconnu'), findsOneWidget);
    expect(find.textContaining('plus haut dans le fichier'), findsOneWidget);
    expect(find.textContaining('Lettres'), findsNothing);
  });
}

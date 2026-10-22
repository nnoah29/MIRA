import 'package:flutter_test/flutter_test.dart';
import 'package:mira/engine/local_label_admin.dart';
import 'package:mira/state/local_label_migration.dart';
import 'package:mira/state/local_labels.dart';
import 'package:shared_preferences/shared_preferences.dart';

/// Administration factice : garde les reprises envoyées, ou échoue.
class FakeAdmin implements LocalLabelAdmin {
  final imports = <List<LocalLabelImport>>[];
  bool failing = false;

  @override
  Future<ImportReport> importLabels(List<LocalLabelImport> labels) async {
    if (failing) throw Exception('moteur indisponible');
    imports.add(labels);
    return const ImportReport(1, 1, 0);
  }

  @override
  dynamic noSuchMethod(Invocation invocation) => super.noSuchMethod(invocation);
}

/// Préférences de l'app avec un libellé local « À payer » posé sur deux conversations.
Future<SharedPreferences> prefsWithLabels() async {
  final labels = LocalLabels();
  final key = labels.create('À payer')!;
  labels.set(['t1', 't2'], key, true);
  SharedPreferences.setMockInitialValues({'localLabels': labels.encode()});
  return SharedPreferences.getInstance();
}

void main() {
  test(
    'la reprise envoie libellés et poses, puis efface la copie locale',
    () async {
      final prefs = await prefsWithLabels();
      final admin = FakeAdmin();
      final done = await migrateLocalLabels(prefs, admin);
      expect(done, true);
      final sent = admin.imports.single.single;
      expect(sent.name, 'À payer');
      expect(sent.conversations.toSet(), {'t1', 't2'});
      expect(prefs.getString('localLabels'), isNull);
      expect(prefs.getBool('localLabelsMigrated'), true);
    },
  );

  test(
    'un échec garde la copie locale et laisse réessayer plus tard',
    () async {
      final prefs = await prefsWithLabels();
      final admin = FakeAdmin()..failing = true;
      final done = await migrateLocalLabels(prefs, admin);
      expect(done, false);
      expect(prefs.getString('localLabels'), isNotNull);
      expect(prefs.getBool('localLabelsMigrated'), isNot(true));
      admin.failing = false;
      expect(await migrateLocalLabels(prefs, admin), true);
    },
  );

  test('déjà reprise ou sans libellés : aucun appel', () async {
    SharedPreferences.setMockInitialValues({});
    final prefs = await SharedPreferences.getInstance();
    final admin = FakeAdmin();
    expect(await migrateLocalLabels(prefs, admin), true);
    await prefs.setString('localLabels', '{"defs":{}}');
    await prefs.setBool('localLabelsMigrated', true);
    expect(await migrateLocalLabels(prefs, admin), true);
    expect(admin.imports, isEmpty);
  });
}


import 'package:flutter_test/flutter_test.dart';
import 'package:mira/engine/engine_mail_source.dart';
import 'package:mira/state/local_labels.dart';
import 'package:mira/state/store.dart';
import 'package:shared_preferences/shared_preferences.dart';

import 'engine_source_test.dart' show FakeEngine;

/// Libellé « À payer » du moteur et sa pose sur la conversation du fil t1.
void seedEngineLabel(FakeEngine fake) {
  fake.localLabels.add({'id': 'l1', 'name': 'À payer', 'color': 4293874731});
  fake.localPoses.add({
    'thread': 't1',
    'labels': ['l1'],
  });
}

/// Ouvre le magasin sur le faux moteur, une fois ses libellés locaux lus.
Future<(MailStore, SharedPreferences)> open(
  FakeEngine fake, [
  Map<String, Object> prefs = const {},
]) async {
  SharedPreferences.setMockInitialValues(prefs);
  final p = await SharedPreferences.getInstance();
  final store = MailStore(p, source: await EngineMailSource.load(fake));
  await store.localLabelsReady;
  return (store, p);
}

Future<void> settle() => Future<void>.delayed(const Duration(milliseconds: 30));

void main() {
  test(
    'libellés et poses du moteur sont posés sur les conversations',
    () async {
      final fake = FakeEngine();
      seedEngineLabel(fake);
      final (store, _) = await open(fake);
      expect(store.localLabels.defs['${localLabelPrefix}l1']!.n, 'À payer');
      final conv = store.mails.firstWhere((m) => m.id == 'acc1/inbox/t1');
      expect(conv.labels, contains('${localLabelPrefix}l1'));
      store.dispose();
    },
  );

  test('poser un libellé local envoie la pose au moteur, par fil', () async {
    final fake = FakeEngine();
    seedEngineLabel(fake);
    fake.localPoses.clear();
    final (store, _) = await open(fake);
    store.sel.add('acc1/inbox/t1');
    store.toggleLabel('${localLabelPrefix}l1');
    await settle();
    final set = fake.labelWrites.firstWhere(
      (w) => w['op'] == 'set_local_label',
    );
    expect(set['label'], 'l1');
    expect(set['threads'], ['t1']);
    expect(set['on'], true);
    store.dispose();
  });

  test('créer, renommer et supprimer passent par le moteur', () async {
    final fake = FakeEngine();
    seedEngineLabel(fake);
    final (store, _) = await open(fake);
    expect(store.addLocalLabel('Urgent'), true);
    expect(store.addLocalLabel('à PAYER'), false);
    expect(store.renameLocalLabel('${localLabelPrefix}l1', 'Dû'), true);
    store.deleteLocalLabel('${localLabelPrefix}l1');
    await settle();
    final ops = [for (final w in fake.labelWrites) w['op']];
    expect(ops, [
      'create_local_label',
      'rename_local_label',
      'delete_local_label',
    ]);
    expect(fake.labelWrites.first['name'], 'Urgent');
    store.dispose();
  });

  test(
    'un changement signalé par le moteur relit les libellés locaux',
    () async {
      final fake = FakeEngine();
      final (store, _) = await open(fake);
      expect(store.localLabels.defs, isEmpty);
      seedEngineLabel(fake);
      fake.feed.add({
        'kind': {'type': 'local_labels_changed'},
      });
      await settle();
      expect(store.localLabels.defs.keys, ['${localLabelPrefix}l1']);
      store.dispose();
    },
  );

  test('les libellés des préférences sont repris au démarrage', () async {
    final labels = LocalLabels();
    final key = labels.create('Banque')!;
    labels.set(['acc1/inbox/t1'], key, true);
    final fake = FakeEngine();
    final (store, prefs) = await open(fake, {'localLabels': labels.encode()});
    final sent = fake.labelWrites.single;
    expect(sent['op'], 'import_local_labels');
    final first = (sent['labels'] as List).single as Map;
    expect(first['name'], 'Banque');
    expect(first['conversations'], ['t1']);
    expect(prefs.getString('localLabels'), isNull);
    store.dispose();
  });

  test(
    'reprise impossible : copie gardée, libellés locaux toujours utilisables',
    () async {
      final labels = LocalLabels();
      labels.create('Banque');
      final fake = FakeEngine()..importFails = true;
      final (store, prefs) = await open(fake, {'localLabels': labels.encode()});
      expect(prefs.getString('localLabels'), isNotNull);
      expect(store.localLabels.defs.values.single.n, 'Banque');
      store.sel.add('acc1/inbox/t1');
      store.toggleLabel(store.localLabels.defs.keys.single);
      await settle();
      expect(fake.labelWrites, isEmpty);
      store.dispose();
    },
  );
}

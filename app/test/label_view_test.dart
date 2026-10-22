import 'package:flutter_test/flutter_test.dart';
import 'package:mira/engine/label_settings_admin.dart' show CounterMode;
import 'package:mira/state/local_labels.dart';

import 'engine_source_test.dart' show FakeEngine;
import 'local_labels_engine_test.dart' show open, seedEngineLabel;

Future<void> settle() => Future<void>.delayed(const Duration(milliseconds: 30));

void main() {
  test(
    'un message masqué de la Réception reste dans la vue de son libellé',
    () async {
      final fake = FakeEngine();
      seedEngineLabel(fake);
      fake.localPoses.add({
        'thread': 'th1',
        'labels': ['l1'],
      });
      final (store, _) = await open(fake);
      expect(store.mails.any((m) => m.id == 'acc1/inbox/th1'), false);
      store.selectLabel('${localLabelPrefix}l1');
      await settle();
      expect(fake.labelViews.single, {'type': 'local_label', 'label': 'l1'});
      expect(store.visible().map((m) => m.id), contains('acc1/inbox/th1'));
      store.dispose();
    },
  );

  test('un réglage modifié relit aussitôt la Réception', () async {
    final fake = FakeEngine()..settingsOn = true;
    seedEngineLabel(fake);
    final (store, _) = await open(fake);
    await store.refreshLabelSettings();
    expect(store.mails.any((m) => m.id == 'acc1/inbox/m3'), true);
    fake.hideInbox = true;
    await store.refreshLabelSettings();
    expect(store.mails.any((m) => m.id == 'acc1/inbox/m3'), false);
    store.dispose();
  });

  test(
    'le compteur d\u2019un libellé masqué de la Réception compte ses messages',
    () async {
      final fake = FakeEngine()
        ..settingsOn = true
        ..hideInbox = true;
      seedEngineLabel(fake);
      fake.localPoses.add({
        'thread': 'th1',
        'labels': ['l1'],
      });
      final (store, _) = await open(fake);
      await store.refreshLabelSettings();
      await settle();
      store.setLabelCounter(CounterMode.total);
      expect(store.labelCount('${localLabelPrefix}l1'), 2);
      store.dispose();
    },
  );

  test(
    'créer un libellé local relit aussitôt les réglages des libellés',
    () async {
      final fake = FakeEngine()..settingsOn = true;
      final (store, _) = await open(fake);
      final before = fake.calls
          .where((c) => c == 'label_settings.list_labels')
          .length;
      expect(store.addLocalLabel('Urgent'), true);
      await settle();
      final after = fake.calls
          .where((c) => c == 'label_settings.list_labels')
          .length;
      expect(after, greaterThan(before));
      store.dispose();
    },
  );

  test(
    'le compteur choisi vaut pour tous les libellés et est mémorisé',
    () async {
      final fake = FakeEngine();
      seedEngineLabel(fake);
      final (store, prefs) = await open(fake);
      final key = '${localLabelPrefix}l1';
      store.setLabelCounter(CounterMode.total);
      expect(store.labelCount(key), 1);
      store.setLabelCounter(CounterMode.none);
      expect(store.labelCount(key), 0);
      expect(prefs.getString('labelCounter'), 'none');
      store.dispose();
    },
  );
}

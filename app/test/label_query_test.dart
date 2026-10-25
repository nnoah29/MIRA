import 'package:flutter_test/flutter_test.dart';
import 'package:mira/data/models.dart';
import 'package:mira/state/label_query.dart';
import 'package:mira/state/local_labels.dart';

import 'engine_source_test.dart' show FakeEngine;
import 'local_labels_engine_test.dart' show open, seedEngineLabel;

const defs = {
  'a': LabelDef('Veille tech', 0, local: true),
  'b': LabelDef('Sécurité', 0, local: true),
  'c': LabelDef('Social', 0, local: true),
  'd': LabelDef('Systeme', 0, local: true),
};

Future<void> settle() => Future<void>.delayed(const Duration(milliseconds: 30));

void main() {
  test('un nom complet, avec espace, casse et accents ignorés', () {
    expect(parseLabelQuery('@:Veille tech', defs), (key: 'a', rest: ''));
    expect(parseLabelQuery('@:securite', defs), (key: 'b', rest: ''));
  });

  test('le texte après le nom reste à chercher', () {
    expect(parseLabelQuery('@:social reddit', defs), (
      key: 'c',
      rest: 'reddit',
    ));
  });

  test('un préfixe unique suffit, un préfixe ambigu ne choisit rien', () {
    expect(parseLabelQuery('@:vei', defs), (key: 'a', rest: ''));
    expect(parseLabelQuery('@:S', defs), (key: null, rest: '@:S'));
  });

  test('sans @: ou avec un nom inconnu, rien ne change', () {
    expect(parseLabelQuery('social', defs), (key: null, rest: 'social'));
    expect(parseLabelQuery('@:zzz', defs), (key: null, rest: '@:zzz'));
  });

  test(
    '@:nom affiche le libellé comme une sélection, l’effacer le retire',
    () async {
      final fake = FakeEngine();
      seedEngineLabel(fake);
      final (store, _) = await open(fake);
      final key = '${localLabelPrefix}l1';
      store.setQuery('@:à payer');
      await settle();
      expect(store.label, key);
      expect(store.q, '');
      expect(store.visible().map((m) => m.id), contains('acc1/inbox/t1'));
      store.setQuery('');
      expect(store.label, null);
      store.setQuery('@:à payer');
      store.clearQuery();
      expect(store.label, null);
      store.dispose();
    },
  );

  test('un libellé choisi dans la barre latérale survit à la saisie', () async {
    final fake = FakeEngine();
    seedEngineLabel(fake);
    final (store, _) = await open(fake);
    final key = '${localLabelPrefix}l1';
    store.selectLabel(key);
    store.setQuery('abc');
    store.setQuery('');
    expect(store.label, key);
    await settle();
    store.dispose();
  });

  test('un libellé masqué de la barre latérale reste atteignable', () async {
    final fake = FakeEngine()..settingsOn = true;
    seedEngineLabel(fake);
    final (store, _) = await open(fake);
    await store.refreshLabelSettings();
    final key = '${localLabelPrefix}l1';
    store.setQuery('@:à payer');
    await settle();
    expect(store.label, key);
    expect(store.visible().map((m) => m.id), contains('acc1/inbox/t1'));
    store.dispose();
  });

  test(
    'marquer lu fait baisser le compteur de la Réception sans attendre',
    () async {
      final fake = FakeEngine();
      final (store, _) = await open(fake);
      final before = store.navCount('inbox');
      final conv = store.mails.firstWhere(
        (m) => m.unread && m.folder == 'inbox',
      );
      store.sel.add(conv.id);
      store.toggleRead();
      await settle();
      expect(store.navCount('inbox'), before - 1);
      store.dispose();
    },
  );

  test('marquer lu dans un libellé met à jour son compteur', () async {
    final fake = FakeEngine();
    seedEngineLabel(fake);
    final (store, _) = await open(fake);
    final key = '${localLabelPrefix}l1';
    final before = store.labelCount(key);
    store.selectLabel(key);
    await settle();
    final shown = store.visible().firstWhere((m) => m.unread);
    store.sel.add(shown.id);
    store.toggleRead();
    expect(store.labelCount(key), before - 1);
    await settle();
    store.dispose();
  });
}

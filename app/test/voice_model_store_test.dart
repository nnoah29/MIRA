import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:mira/voice/dictation_types.dart';
import 'package:mira/voice/voice_model_catalog.dart';
import 'package:mira/voice/voice_model_store.dart';

import 'voice_model_server.dart';

void main() {
  late Directory root;
  late ModelServer server;
  const fr = DictationLanguage.fr;

  setUp(() async {
    root = Directory.systemTemp.createTempSync('voice-store');
    server = ModelServer();
    await server.start();
  });

  tearDown(() async {
    await server.stop();
    root.deleteSync(recursive: true);
  });

  VoiceModelStore storeFor(VoiceModelEntry e) =>
      VoiceModelStore(root: root, catalog: [e]);

  List<String> leftovers() => root
      .listSync()
      .map((e) => e.path.split('/').last)
      .where((n) => n != 'model-fr')
      .toList();

  test('le catalogue pinne le français et l’anglais', () {
    final langs = voiceModelCatalog.map((e) => e.language).toSet();
    expect(langs, {DictationLanguage.fr, DictationLanguage.en});
    for (final e in voiceModelCatalog) {
      expect(e.sha256, hasLength(64));
      expect(e.archiveBytes, greaterThan(50 * 1000 * 1000));
      expect(e.url.scheme, 'https');
    }
  });

  test('install réussie : Installed et les quatre fichiers présents', () async {
    final store = storeFor(server.entry());
    expect(store.statusOf(fr), isA<VoiceModelNotInstalled>());
    await store.install(fr);
    expect(store.statusOf(fr), isA<VoiceModelInstalled>());
    expect(store.isInstalled(fr), isTrue);
    final dir = store.directoryOf(fr)!;
    for (final n in [
      'encoder.onnx',
      'decoder.onnx',
      'joiner.onnx',
      'tokens.txt',
    ]) {
      expect(File('${dir.path}/$n').existsSync(), isTrue, reason: n);
    }
    expect(File('${dir.path}/README.md').existsSync(), isFalse);
    expect(leftovers(), isEmpty);
  });

  test('la progression monte avant l’installation', () async {
    final store = storeFor(server.entry());
    final seen = <double>[];
    store.addListener(() {
      final s = store.statusOf(fr);
      if (s is VoiceModelDownloading) seen.add(s.progress);
    });
    await store.install(fr);
    expect(seen.any((p) => p > 0 && p < 1), isTrue);
    expect(seen, orderedEquals([...seen]..sort()));
  });

  test(
    'empreinte différente : Failed, rien d’installé, aucun résidu',
    () async {
      final good = server.entry();
      final bad = VoiceModelEntry(
        language: fr,
        url: good.url,
        archiveBytes: good.archiveBytes,
        sha256: '0' * 64,
        folder: good.folder,
        files: good.files,
      );
      final store = storeFor(bad);
      await store.install(fr);
      expect(store.statusOf(fr), isA<VoiceModelFailed>());
      expect(store.directoryOf(fr), isNull);
      expect(root.listSync(), isEmpty);
    },
  );

  test(
    'connexion coupée en route : Failed, aucun dossier utilisable',
    () async {
      server.mode = ServeMode.cutHalfway;
      final store = storeFor(server.entry());
      await store.install(fr);
      expect(store.statusOf(fr), isA<VoiceModelFailed>());
      expect(store.directoryOf(fr), isNull);
      expect(root.listSync(), isEmpty);
    },
  );

  test(
    'annulation en cours de téléchargement : NotInstalled, rien ne reste',
    () async {
      server.mode = ServeMode.hangHalfway;
      final store = storeFor(server.entry());
      final done = store.install(fr);
      await server.halfway.future;
      store.cancel(fr);
      await done;
      expect(store.statusOf(fr), isA<VoiceModelNotInstalled>());
      expect(root.listSync(), isEmpty);
    },
  );

  test('archive à laquelle il manque un fichier : Failed', () async {
    final partial = ModelServer();
    await partial.start(only: ['encoder.onnx', 'tokens.txt']);
    addTearDown(partial.stop);
    final store = storeFor(partial.entry());
    await store.install(fr);
    expect(store.statusOf(fr), isA<VoiceModelFailed>());
    expect(root.listSync(), isEmpty);
  });

  test('remove supprime le dossier', () async {
    final store = storeFor(server.entry());
    await store.install(fr);
    await store.remove(fr);
    expect(store.statusOf(fr), isA<VoiceModelNotInstalled>());
    expect(Directory('${root.path}/model-fr').existsSync(), isFalse);
  });

  test('un modèle déjà présent est reconnu, les restes d’un plantage sont nettoyés', () async {
    final first = storeFor(server.entry());
    await first.install(fr);
    Directory('${root.path}/model-fr.tmp').createSync();
    File('${root.path}/model-fr.tar.bz2.part').writeAsStringSync('x');
    final second = storeFor(server.entry());
    expect(second.isInstalled(fr), isTrue);
    await second.refresh();
    expect(second.statusOf(fr), isA<VoiceModelInstalled>());
    expect(leftovers(), isEmpty);
  });

  test('un dossier incomplet n’est pas pris pour un modèle installé', () async {
    Directory('${root.path}/model-fr').createSync();
    File('${root.path}/model-fr/encoder.onnx').writeAsStringSync('x');
    final store = storeFor(server.entry());
    expect(store.isInstalled(fr), isFalse);
  });
}

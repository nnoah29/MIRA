import 'dart:io';
import 'dart:typed_data';

import 'package:flutter_test/flutter_test.dart';
import 'package:mira/voice/dictation_types.dart';
import 'package:mira/voice/sherpa_recognizer.dart';
import 'package:mira/voice/voice_model_catalog.dart';
import 'package:mira/voice/voice_model_store.dart';
import 'package:sherpa_onnx/sherpa_onnx.dart' as so;

final String? modelsDir = Platform.environment['MIRA_VOICE_MODELS'];
const why = 'modèles absents (définir MIRA_VOICE_MODELS)';

VoiceModelStore storeWithModels(Directory root) {
  for (final e in voiceModelCatalog) {
    Link('${root.path}/${e.folder}').createSync('$modelsDir/${e.folder}');
  }
  return VoiceModelStore(root: root);
}

Future<int> timedStart(SherpaRecognizer r, DictationLanguage l) async {
  final sw = Stopwatch()..start();
  await r.start(l);
  return sw.elapsedMilliseconds;
}

void main() {
  late Directory root;
  setUp(() => root = Directory.systemTemp.createTempSync('voice-warm'));
  tearDown(() => root.deleteSync(recursive: true));

  test(
    'le modèle gardé chaud rend le démarrage suivant presque instantané',
    () async {
      so.initBindings();
      final r = SherpaRecognizer(
        storeWithModels(root),
        keepWarm: const Duration(minutes: 3),
      );
      final cold = await timedStart(r, DictationLanguage.fr);
      r.feed(Float32List(16000));
      await r.stop();
      expect(r.isWarm, isTrue);
      final warm = await timedStart(r, DictationLanguage.fr);
      await r.stop();
      expect(warm, lessThan(cold ~/ 2));
      expect(r.isWarm, isTrue);
    },
    skip: modelsDir == null ? why : false,
  );

  test('durée nulle : le modèle est libéré à la fin de la dictée', () async {
    so.initBindings();
    final r = SherpaRecognizer(storeWithModels(root), keepWarm: Duration.zero);
    await r.start(DictationLanguage.fr);
    await r.stop();
    await Future<void>.delayed(const Duration(milliseconds: 500));
    expect(r.isWarm, isFalse);
  }, skip: modelsDir == null ? why : false);

  test('après la durée de garde, le modèle est libéré', () async {
    so.initBindings();
    final r = SherpaRecognizer(
      storeWithModels(root),
      keepWarm: const Duration(milliseconds: 700),
    );
    await r.start(DictationLanguage.fr);
    await r.stop();
    expect(r.isWarm, isTrue);
    await Future<void>.delayed(const Duration(milliseconds: 1800));
    expect(r.isWarm, isFalse);
  }, skip: modelsDir == null ? why : false);

  test('changer de langue recharge le bon modèle', () async {
    so.initBindings();
    final r = SherpaRecognizer(storeWithModels(root));
    await r.start(DictationLanguage.fr);
    await r.stop();
    final other = await timedStart(r, DictationLanguage.en);
    await r.stop();
    expect(other, greaterThan(800));
    expect(r.isWarm, isTrue);
  }, skip: modelsDir == null ? why : false);
}

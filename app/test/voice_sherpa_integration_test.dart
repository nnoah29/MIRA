import 'dart:io';

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

Future<List<DictationEvent>> dictate(
  SherpaRecognizer r,
  DictationLanguage language,
  String wavPath,
) async {
  final events = <DictationEvent>[];
  final sub = r.events.listen(events.add);
  await r.start(language);
  final wave = so.readWave(wavPath);
  for (var i = 0; i < wave.samples.length; i += 1600) {
    final end = (i + 1600).clamp(0, wave.samples.length);
    r.feed(wave.samples.sublist(i, end));
  }
  await r.stop();
  await sub.cancel();
  return events;
}

String finals(List<DictationEvent> events) =>
    events.whereType<DictationFinal>().map((e) => e.text).join(' ');

void main() {
  late Directory root;
  setUp(() => root = Directory.systemTemp.createTempSync('voice-sherpa'));
  tearDown(() => root.deleteSync(recursive: true));

  test(
    'français : phrase reconnue en direct puis validée, fin comprise',
    () async {
      so.initBindings();
      final r = SherpaRecognizer(storeWithModels(root));
      final wav =
          '$modelsDir/${voiceModelCatalog.first.folder}/test_wavs/0.wav';
      final events = await dictate(r, DictationLanguage.fr, wav);
      expect(events.whereType<DictationPartial>(), isNotEmpty);
      final text = finals(events).toLowerCase();
      expect(text, contains('demandez'));
      expect(text, contains('pays'));
      expect(text.trim(), endsWith('faire'));
    },
    skip: modelsDir == null ? why : false,
  );

  test('anglais : phrase reconnue', () async {
    so.initBindings();
    final r = SherpaRecognizer(storeWithModels(root));
    final wav = '$modelsDir/${voiceModelCatalog.last.folder}/test_wavs/0.wav';
    final events = await dictate(r, DictationLanguage.en, wav);
    expect(finals(events).toLowerCase(), contains('country'));
  }, skip: modelsDir == null ? why : false);

  test('modèle absent : modelMissing', () async {
    final r = SherpaRecognizer(VoiceModelStore(root: root));
    expect(
      () => r.start(DictationLanguage.fr),
      throwsA(
        isA<DictationException>().having(
          (e) => e.problem,
          'problem',
          DictationProblem.modelMissing,
        ),
      ),
    );
  });
}

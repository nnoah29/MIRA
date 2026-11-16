import 'package:flutter_test/flutter_test.dart';
import 'package:mira/voice/sherpa_config.dart';

void main() {
  const dir = '/modeles/fr';

  test(
    'le type du modèle est donné : un seul chargement du fichier de 70 Mo',
    () {
      final config = recognizerConfigFor(dir);
      expect(config.model.modelType, 'zipformer2');
    },
  );

  test('aucun journal de débogage natif', () {
    expect(recognizerConfigFor(dir).model.debug, isFalse);
  });

  test('les fichiers du modèle viennent du dossier donné', () {
    final model = recognizerConfigFor(dir).model;
    expect(model.transducer.encoder, '$dir/encoder.onnx');
    expect(model.transducer.decoder, '$dir/decoder.onnx');
    expect(model.transducer.joiner, '$dir/joiner.onnx');
    expect(model.tokens, '$dir/tokens.txt');
  });

  test('deux fils de calcul, fin de phrase détectée', () {
    final config = recognizerConfigFor(dir);
    expect(config.model.numThreads, 2);
    expect(config.enableEndpoint, isTrue);
  });
}

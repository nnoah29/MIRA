//! Réglage du reconnaisseur sherpa-onnx : fichiers du modèle, type d'architecture, fils de calcul.

import 'package:sherpa_onnx/sherpa_onnx.dart' as so;

const _architecture = 'zipformer2';

/// Réglage pour un dossier de modèle Kroko ; le type donné évite un premier chargement complet pour le deviner.
so.OnlineRecognizerConfig recognizerConfigFor(String modelDir) {
  final model = so.OnlineModelConfig(
    transducer: so.OnlineTransducerModelConfig(
      encoder: '$modelDir/encoder.onnx',
      decoder: '$modelDir/decoder.onnx',
      joiner: '$modelDir/joiner.onnx',
    ),
    tokens: '$modelDir/tokens.txt',
    numThreads: 2,
    modelType: _architecture,
    debug: false,
  );
  return so.OnlineRecognizerConfig(model: model);
}

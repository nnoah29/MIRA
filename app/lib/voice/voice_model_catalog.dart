//! Modèles de reconnaissance pinnés : source, taille et empreinte de chaque archive, par langue.

import 'dictation_types.dart';

/// Un modèle téléchargeable : archive à récupérer et fichiers à en garder.
class VoiceModelEntry {
  const VoiceModelEntry({
    required this.language,
    required this.url,
    required this.archiveBytes,
    required this.sha256,
    required this.folder,
    required this.files,
  });

  final DictationLanguage language;
  final Uri url;
  final int archiveBytes;
  final String sha256;
  final String folder;
  final List<String> files;
}

const _release =
    'https://github.com/k2-fsa/sherpa-onnx/releases/download/asr-models';
const _files = ['encoder.onnx', 'decoder.onnx', 'joiner.onnx', 'tokens.txt'];

/// Modèles Kroko (licence communautaire CC-BY-SA), un par langue, pinnés par empreinte SHA-256.
final List<VoiceModelEntry> voiceModelCatalog = [
  VoiceModelEntry(
    language: DictationLanguage.fr,
    url: Uri.parse(
      '$_release/sherpa-onnx-streaming-zipformer-fr-kroko-2025-08-06.tar.bz2',
    ),
    archiveBytes: 57220361,
    sha256: 'e6ffd3dc43725cd6c8137b05c739f15607d0df946b9b90eb141e10059efca024',
    folder: 'sherpa-onnx-streaming-zipformer-fr-kroko-2025-08-06',
    files: _files,
  ),
  VoiceModelEntry(
    language: DictationLanguage.en,
    url: Uri.parse(
      '$_release/sherpa-onnx-streaming-zipformer-en-kroko-2025-08-06.tar.bz2',
    ),
    archiveBytes: 57267600,
    sha256: 'c8676e5ff9ac2a85296e53ee0fd4d5fb1db6770e7a7647166eeafe349ade6834',
    folder: 'sherpa-onnx-streaming-zipformer-en-kroko-2025-08-06',
    files: _files,
  ),
];

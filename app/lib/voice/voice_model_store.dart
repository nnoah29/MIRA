//! Magasin des modèles de dictée : téléchargement vérifié, extraction, état par langue, suppression.

import 'dart:async';
import 'dart:io';

import 'package:flutter/foundation.dart';

import 'dictation_types.dart';
import 'voice_model_archive.dart';
import 'voice_model_catalog.dart';

/// État d'un modèle pour une langue.
sealed class VoiceModelStatus {
  const VoiceModelStatus();
}

final class VoiceModelNotInstalled extends VoiceModelStatus {
  const VoiceModelNotInstalled();
}

final class VoiceModelDownloading extends VoiceModelStatus {
  const VoiceModelDownloading(this.progress);
  final double progress;
}

final class VoiceModelInstalled extends VoiceModelStatus {
  const VoiceModelInstalled();
}

final class VoiceModelFailed extends VoiceModelStatus {
  const VoiceModelFailed(this.reason);
  final String reason;
}

/// Une installation en cours : son client réseau et son drapeau d'annulation.
class _Run {
  HttpClient? client;
  bool cancelled = false;
}

/// Modèles de dictée installés sur l'appareil ; rien n'est téléchargé sans appel explicite à [install].
class VoiceModelStore extends ChangeNotifier implements ModelAvailability {
  VoiceModelStore({required this.root, List<VoiceModelEntry>? catalog})
    : catalog = catalog ?? voiceModelCatalog {
    _scan();
  }

  final Directory root;
  final List<VoiceModelEntry> catalog;
  final Map<DictationLanguage, VoiceModelStatus> _status = {};
  final Map<DictationLanguage, _Run> _runs = {};

  /// État du modèle d'une langue.
  VoiceModelStatus statusOf(DictationLanguage l) =>
      _status[l] ?? const VoiceModelNotInstalled();

  @override
  bool isInstalled(DictationLanguage language) =>
      statusOf(language) is VoiceModelInstalled;

  /// Dossier du modèle installé, ou null s'il ne l'est pas.
  Directory? directoryOf(DictationLanguage l) =>
      isInstalled(l) ? Directory('${root.path}/${_entry(l).folder}') : null;

  /// Relit le disque et nettoie les restes d'une installation interrompue.
  Future<void> refresh() async {
    _scan();
    notifyListeners();
  }

  /// Télécharge, vérifie l'empreinte, extrait puis range le modèle ; en cas d'échec rien n'est conservé.
  Future<void> install(DictationLanguage l) async {
    if (_runs.containsKey(l)) return;
    final run = _runs[l] = _Run();
    _setStatus(l, const VoiceModelDownloading(0));
    try {
      await _download(l, run);
      _setStatus(l, const VoiceModelInstalled());
    } on Object catch (e) {
      _cleanup(l);
      _setStatus(
        l,
        run.cancelled ? const VoiceModelNotInstalled() : VoiceModelFailed('$e'),
      );
    } finally {
      _runs.remove(l);
    }
  }

  /// Interrompt un téléchargement en cours.
  void cancel(DictationLanguage l) {
    final run = _runs[l];
    if (run == null) return;
    run.cancelled = true;
    run.client?.close(force: true);
  }

  /// Supprime le modèle d'une langue.
  Future<void> remove(DictationLanguage l) async {
    final dir = Directory('${root.path}/${_entry(l).folder}');
    if (dir.existsSync()) await dir.delete(recursive: true);
    _setStatus(l, const VoiceModelNotInstalled());
  }

  /// Entrée du catalogue pour une langue.
  VoiceModelEntry _entry(DictationLanguage l) =>
      catalog.firstWhere((e) => e.language == l);

  /// Fichier temporaire de l'archive en cours de téléchargement.
  File _part(VoiceModelEntry e) =>
      File('${root.path}/${e.folder}.tar.bz2.part');

  /// Dossier temporaire de l'extraction.
  Directory _tmp(VoiceModelEntry e) =>
      Directory('${root.path}/${e.folder}.tmp');

  /// Enchaîne téléchargement, vérification, extraction et rangement atomique.
  Future<void> _download(DictationLanguage l, _Run run) async {
    final e = _entry(l);
    await root.create(recursive: true);
    await _fetch(e, run);
    _stillWanted(run);
    if (await fileSha256(_part(e)) != e.sha256) {
      throw const FormatException('sha256 mismatch');
    }
    _stillWanted(run);
    await extractModelFiles(_part(e).path, _tmp(e).path, e.files);
    _stillWanted(run);
    final target = Directory('${root.path}/${e.folder}');
    if (target.existsSync()) await target.delete(recursive: true);
    await _tmp(e).rename(target.path);
    await _part(e).delete();
  }

  /// Récupère l'archive dans le fichier temporaire en suivant la progression.
  Future<void> _fetch(VoiceModelEntry e, _Run run) async {
    _stillWanted(run);
    final client = run.client = HttpClient()
      ..connectionTimeout = const Duration(seconds: 20);
    final response = await (await client.getUrl(e.url)).close();
    if (response.statusCode != 200) {
      throw HttpException('http status ${response.statusCode}');
    }
    final sink = _part(e).openWrite();
    var received = 0;
    try {
      await for (final chunk in response) {
        _stillWanted(run);
        sink.add(chunk);
        received += chunk.length;
        _setStatus(e.language, VoiceModelDownloading(_ratio(received, e)));
      }
    } finally {
      await sink.close();
      client.close();
    }
    if (received != e.archiveBytes) {
      throw const FormatException('unexpected size');
    }
  }

  /// Interrompt l'installation si l'utilisateur l'a annulée.
  void _stillWanted(_Run run) {
    if (run.cancelled) throw const HttpException('cancelled');
  }

  /// Avancement entre 0 et 1.
  double _ratio(int received, VoiceModelEntry e) =>
      (received / e.archiveBytes).clamp(0.0, 1.0);

  /// Supprime fichier et dossier temporaires d'une langue.
  void _cleanup(DictationLanguage l) {
    final e = _entry(l);
    if (_part(e).existsSync()) _part(e).deleteSync();
    if (_tmp(e).existsSync()) _tmp(e).deleteSync(recursive: true);
  }

  /// Détermine l'état de chaque langue d'après le disque et efface les restes temporaires.
  void _scan() {
    for (final e in catalog) {
      _cleanup(e.language);
      final dir = Directory('${root.path}/${e.folder}');
      final complete =
          dir.existsSync() &&
          e.files.every((f) => File('${dir.path}/$f').existsSync());
      _status[e.language] = complete
          ? const VoiceModelInstalled()
          : const VoiceModelNotInstalled();
    }
  }

  /// Change l'état d'une langue et prévient l'interface.
  void _setStatus(DictationLanguage l, VoiceModelStatus s) {
    _status[l] = s;
    notifyListeners();
  }
}

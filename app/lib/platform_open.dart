import 'dart:io';

import 'package:url_launcher/url_launcher.dart';

import 'platform_info.dart';

/// Ouvre une adresse avec le lanceur du système.
typedef UrlLauncher = Future<bool> Function(Uri url);

/// Lance un programme et attend sa fin ; remplaçable dans les tests.
typedef ProgramRunner = Future<ProcessResult> Function(
  String executable,
  List<String> arguments,
);

const Set<String> _safeSchemes = {'http', 'https', 'mailto'};

/// Ouvre une adresse web ou mail dans le navigateur du système ; sous Windows, si `url_launcher` refuse ou échoue, le gestionnaire de protocole de Windows prend le relais. Tout autre schéma est refusé.
Future<bool> openExternal(
  Uri url, {
  UrlLauncher? primary,
  ProgramRunner? run,
  bool? windows,
}) async {
  if (!_safeSchemes.contains(url.scheme)) return false;
  try {
    if (await (primary ?? launchUrl)(url)) return true;
  } on Object {
    // le lanceur est absent ou a échoué : repli ci-dessous
  }
  if (!(windows ?? runsOnWindows)) return false;
  try {
    final result = await (run ?? Process.run)('rundll32', [
      'url.dll,FileProtocolHandler',
      url.toString(),
    ]);
    return result.exitCode == 0;
  } on ProcessException {
    return false;
  }
}

import 'dart:io' show Platform;

import 'package:flutter/foundation.dart' show kIsWeb;

/// Vrai sous Android ; faux dans un navigateur, où `Platform` lève une erreur.
bool get runsOnAndroid => !kIsWeb && Platform.isAndroid;

/// Vrai sous Linux ; faux dans un navigateur, où `Platform` lève une erreur.
bool get runsOnLinux => !kIsWeb && Platform.isLinux;

/// Vrai sous Windows ; faux dans un navigateur, où `Platform` lève une erreur.
bool get runsOnWindows => !kIsWeb && Platform.isWindows;

/// Vrai là où l'app peut choisir le démon : Linux de bureau et Windows.
bool get platformHasDaemon => runsOnLinux || runsOnWindows;

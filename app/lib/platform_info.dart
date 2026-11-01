import 'dart:io' show Platform;

import 'package:flutter/foundation.dart' show kIsWeb;

/// Vrai sous Android ; faux dans un navigateur, où `Platform` lève une erreur.
bool get runsOnAndroid => !kIsWeb && Platform.isAndroid;

/// Vrai sous Linux ; faux dans un navigateur, où `Platform` lève une erreur.
bool get runsOnLinux => !kIsWeb && Platform.isLinux;

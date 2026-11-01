// Chargeur de la bibliothèque du moteur : natif partout, absent dans un navigateur.
export 'bridge_library_io.dart'
    if (dart.library.js_interop) 'bridge_library_web.dart';

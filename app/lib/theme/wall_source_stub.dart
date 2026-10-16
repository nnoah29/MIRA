import 'wall_theme.dart';

/// Plateformes sans accès au système de fichiers (web) : pas de thème issu du fond d'écran.
bool get wallSupported => false;

Stream<WallSeed?> watchWallSeed() => const Stream.empty();

/// Que faire des images animées (GIF) d'un message : les figer, les animer sous la souris, ou toujours les animer.
enum GifPlayback {
  off('off'),
  hover('hover'),
  always('always');

  final String code;
  const GifPlayback(this.code);

  /// Le réglage enregistré ; tout autre texte donne le réglage par défaut, images figées.
  static GifPlayback parse(String? code) =>
      values.firstWhere((m) => m.code == code, orElse: () => off);
}

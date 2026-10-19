import 'dart:io';

import 'package:url_launcher/url_launcher.dart';

import '../data/models.dart';

/// Désabonnement d'une liste de diffusion : appel direct, page web ou message, selon ce que l'expéditeur annonce.
class Unsubscriber {
  /// Envoi du désabonnement en un clic (RFC 8058) ; vrai si le serveur l'accepte.
  Future<bool> Function(Uri url) postOneClick = _post;

  /// Ouvre une page dans le navigateur.
  Future<bool> Function(Uri url) openPage = launchUrl;

  /// Dernier moyen annoncé par ces messages (`null` : aucun).
  static Unsub? of(Iterable<Msg> msgs) {
    for (final m in msgs.toList().reversed) {
      if (m.unsub != null) return m.unsub;
    }
    return null;
  }

  static Future<bool> _post(Uri url) async {
    final http = HttpClient()..connectionTimeout = const Duration(seconds: 10);
    try {
      final req = await http.postUrl(url);
      req.headers.contentType = ContentType(
        'application',
        'x-www-form-urlencoded',
      );
      req.write('List-Unsubscribe=One-Click');
      final res = await req.close();
      await res.drain<void>();
      return res.statusCode >= 200 && res.statusCode < 300;
    } catch (_) {
      return false;
    } finally {
      http.close();
    }
  }
}

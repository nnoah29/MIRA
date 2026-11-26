import 'dart:io';
import 'dart:typed_data';

import '../data/models.dart';
import '../platform_open.dart';

/// Désabonnement d'une liste de diffusion : appel direct, page web ou message, selon ce que l'expéditeur annonce.
class Unsubscriber {
  /// Envoi du désabonnement en un clic (RFC 8058) ; vrai si le serveur l'accepte.
  late Future<bool> Function(Uri url) postOneClick = _post;

  /// Ouvre une page dans le navigateur.
  Future<bool> Function(Uri url) openPage = openExternal;

  /// Dernier moyen annoncé par ces messages (`null` : aucun).
  static Unsub? of(Iterable<Msg> msgs) {
    for (final m in msgs.toList().reversed) {
      if (m.unsub != null) return m.unsub;
    }
    return null;
  }

  /// Filtre des adresses atteignables ; remplaçable seulement par les tests.
  bool Function(InternetAddress address) allowAddress = isPublicAddress;

  Future<bool> _post(Uri url) async {
    if (url.scheme != 'https' || url.userInfo.isNotEmpty) return false;
    try {
      final found = await InternetAddress.lookup(url.host);
      if (found.isEmpty || !found.every(allowAddress)) return false;
    } catch (_) {
      return false;
    }
    return postHttp(url);
  }

  /// Requête de désabonnement : sans redirection, réponse limitée à 64 Ko, délai de 10 s.
  Future<bool> postHttp(Uri url) async {
    final http = HttpClient()..connectionTimeout = const Duration(seconds: 10);
    try {
      final req = await http.postUrl(url);
      req.followRedirects = false;
      req.headers.contentType = ContentType(
        'application',
        'x-www-form-urlencoded',
      );
      req.write('List-Unsubscribe=One-Click');
      final res = await req.close().timeout(const Duration(seconds: 10));
      var size = 0;
      await for (final chunk in res) {
        size += chunk.length;
        if (size > _maxResponse) return false;
      }
      return res.statusCode >= 200 && res.statusCode < 300;
    } catch (_) {
      return false;
    } finally {
      http.close(force: true);
    }
  }
}

const _maxResponse = 64 * 1024;

/// Vrai pour une adresse publique : ni boucle locale, ni privée, ni locale au lien, ni réservée.
bool isPublicAddress(InternetAddress a) {
  final b = a.rawAddress;
  if (a.isLoopback || a.isLinkLocal || a.isMulticast) return false;
  if (b.length == 4) {
    return !(b[0] == 0 ||
        b[0] == 10 ||
        (b[0] == 172 && b[1] >= 16 && b[1] < 32) ||
        (b[0] == 192 && b[1] == 168) ||
        (b[0] == 100 && b[1] >= 64 && b[1] < 128));
  }
  final mapped =
      b.sublist(0, 10).every((x) => x == 0) && b[10] == 255 && b[11] == 255;
  if (mapped) {
    return isPublicAddress(
      InternetAddress.fromRawAddress(Uint8List.fromList(b.sublist(12))),
    );
  }
  return !(b.every((x) => x == 0) || (b[0] & 0xfe) == 0xfc);
}

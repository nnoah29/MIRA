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

  /// Certificats de confiance du TLS ; remplaçable seulement par les tests (`null` : racines du système).
  SecurityContext? tlsContext;

  /// Résolution des noms ; remplaçable seulement par les tests.
  Future<List<InternetAddress>> Function(String host) resolve =
      InternetAddress.lookup;

  Future<bool> _post(Uri url) async {
    if (url.scheme != 'https' || url.userInfo.isNotEmpty) return false;
    return pinAndPost(url);
  }

  /// Résout le nom, contrôle toutes les adresses, puis poste sans nouvelle résolution.
  Future<bool> pinAndPost(Uri url) async {
    final List<InternetAddress> found;
    try {
      found = await resolve(url.host);
      if (found.isEmpty || !found.every(allowAddress)) return false;
    } catch (_) {
      return false;
    }
    return postHttp(url, pinned: found.first);
  }

  /// Requête de désabonnement : sans redirection, réponse limitée à 64 Ko, délai de 10 s ; `pinned` impose l'adresse de connexion (TLS et en-tête Host restent sur le nom).
  Future<bool> postHttp(Uri url, {InternetAddress? pinned}) async {
    final http = HttpClient(context: tlsContext)
      ..connectionTimeout = const Duration(seconds: 10);
    if (pinned != null) {
      http.connectionFactory = pinnedConnection(pinned, tlsContext);
    }
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

/// Fabrique de connexions qui vise toujours `pinned` ; en https, TLS négocié avec le nom de l'URL (SNI, vérification du certificat).
Future<ConnectionTask<Socket>> Function(Uri, String?, int?) pinnedConnection(
  InternetAddress pinned,
  SecurityContext? context,
) {
  return (uri, proxyHost, proxyPort) async {
    final task = await Socket.startConnect(pinned, uri.port);
    if (uri.scheme != 'https') return task;
    final secured = task.socket.then(
      (socket) => SecureSocket.secure(socket, host: uri.host, context: context),
    );
    return ConnectionTask.fromSocket(secured, task.cancel);
  };
}

/// Vrai pour une adresse publique : ni boucle locale, ni privée, ni locale au lien, ni réservée.
bool isPublicAddress(InternetAddress a) {
  final b = a.rawAddress;
  if (a.isLoopback || a.isLinkLocal || a.isMulticast) return false;
  return b.length == 4 ? _isPublicV4(b) : _isPublicV6(b);
}

bool _isPublicV4(List<int> b) {
  return !(b[0] == 0 ||
      b[0] == 10 ||
      b[0] >= 240 ||
      (b[0] == 172 && b[1] >= 16 && b[1] < 32) ||
      (b[0] == 192 && b[1] == 168) ||
      (b[0] == 192 && b[1] == 0 && b[2] == 0) ||
      (b[0] == 198 && (b[1] == 18 || b[1] == 19)) ||
      (b[0] == 100 && b[1] >= 64 && b[1] < 128));
}

bool _isPublicV6(List<int> b) {
  final mapped =
      b.sublist(0, 10).every((x) => x == 0) && b[10] == 255 && b[11] == 255;
  if (mapped) return _embeddedIsPublic(b);
  const nat64 = [0, 0x64, 0xff, 0x9b, 0, 0, 0, 0, 0, 0, 0, 0];
  if (_startsWith(b, nat64)) return _embeddedIsPublic(b);
  return !(b.sublist(0, 12).every((x) => x == 0) ||
      (b[0] & 0xfe) == 0xfc ||
      (b[0] == 0xfe && (b[1] & 0xc0) == 0xc0));
}

bool _startsWith(List<int> b, List<int> prefix) {
  for (var i = 0; i < 12; i++) {
    if (b[i] != prefix[i]) return false;
  }
  return true;
}

bool _embeddedIsPublic(List<int> b) {
  return isPublicAddress(
    InternetAddress.fromRawAddress(Uint8List.fromList(b.sublist(12))),
  );
}

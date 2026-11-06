import 'byte_channel.dart';

/// Où joindre le démon : socket Unix ou TCP loopback, et le jeton local qui l'authentifie.
class DaemonEndpoint {
  /// Chemin de la socket Unix (prioritaire s'il est donné).
  final String? unixSocket;
  final String? tcpHost;
  final int? tcpPort;

  /// Jeton local ; jamais affiché.
  final String token;

  const DaemonEndpoint({
    this.unixSocket,
    this.tcpHost,
    this.tcpPort,
    required this.token,
  });

  /// Ouvre un canal vers le démon.
  Future<ByteChannel> connect() {
    final path = unixSocket;
    if (path != null) return SocketChannel.connectUnix(path);
    return SocketChannel.connectTcp(tcpHost ?? '127.0.0.1', tcpPort ?? 0);
  }

  @override
  String toString() =>
      'DaemonEndpoint(${unixSocket ?? '$tcpHost:$tcpPort'}, token: ***)';
}

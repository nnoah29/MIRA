import 'engine_client.dart';

/// Fournisseur proposé par le moteur (Gmail, Outlook, Yahoo, Autre…).
class ProviderInfo {
  final String id, name;
  final List<String> domains;

  /// Méthodes d'authentification : `oauth2`, `app_password`, `password`.
  final List<String> methods;
  const ProviderInfo(this.id, this.name, this.domains, this.methods);

  /// Le fournisseur de repli n'a pas de domaine : les serveurs se saisissent à la main.
  bool get isManual => domains.isEmpty;

  static ProviderInfo fromJson(Map<String, dynamic> j) => ProviderInfo(
    '${j['id']}',
    '${j['display_name']}',
    [for (final d in (j['domains'] as List? ?? const [])) '$d'],
    [
      for (final m in (j['auth_methods'] as List? ?? const []))
        '${(m as Map)['type']}',
    ],
  );
}

/// Chiffrement d'une connexion : `tls`, `start_tls` ou `plaintext`.
const securities = ['tls', 'start_tls', 'plaintext'];

/// Serveur IMAP ou SMTP : hôte, port, chiffrement.
class Endpoint {
  final String host;
  final int port;
  final String security;
  const Endpoint(this.host, this.port, this.security);

  Map<String, dynamic> toJson() => {
    'host': host,
    'port': port,
    'security': security,
  };

  static Endpoint fromJson(Map<String, dynamic> j) =>
      Endpoint('${j['host']}', (j['port'] as num).toInt(), '${j['security']}');
}

/// Ce que le moteur retient pour une adresse : fournisseur et serveurs qui seront contactés.
class Preparation {
  final ProviderInfo provider;
  final Endpoint imap, smtp;
  const Preparation(this.provider, this.imap, this.smtp);
}

/// Compte à ajouter tel que saisi dans le panneau.
class NewAccount {
  final String address, displayName, providerId, method;
  final Endpoint? imap, smtp;
  const NewAccount({
    required this.address,
    required this.displayName,
    required this.providerId,
    required this.method,
    this.imap,
    this.smtp,
  });

  Map<String, dynamic> toDraft() => {
    'address': address,
    'display_name': displayName.isEmpty ? address : displayName,
    'provider': providerId,
    'servers': imap == null || smtp == null
        ? null
        : {'imap': imap!.toJson(), 'smtp': smtp!.toJson()},
    'auth': method,
  };
}

/// Ajout d'un compte par le contrat du moteur : fournisseurs, préparation, test, OAuth, ajout.
class AccountSetup {
  final EngineClient client;
  const AccountSetup(this.client);

  Future<List<ProviderInfo>> providers() async {
    final raw = await client.call(req('provider', 'list_providers'));
    return [
      for (final p in (raw['data'] as List))
        ProviderInfo.fromJson((p as Map).cast()),
    ];
  }

  /// Fournisseur et serveurs que le moteur choisirait pour cette adresse.
  Future<Preparation> prepare(String address, {String? providerId}) async {
    final raw = await client.call(
      req('account', 'prepare_account', {
        'address': address,
        'provider': providerId,
      }),
    );
    final d = (raw['data'] as Map).cast<String, dynamic>();
    final servers = (d['servers'] as Map).cast<String, dynamic>();
    return Preparation(
      ProviderInfo.fromJson((d['provider'] as Map).cast()),
      Endpoint.fromJson((servers['imap'] as Map).cast()),
      Endpoint.fromJson((servers['smtp'] as Map).cast()),
    );
  }

  Map<String, dynamic> _credential(String method, String secret) => {
    'type': method,
    'value': secret,
  };

  /// Teste la connexion sans rien enregistrer ; lève [EngineException] avec la raison.
  Future<void> test(NewAccount a, String secret) async {
    await client.call(
      req('account', 'test_connection', {
        'draft': a.toDraft(),
        'credential': _credential(a.method, secret),
      }),
    );
  }

  /// Ajoute le compte (secret envoyé une seule fois) ; rend son identifiant.
  Future<String> add(NewAccount a, String secret, String key) async {
    final raw = await client.call(
      req('account', 'add_account', {
        'draft': a.toDraft(),
        'credential': _credential(a.method, secret),
        'idempotency_key': key,
      }),
    );
    return '${(raw['data'] as Map)['id']}';
  }

  /// Commence un flux OAuth : identifiant du flux et adresse à ouvrir dans le navigateur.
  Future<({String flow, String url})> startOAuth(
    String address,
    String providerId,
  ) async {
    final raw = await client.call(
      req('auth', 'start_oauth', {'address': address, 'provider': providerId}),
    );
    final d = (raw['data'] as Map).cast<String, dynamic>();
    return (flow: '${d['flow']}', url: '${d['authorize_url']}');
  }

  /// Attend la redirection du navigateur (long) ; l'utilisateur autorise dans la page ouverte.
  Future<void> completeOAuth(String flow) async {
    await client.call(req('auth', 'complete_oauth', {'flow': flow}));
  }

  Future<void> cancelOAuth(String flow) async {
    await client.call(req('auth', 'cancel_oauth', {'flow': flow}));
  }

  /// Ajoute un compte autorisé par OAuth (le flux a déjà rangé les jetons).
  Future<String> addOAuth(NewAccount a, String flow, String key) async {
    final raw = await client.call(
      req('account', 'add_account', {
        'draft': a.toDraft(),
        'credential': {
          'type': 'oauth_flow',
          'value': {'flow': flow},
        },
        'idempotency_key': key,
      }),
    );
    return '${(raw['data'] as Map)['id']}';
  }

  /// Lance la première synchronisation du compte.
  Future<void> startSync(String accountId) async {
    await client.call(
      req('sync', 'start_sync', {
        'scope': {'type': 'account', 'account': accountId},
      }),
    );
  }
}

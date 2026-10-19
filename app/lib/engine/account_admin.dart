import 'account_setup.dart';
import 'engine_client.dart';

/// Compte tel que le moteur le décrit, avec son état et sa file d'actions.
class AccountDetail {
  final String id, address, displayName, provider, auth;

  /// `connected`, `offline`, `auth_required`, `paused` ou `error`.
  final String state;
  final String? errorCode;
  final int windowDays, pendingActions;
  final Endpoint? imap, smtp;
  const AccountDetail({
    required this.id,
    required this.address,
    required this.displayName,
    required this.provider,
    required this.auth,
    required this.state,
    required this.windowDays,
    required this.pendingActions,
    this.errorCode,
    this.imap,
    this.smtp,
  });

  bool get paused => state == 'paused';
  bool get needsAuth => state == 'auth_required';

  static AccountDetail fromJson(Map<String, dynamic> a, int pending) {
    final servers = (a['servers'] as Map?)?.cast<String, dynamic>();
    final st = (a['state'] as Map).cast<String, dynamic>();
    return AccountDetail(
      id: '${a['id']}',
      address: '${a['address']}',
      displayName: '${a['display_name']}',
      provider: '${a['provider']}',
      auth: '${a['auth']}',
      state: '${st['type']}',
      errorCode: st['code'] as String?,
      windowDays: ((a['sync'] as Map)['window_days'] as num).toInt(),
      pendingActions: pending,
      imap: servers == null
          ? null
          : Endpoint.fromJson((servers['imap'] as Map).cast()),
      smtp: servers == null
          ? null
          : Endpoint.fromJson((servers['smtp'] as Map).cast()),
    );
  }
}

/// Identité d'envoi : adresse, nom, signature.
class IdentityDetail {
  final String id, accountId, address, displayName;
  final String? signature;
  final bool isDefault;
  const IdentityDetail({
    required this.id,
    required this.accountId,
    required this.address,
    required this.displayName,
    required this.signature,
    required this.isDefault,
  });

  static IdentityDetail fromJson(Map<String, dynamic> j) => IdentityDetail(
    id: '${j['id']}',
    accountId: '${j['account']}',
    address: '${j['address']}',
    displayName: '${j['display_name']}',
    signature: j['signature'] as String?,
    isDefault: j['is_default'] == true,
  );
}

/// Gestion des comptes et des identités par le contrat du moteur.
/// Boîte d'un compte et son état de synchronisation.
class BoxChoice {
  final String id, name, role;
  final bool synced;
  const BoxChoice(this.id, this.name, this.role, this.synced);
}

/// Action du compte en attente d'application sur le serveur.
class QueuedAction {
  final String id, kind, state;
  final int targets, attempts;
  final String? error;
  const QueuedAction(
    this.id,
    this.kind,
    this.state,
    this.targets,
    this.attempts,
    this.error,
  );
}

/// État de synchronisation d'un compte selon le moteur.
class SyncStatus {
  final int? lastSyncedAt;
  final String? lastError;
  final bool syncing;
  const SyncStatus(this.lastSyncedAt, this.lastError, this.syncing);
}

class AccountAdmin {
  final EngineClient client;
  const AccountAdmin(this.client);

  /// Comptes avec leur état et le nombre d'actions en attente.
  Future<List<AccountDetail>> accounts() async {
    final raw = await client.call(req('engine', 'get_status'));
    final list = ((raw['data'] as Map)['accounts'] as List);
    return [
      for (final e in list)
        AccountDetail.fromJson(
          ((e as Map)['account'] as Map).cast(),
          (e['pending_actions'] as num).toInt(),
        ),
    ];
  }

  /// Boîtes du compte, avec celles que le moteur synchronise.
  Future<List<BoxChoice>> mailboxes(String id) async {
    final raw = await client.call(
      req('mailbox', 'list_mailboxes', {'account': id}),
    );
    return [
      for (final b in (raw['data'] as List).cast<Map>())
        BoxChoice(
          '${b['id']}',
          '${b['display_name'] ?? b['name'] ?? b['id']}',
          '${b['role']}',
          b['sync_enabled'] != false,
        ),
    ];
  }

  /// Actions pas encore appliquées sur le serveur (hors envois, visibles dans la boîte d'envoi).
  Future<List<QueuedAction>> pendingActions(String id) async {
    final raw = await client.call(
      req('queue', 'list_actions', {
        'account': id,
        'states': ['pending', 'in_progress', 'failed', 'uncertain'],
      }),
    );
    final list = ((raw['data'] as Map)['data'] as List).cast<Map>();
    return [
      for (final a in list)
        if ((a['kind'] as Map?)?['type'] != 'send')
          QueuedAction(
            '${a['id']}',
            '${(a['kind'] as Map?)?['type']}',
            '${a['state']}',
            (a['targets'] as List? ?? const []).length,
            (a['attempts'] as num?)?.toInt() ?? 0,
            a['last_error'] as String?,
          ),
    ];
  }

  /// Dernière synchronisation, dernière erreur et synchro en cours, pour chaque compte.
  Future<Map<String, SyncStatus>> syncStatus() async {
    final raw = await client.call(req('sync', 'get_sync_status'));
    final list = ((raw['data'] as Map)['data'] as List).cast<Map>();
    return {
      for (final e in list)
        '${e['account']}': SyncStatus(
          (e['last_synced_at'] as num?)?.toInt(),
          e['last_error'] as String?,
          e['syncing'] == true,
        ),
    };
  }

  /// Abandonne une action en attente.
  Future<void> cancelAction(String action) =>
      client.call(req('queue', 'cancel_action', {'action': action}));

  /// Relance tout de suite une action échouée.
  Future<void> retryAction(String action) =>
      client.call(req('queue', 'retry_action', {'action': action}));

  /// Ne synchronise plus que ces boîtes.
  Future<void> setSyncedMailboxes(String id, List<String> mailboxes) =>
      client.call(
        req('mailbox', 'set_synced_mailboxes', {
          'account': id,
          'mailboxes': mailboxes,
        }),
      );

  Future<void> _account(String op, String id, [Map<String, dynamic>? extra]) =>
      client.call(req('account', op, {'account': id, ...?extra}));

  Future<void> rename(String id, String displayName) =>
      _account('update_account', id, {
        'changes': {'display_name': displayName},
      });

  Future<void> setWindowDays(String id, int days) =>
      _account('update_account', id, {
        'changes': {'window_days': days},
      });

  /// Remplace les serveurs (le moteur vérifie la connexion avant d'accepter).
  Future<void> setServers(String id, Endpoint imap, Endpoint smtp) =>
      _account('update_account', id, {
        'changes': {
          'servers': {'imap': imap.toJson(), 'smtp': smtp.toJson()},
        },
      });

  Future<void> pause(String id) => _account('pause_account', id);
  Future<void> resume(String id) => _account('resume_account', id);

  /// Supprime le compte du profil : messages locaux, secrets et file d'actions (le serveur n'est pas touché).
  Future<void> delete(String id) => _account('delete_account', id);

  /// Enregistre un nouveau mot de passe (ou mot de passe d'application) pour un compte déconnecté.
  Future<void> storePassword(String id, String password) => client.call(
    req('auth', 'store_app_password', {'account': id, 'password': password}),
  );

  /// Commence une nouvelle autorisation OAuth ; rend le flux et l'adresse à ouvrir.
  Future<({String flow, String url})> reauthenticate(String id) async {
    final raw = await client.call(
      req('auth', 'reauthenticate', {'account': id}),
    );
    final d = (raw['data'] as Map).cast<String, dynamic>();
    return (flow: '${d['flow']}', url: '${d['authorize_url']}');
  }

  Future<void> completeOAuth(String flow) =>
      client.call(req('auth', 'complete_oauth', {'flow': flow}));

  Future<List<IdentityDetail>> identities(String accountId) async {
    final raw = await client.call(
      req('identity', 'list_identities', {'account': accountId}),
    );
    return [
      for (final i in (raw['data'] as List))
        IdentityDetail.fromJson((i as Map).cast()),
    ];
  }

  Future<void> addIdentity({
    required String accountId,
    required String address,
    required String displayName,
    String? signature,
    bool isDefault = false,
  }) => client.call(
    req('identity', 'add_identity', {
      'account': accountId,
      'address': address,
      'display_name': displayName,
      'signature': signature,
      'is_default': isDefault,
    }),
  );

  Future<void> updateIdentity(
    String id, {
    String? displayName,
    String? signature,
  }) => client.call(
    req('identity', 'update_identity', {
      'identity': id,
      'display_name': displayName,
      'signature': signature,
    }),
  );

  Future<void> setDefaultIdentity(String id) =>
      client.call(req('identity', 'set_default_identity', {'identity': id}));
}

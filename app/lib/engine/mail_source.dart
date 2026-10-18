import 'package:flutter/foundation.dart';

import '../data/models.dart';
import 'account_admin.dart';
import 'account_setup.dart';
import 'compose_admin.dart';
import 'label_admin.dart';
import 'message_actions.dart';
import 'sync_state.dart';

/// Résultat d'une recherche sur le serveur : messages déjà connus du moteur, et nombre de ceux qu'il n'a pas synchronisés.
class ServerSearch {
  final List<Mail> found;
  final int unsynced;
  final bool truncated;
  const ServerSearch(this.found, this.unsynced, this.truncated);
}

/// Origine des données affichées : comptes, boîtes chargées, et enrichissement d'un message.
abstract class MailSource {
  /// Comptes de l'utilisateur (au moins un).
  List<Account> get accounts;

  /// Messages déjà chargés, par identifiant de compte.
  Map<String, List<Mail>> get boxes;

  /// Charge corps et pièces jointes d'une conversation ; rend `true` si elle a changé.
  Future<bool> hydrate(Mail mail, {VoidCallback? onProgress});

  /// Autorise le contenu distant d'un message (ou de son expéditeur) puis recharge son HTML.
  Future<bool> allowRemote(Msg msg, {bool sender = false});

  /// Ajout de comptes par le moteur (`null` : données de démonstration).
  AccountSetup? get setup;

  /// Demande au moteur de synchroniser tous les comptes tout de suite.
  Future<void> syncNow();

  /// Gestion des comptes et identités (`null` : données de démonstration).
  AccountAdmin? get admin;

  /// Identités d'envoi par compte.
  Map<String, List<IdentityDetail>> get identities;

  /// Relit les identités de tous les comptes.
  Future<void> refreshIdentities();

  /// Relit la liste des comptes ; rend les comptes apparus depuis la dernière lecture.
  Future<List<Account>> refreshAccounts();

  /// Libellés de tous les comptes, par identifiant (`null` : données de démonstration).
  Map<String, LabelDef> get labels;

  /// Boîtes du serveur rattachables à un dossier (libellés, « tous les messages »), par identifiant.
  Map<String, LabelDef> get mappable;

  /// Rattachements boîte du serveur → dossier de l'interface (modifiable ; relire le compte ensuite).
  Map<String, String> get folderMap;

  /// Création, renommage, suppression et pose des libellés (`null` : démonstration).
  LabelAdmin? get labelAdmin;

  /// Contenu d'une pièce jointe du moteur (`null` : illisible ou sans identifiant).
  Future<Uint8List?> attachmentBytes(Attachment a);

  /// Actions sur les messages (`null` : démonstration).
  MessageActions? get messageActions;

  /// Boîte du serveur qui tient le rôle de ce dossier pour ce compte (`null` : inconnue).
  String? mailboxOf(String account, String folder);

  /// État de synchronisation par compte (hors ligne, en cours, échec).
  Map<String, SyncState> get syncStates;

  /// Émet à chaque changement de [syncStates].
  Stream<void> get syncEvents;

  /// Télécharge du serveur `days` jours de plus pour ce dossier ; rend l'opération (`null` : dossier inconnu).
  Future<String?> extendWindow(String account, String folder, int days);

  /// Annule une opération en cours du moteur.
  Future<void> cancelOperation(String operation);

  /// Nombre de non lus exact du dossier selon le moteur (`null` : inconnu).
  int? unreadOf(String account, String folder);

  /// Vrai si ce compte a des messages plus anciens que ceux chargés.
  bool hasMore(String account);

  /// Demande une page de plus par boîte à la prochaine lecture du compte.
  void readDeeper(String account);

  /// Recherche plein texte dans tout l'index local du moteur (tous comptes, toute l'ancienneté).
  Future<List<Mail>> search(String query);

  /// Cherche aussi sur le serveur (corps non téléchargés compris) ; plus lent que la recherche locale.
  Future<ServerSearch> searchServer(String query);

  /// Brouillons et envois (`null` : démonstration).
  ComposeAdmin? get composer;

  /// Relit les boîtes du compte depuis le moteur.
  Future<List<Mail>> reload(Account account);

  /// Événements « quelque chose a changé pour ce compte » (identifiant de compte).
  Stream<String> get changes;
}

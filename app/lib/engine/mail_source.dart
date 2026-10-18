import 'package:flutter/foundation.dart';

import '../data/models.dart';
import 'account_admin.dart';
import 'account_setup.dart';
import 'label_admin.dart';

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

  /// Relit les boîtes du compte depuis le moteur.
  Future<List<Mail>> reload(Account account);

  /// Événements « quelque chose a changé pour ce compte » (identifiant de compte).
  Stream<String> get changes;
}

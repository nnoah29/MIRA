import '../data/models.dart';

/// Origine des données affichées : comptes, boîtes chargées, et enrichissement d'un message.
abstract class MailSource {
  /// Comptes de l'utilisateur (au moins un).
  List<Account> get accounts;

  /// Messages déjà chargés, par identifiant de compte.
  Map<String, List<Mail>> get boxes;

  /// Charge corps et pièces jointes d'une conversation ; rend `true` si elle a changé.
  Future<bool> hydrate(Mail mail);

  /// Relit les boîtes du compte depuis le moteur.
  Future<List<Mail>> reload(Account account);

  /// Événements « quelque chose a changé pour ce compte » (identifiant de compte).
  Stream<String> get changes;
}

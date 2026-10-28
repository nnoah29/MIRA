/// Adresses et noms sous lesquels on peut me joindre ou me citer, en minuscules.
class Identity {
  final Set<String> addresses, names;
  const Identity(this.addresses, this.names);

  /// Identité à partir d'adresses et de noms d'affichage ; le prénom (3 lettres et plus) compte aussi.
  factory Identity.from(Iterable<String> addresses, Iterable<String> names) {
    final shown = names
        .map((n) => n.trim().toLowerCase())
        .where((n) => n.isNotEmpty);
    return Identity(
      {for (final a in addresses) a.toLowerCase()},
      {
        for (final n in shown) ...[
          n,
          if (n.split(' ').first.length > 2) n.split(' ').first,
        ],
      },
    );
  }

  /// Cette adresse est-elle l'une des miennes ?
  bool owns(String email) => addresses.contains(email.toLowerCase());
}

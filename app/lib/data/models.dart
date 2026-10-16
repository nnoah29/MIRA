import 'dart:typed_data';

/// Une personne (nom + adresse).
class Person {
  final String name;
  final String email;
  const Person(this.name, this.email);
}

class Attachment {
  final String n; // nom
  final String t; // type (PDF, PNG…)
  final String s; // taille formatée
  final Uint8List? bytes; // aperçu pour les images ajoutées
  const Attachment(this.n, this.t, this.s, {this.bytes});
}

/// Fragment de texte stylé (équivalent d'un nœud inline HTML).
class Span {
  final String text;
  final bool bold, italic, underline, code, at, link;
  const Span(
    this.text, {
    this.bold = false,
    this.italic = false,
    this.underline = false,
    this.code = false,
    this.at = false,
    this.link = false,
  });
}

sealed class Block {
  const Block();
}

class Para extends Block {
  final List<Span> spans;

  /// Contenu en ligne (hors `<p>`) : pas de marge basse.
  final bool loose;
  const Para(this.spans, {this.loose = false});
  String get text => spans.map((s) => s.text).join();
}

class Quote extends Block {
  final List<Block> children;
  const Quote(this.children);
}

class Msg {
  Person from;
  int date;
  List<Block> body;
  List<Attachment> att;

  /// Identifiant du message dans le moteur (`null` : message local).
  final String? ref;

  /// Le moteur annonce des pièces jointes (liste lue à l'ouverture).
  final bool hasAttachments;

  /// Corps et pièces jointes déjà demandés au moteur.
  bool hydrated = false;
  Msg({
    required this.from,
    required this.date,
    required this.body,
    List<Attachment>? att,
    this.ref,
    this.hasAttachments = false,
  }) : att = att ?? [];
  Msg copy() => Msg(
    from: from,
    date: date,
    body: body,
    att: [...att],
    ref: ref,
    hasAttachments: hasAttachments,
  )..hydrated = hydrated;
}

class DraftMeta {
  String from;
  List<Person> bcc;
  String? replyTo;
  String mode;
  String? rawText;
  List<int>? rawFlags;
  DraftMeta({
    required this.from,
    this.bcc = const [],
    this.replyTo,
    this.mode = 'new',
    this.rawText,
    this.rawFlags,
  });
}

class Mail {
  String id;
  String folder;
  Person from;
  List<Person> to;
  List<Person> cc;
  String subject;
  int date;
  bool unread;
  bool starred;
  List<String> labels;
  List<String> flags;
  String? snooze;
  String? sched;
  List<Msg> msgs;
  DraftMeta? draft;

  /// Compte auquel appartient ce message.
  String account;
  Mail({
    required this.id,
    required this.folder,
    required this.from,
    required this.to,
    this.cc = const [],
    required this.subject,
    required this.date,
    this.unread = false,
    this.starred = false,
    List<String>? labels,
    List<String>? flags,
    this.snooze,
    this.sched,
    required this.msgs,
    this.draft,
    String? account,
  }) : account = account ?? currentAccountId,
       labels = labels ?? [],
       flags = flags ?? [];
}

class LabelDef {
  final String n;
  final int c;
  const LabelDef(this.n, this.c);
}

class FolderDef {
  final String id;
  final String n;
  final String icon;
  const FolderDef(this.id, this.n, this.icon);
}

/// Compte mail (fictif).
class Account {
  final String id;
  final String name;
  final String email;
  const Account(this.id, this.name, this.email);

  Map<String, String> toJson() => {'id': id, 'name': name, 'email': email};
  static Account fromJson(Map<String, dynamic> j) =>
      Account(j['id'] as String, j['name'] as String, j['email'] as String);
}

/// Comptes proposés par défaut (le premier est le compte principal).
const defaultAccounts = <Account>[
  Account('a1', 'Noah Toffa', 'noah@kernel.epitech.africa'),
  Account('a2', 'Noah Toffa', 'toffanoah@gmail.com'),
  Account('a3', 'StockBot', 'contact@stockbot.africa'),
];

/// Compte « courant » : celui auquel sont rattachés les nouveaux messages.
String currentAccountId = defaultAccounts[0].id;

/// Tous les comptes connus (tenu à jour par le store).
List<Account> knownAccounts = [...defaultAccounts];

Account? accountById(String id) {
  for (final a in knownAccounts) {
    if (a.id == id) return a;
  }
  return null;
}

Account? accountByEmail(String email) {
  final e = email.toLowerCase();
  for (final a in knownAccounts) {
    if (a.email.toLowerCase() == e) return a;
  }
  return null;
}

/// Cette adresse est-elle l'une des miennes (n'importe quel compte) ?
bool isMine(String email) => accountByEmail(email) != null;

/// Identité du compte actif (change quand on bascule de compte).
Person me = Person(defaultAccounts[0].name, defaultAccounts[0].email);

const labelDefs = <String, LabelDef>{
  'projets': LabelDef('Projets', 0xFFe8542b),
  'candidatures': LabelDef('Candidatures', 0xFF6aa8d9),
  'factures': LabelDef('Factures', 0xFF8bc48a),
  'equipe': LabelDef('Équipe', 0xFFd9b46a),
};

const folderDefs = <FolderDef>[
  FolderDef('inbox', 'Réception', 'inbox'),
  FolderDef('starred', 'Favoris', 'star'),
  FolderDef('snoozed', 'Reportés', 'clock'),
  FolderDef('sent', 'Envoyés', 'send'),
  FolderDef('drafts', 'Brouillons', 'file'),
  FolderDef('archive', 'Archives', 'archive'),
  FolderDef('spam', 'Spam', 'warn'),
  FolderDef('trash', 'Corbeille', 'trash'),
];

FolderDef folderById(String id) =>
    folderDefs.firstWhere((f) => f.id == id, orElse: () => folderDefs.first);

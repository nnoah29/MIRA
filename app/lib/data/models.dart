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

  /// Fichier local d'origine (pièce jointe ajoutée à la rédaction).
  final String? path;

  /// Identifiant de la pièce jointe dans le moteur (`null` : fichier local ou démonstration).
  final String? id;
  const Attachment(this.n, this.t, this.s, {this.bytes, this.path, this.id});
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

/// Moyens de désabonnement annoncés par l'expéditeur.
class Unsub {
  final String? url, mailto;
  final bool oneClick;
  const Unsub({this.url, this.mailto, this.oneClick = false});
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

  /// Désabonnement proposé par l'expéditeur (`null` : aucun).
  final Unsub? unsub;

  /// Corps et pièces jointes déjà demandés au moteur.
  bool hydrated = false;

  /// Chargement terminé (réussi ou non) : l'aperçu n'est plus un état d'attente.
  bool loaded = false;

  /// HTML déjà assaini par le moteur (`null` : message en texte seul).
  String? html;

  /// Ressources distantes retirées par le moteur tant que l'utilisateur ne les autorise pas.
  List<String> blocked = const [];

  /// Images incorporées (`cid:`), par identifiant de contenu, lues dans les pièces jointes.
  Map<String, Uint8List> inline = {};
  Msg({
    required this.from,
    required this.date,
    required this.body,
    List<Attachment>? att,
    this.ref,
    this.hasAttachments = false,
    this.unsub,
  }) : att = att ?? [];
  Msg copy() =>
      Msg(
          from: from,
          date: date,
          body: body,
          att: [...att],
          ref: ref,
          hasAttachments: hasAttachments,
          unsub: unsub,
        )
        ..hydrated = hydrated
        ..loaded = loaded
        ..html = html
        ..blocked = blocked
        ..inline = inline;
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

/// État d'un envoi dans la boîte d'envoi du moteur.
class OutboxInfo {
  final String action, draft, state, subject;
  final String? error;

  /// Instant prévu pour le départ, en millisecondes.
  final int at;
  const OutboxInfo({
    required this.action,
    required this.draft,
    required this.state,
    required this.at,
    this.subject = '',
    this.error,
  });

  bool get waiting => state == 'pending' || state == 'in_progress';
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

  /// Envoi en attente, en cours, échoué ou incertain (`null` : message ordinaire).
  OutboxInfo? outbox;

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
    this.outbox,
    String? account,
  }) : account = account ?? currentAccountId,
       labels = labels ?? [],
       flags = flags ?? [];
}

class LabelDef {
  final String n;
  final int c;

  /// Compte propriétaire (`null` : libellé de démonstration).
  final String? account;

  /// Boîte « tous les messages » ou « favoris » : rattachable, mais ni renommée ni supprimée.
  final bool system;
  const LabelDef(this.n, this.c, {this.account, this.system = false});
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
bool isMine(String email) {
  final e = email.toLowerCase();
  return accountByEmail(email) != null ||
      knownSenders.any((s) => s.email.toLowerCase() == e);
}

/// Adresse sous laquelle on peut écrire (identité d'un compte) : nom, signature et compte d'envoi.
class Sender {
  final String email, name, accountId;
  final String? signature;
  final bool isDefault;
  const Sender(
    this.email,
    this.name,
    this.accountId, {
    this.signature,
    this.isDefault = false,
  });
}

/// Identités d'envoi connues (tenues à jour par le store).
List<Sender> knownSenders = [];

/// Identité du compte actif (change quand on bascule de compte).
Person me = Person(defaultAccounts[0].name, defaultAccounts[0].email);

const seedLabelDefs = <String, LabelDef>{
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
  FolderDef('outbox', 'Boîte d’envoi', 'outbox'),
  FolderDef('archive', 'Archives', 'archive'),
  FolderDef('spam', 'Spam', 'warn'),
  FolderDef('trash', 'Corbeille', 'trash'),
];

FolderDef folderById(String id) =>
    folderDefs.firstWhere((f) => f.id == id, orElse: () => folderDefs.first);

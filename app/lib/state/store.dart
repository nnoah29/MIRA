import 'dart:async';
import 'dart:convert';

import 'package:file_picker/file_picker.dart';
import 'package:flutter/foundation.dart' show Uint8List;
import 'package:flutter/material.dart';
import 'package:shared_preferences/shared_preferences.dart';

import '../data/format.dart';
import '../data/html_lite.dart';
import '../data/models.dart';
import '../data/seed.dart';
import '../engine/blocks_html.dart';
import '../engine/compose_admin.dart';
import '../engine/engine_client.dart';
import '../engine/label_admin.dart';
import '../engine/message_actions.dart';
import '../engine/mail_source.dart';
import '../theme/wall_source.dart';
import '../theme/wall_theme.dart';
import '../theme/app_theme.dart';
import 'assistant.dart';
import 'contacts.dart';
import 'folder_map.dart';
import 'image_policy.dart';
import 'rich_controller.dart';
import 'unsubscribe.dart';
import '../i18n/i18n.dart';

enum PaneView { empty, read, compose, assistant, settings, addAccount }

class ToastItem {
  final int id;
  final String msg;
  final String? label;
  final VoidCallback? cb;
  final bool err;
  bool out = false;
  ToastItem(this.id, this.msg, this.label, this.cb, this.err);
}

class ChatMsg {
  final String role; // 'me' | 'ai'
  bool done;
  String shown;
  String text;
  List<String>? draft;
  List<String>? refs;
  ChatMsg({
    required this.role,
    this.done = true,
    this.shown = '',
    required this.text,
    this.draft,
    this.refs,
  });
}

class ComposeState {
  String? id;
  String? replyTo;
  String mode;
  List<Person> to, cc, bcc;
  List<Attachment> atts;
  String from;
  final TextEditingController subject = TextEditingController();
  final RichController body = RichController();
  final FocusNode toFocus = FocusNode();
  final FocusNode bodyFocus = FocusNode();
  List<Block>? quote;
  bool dirty = false;
  Timer? timer;
  bool showBcc;
  String saved = '';
  bool center = false;
  int shake = 0;

  /// Signature insérée dans le corps (vide : aucune) ; remplacée si l'expéditeur change.
  String sigText = '';

  /// L'expéditeur a été choisi à la main : le brouillon existe même sans autre contenu.
  bool fromChosen = false;

  /// Brouillon correspondant dans le moteur (`null` : pas encore créé).
  String? engineId;

  /// Pièces jointes déjà confiées au moteur, dans son ordre.
  List<Attachment> engineAtts = [];

  /// Le brouillon du moteur a changé depuis la dernière copie sur le serveur.
  bool copyStale = false;

  /// Enregistrements en cours, exécutés l'un après l'autre.
  Future<void> push = Future.value();
  ComposeState({
    this.id,
    this.replyTo,
    this.mode = 'new',
    required this.to,
    required this.cc,
    required this.bcc,
    required this.atts,
    required this.from,
    this.quote,
  }) : showBcc = bcc.isNotEmpty;
}

/// Paramètres d'ouverture d'une rédaction.
class ComposeInit {
  final String? id, replyTo, from;
  final String mode;
  final List<Person> to, cc, bcc;
  final String subject;
  final List<Attachment> atts;
  final String? text;
  final List<int>? flags;
  final List<Block>? blocks;
  final List<Block>? quote;
  const ComposeInit({
    this.id,
    this.replyTo,
    this.from,
    this.mode = 'new',
    this.to = const [],
    this.cc = const [],
    this.bcc = const [],
    this.subject = '',
    this.atts = const [],
    this.text,
    this.flags,
    this.blocks,
    this.quote,
  });
}

class MailStore extends ChangeNotifier {
  final SharedPreferences prefs;

  /// Données réelles du moteur (`null` : données fictives de démonstration).
  final MailSource? source;
  StreamSubscription<String>? _changeSub;

  /// Historique des destinataires et noms choisis.
  late final ContactBook contactBook;
  MailStore(this.prefs, {bool watchWallpaper = false, this.source}) {
    contactBook = ContactBook.decode(
      prefs.getString('contactNames'),
      prefs.getString('contactHistory'),
    );
    followWallpaper = prefs.getString('follow') != '0';
    lang = AppLang.fromCode(prefs.getString('lang'));
    appLang = lang;
    if (watchWallpaper && wallSupported) {
      _wallSub = watchWallSeed().listen((seed) {
        if (seed == null) {
          _wall = null;
        } else {
          _wall = palFromSeed(seed);
        }
        notifyListeners();
      });
    }
    imagePolicy = ImagePolicy.decode(prefs.getString('imgPolicy'));
    theme = prefs.getString('theme') == 'light' ? 'light' : 'dark';
    accentHex = prefs.getString('accent');
    dense = prefs.getString('dense') == '1';
    live = prefs.getString('live') != '0';
    sig = prefs.getString('sig') ?? 'Cordialement,\nNoah';
    final extra = <Account>[];
    try {
      final raw = prefs.getString('accounts');
      if (raw != null) {
        for (final j in jsonDecode(raw) as List) {
          extra.add(Account.fromJson(j as Map<String, dynamic>));
        }
      }
    } catch (_) {
      /* préférences illisibles : on ignore */
    }
    final real = source?.accounts ?? [...defaultAccounts, ...extra];
    accounts = real.isEmpty
        ? [const Account('none', 'Aucun compte', '')]
        : real;
    knownAccounts = accounts;
    if (source != null) {
      _applyIdentities();
      _boxes.addAll(source!.boxes);
      _loadSnoozes();
      for (final l in _boxes.values) {
        _applySnoozes(l);
      }
      _changeSub = source!.changes.listen(_reloadAccount);
    }
    final saved = prefs.getString('account');
    unified = saved == 'all';
    activeId = accounts.any((a) => a.id == saved) ? saved! : accounts.first.id;
    if (unified) {
      mails = _mergedBoxes();
      _useIdentity(active);
    } else {
      mails = source != null ? (_boxes[active.id] ?? []) : seedAccount(active);
    }
    chat = [_welcome()];
    if (source != null) {
      wakeSnoozed();
      _armWake();
    }
  }

  // ---- synchronisation ----
  bool syncing = false;
  DateTime? _lastRefresh;

  /// Demande une synchronisation immédiate et relit la boîte active ; `quiet` : sans message, au plus toutes les 30 s.
  Future<void> refreshNow({bool quiet = false}) async {
    final src = source;
    final now = DateTime.now();
    if (src == null || syncing) return;
    final last = _lastRefresh;
    if (quiet && last != null && now.difference(last).inSeconds < 30) return;
    _lastRefresh = now;
    syncing = true;
    notifyListeners();
    try {
      unawaited(_reloadAccount(activeId));
      await src.syncNow();
      if (!quiet) toast('Synchronisation lancée');
    } catch (_) {
      if (!quiet) toast('Synchronisation impossible', err: true);
    } finally {
      await Future<void>.delayed(const Duration(milliseconds: 1200));
      syncing = false;
      notifyListeners();
    }
  }

  // ---- images distantes ----
  late ImagePolicy imagePolicy;

  /// Décision pour les images distantes d'un expéditeur (règles, puis réglage général).
  ImageAction imageActionFor(String email) => imagePolicy.decide(email);

  void _saveImagePolicy(ImagePolicy p) {
    imagePolicy = p;
    prefs.setString('imgPolicy', p.encode());
    notifyListeners();
  }

  void setImageDefault(ImageAction a) =>
      _saveImagePolicy(imagePolicy.withFallback(a));

  /// Ajoute (ou remplace) une règle ; `false` si la saisie n'est ni une adresse ni un domaine.
  bool addImageRule(String input, ImageAction action) {
    final m = normalizeMatch(input);
    if (m == null) return false;
    _saveImagePolicy(imagePolicy.withRule(ImageRule(m, action)));
    return true;
  }

  void removeImageRule(String match) =>
      _saveImagePolicy(imagePolicy.withoutRule(match));

  // ---- état ----
  late List<Mail> mails;
  late List<Account> accounts;
  late String activeId;

  /// « Toutes les boîtes » : les messages de tous les comptes dans une seule vue.
  bool unified = false;
  final Map<String, List<Mail>> _boxes = {};
  Account get active => accounts.firstWhere((a) => a.id == activeId);
  String folder = 'inbox';
  String? label;
  String pill = 'all';
  String q = '';
  String sort = 'new';
  final Set<String> sel = <String>{};
  String? open;
  PaneView view = PaneView.empty;
  late String theme;

  /// Langue de l'interface.
  late AppLang lang;

  /// Suivre les couleurs du fond d'écran (ShellArie) quand elles sont disponibles.
  late bool followWallpaper;
  StreamSubscription<WallSeed?>? _wallSub;
  ({Pal pal, bool dark})? _wall;

  /// Palette issue du fond d'écran, ou `null` si désactivée / indisponible.
  Pal? get wallPal => followWallpaper ? _wall?.pal : null;
  bool get wallDark => _wall?.dark ?? true;
  bool get wallAvailable => _wall != null;

  @visibleForTesting
  void debugSetWallSeed(WallSeed? seed) {
    _wall = seed == null ? null : palFromSeed(seed);
    notifyListeners();
  }

  /// Thème sombre effectif (celui du fond d'écran quand il est suivi).
  bool get isDark => wallPal != null ? wallDark : theme != 'light';

  /// Couleur principale choisie (hex `rrggbb`), ou `null` pour l'orange d'origine.
  String? accentHex;
  Color? get accentColor {
    final h = accentHex;
    if (h == null || !RegExp(r'^[0-9a-fA-F]{6}$').hasMatch(h)) return null;
    return Color(0xFF000000 | int.parse(h, radix: 16));
  }

  late bool dense, live;
  late String sig;
  late List<ChatMsg> chat;
  bool busy = false;
  ComposeState? c;
  String? freshId;
  bool navOpen = false;
  bool showHelp = false;
  bool narrow = false;
  final TextEditingController searchCtl = TextEditingController();
  final FocusNode searchFocus = FocusNode();
  final List<ToastItem> toasts = [];
  final List<Timer> _timers = [];
  int _toastId = 0;
  int incIdx = 0;
  int _scrollTick = 0;
  int get scrollTick => _scrollTick;

  static ChatMsg _welcome() => ChatMsg(
    role: 'ai',
    done: true,
    text: t(
      'Salut Noah ! Je peux **résumer** un mail, **rédiger** une réponse, **trier** ce qui est urgent ou **retrouver** un message. Sélectionne un mail dans la liste pour que je m’en serve comme contexte.',
    ),
  );

  void refresh() => notifyListeners();

  Timer _after(int ms, VoidCallback f) {
    late Timer t;
    t = Timer(Duration(milliseconds: ms), () {
      _timers.remove(t);
      f();
    });
    _timers.add(t);
    return t;
  }

  @override
  void dispose() {
    _changeSub?.cancel();
    _searchTimer?.cancel();
    _wake?.cancel();
    _wallSub?.cancel();
    for (final t in _timers) {
      t.cancel();
    }
    c?.timer?.cancel();
    super.dispose();
  }

  // ---- sélecteurs ----
  Mail? byId(String? id) {
    if (id == null) return null;
    for (final m in mails) {
      if (m.id == id) return m;
    }
    for (final m in _hits) {
      if (m.id == id) return m;
    }
    return null;
  }

  // ---- messages plus anciens ----
  bool loadingMore = false;

  /// Comptes de la vue courante ayant des messages plus anciens non chargés.
  List<String> _deeper() {
    final src = source;
    if (src == null) return const [];
    return [
      for (final a in accounts)
        if ((unified || a.id == activeId) && src.hasMore(a.id)) a.id,
    ];
  }

  bool get moreAvailable => _deeper().isNotEmpty;

  /// Charge une page de plus par boîte pour les comptes de la vue.
  Future<void> loadMore() async {
    final src = source;
    final ids = _deeper();
    if (src == null || ids.isEmpty || loadingMore) return;
    loadingMore = true;
    notifyListeners();
    try {
      for (final id in ids) {
        src.readDeeper(id);
        await _reloadOnce(id);
      }
    } finally {
      loadingMore = false;
      notifyListeners();
    }
  }

  // ---- recherche dans l'index du moteur ----
  List<Mail> _hits = const [];
  String _hitsFor = '';
  Timer? _searchTimer;

  /// Recherche sur le serveur en cours, et son bilan une fois finie (`null` : pas lancée pour cette requête).
  bool serverSearching = false;
  String? serverNote;

  /// Étend la recherche au serveur : trouve aussi des messages dont le corps n'est pas téléchargé.
  Future<void> searchServer() async {
    final src = source;
    final asked = q;
    if (src == null || asked.trim().length < 2 || serverSearching) return;
    serverSearching = true;
    serverNote = null;
    notifyListeners();
    try {
      final r = await src.searchServer(asked.trim());
      if (q != asked) return;
      final have = {for (final m in _hits) m.id};
      final loaded = {
        for (final l in _boxes.values)
          for (final m in l) m.id: m,
      };
      _hits = [
        ..._hits,
        for (final m in r.found)
          if (!have.contains(m.id)) loaded[m.id] ?? m,
      ];
      _hitsFor = asked;
      serverNote = _serverNoteOf(r);
    } catch (_) {
      if (q == asked) serverNote = t('Recherche sur le serveur impossible.');
    } finally {
      serverSearching = false;
      notifyListeners();
    }
  }

  String _serverNoteOf(ServerSearch r) {
    final parts = [
      if (r.unsynced > 0)
        t('{0} message(s) du serveur ne sont pas synchronisés.', [r.unsynced]),
      if (r.truncated) t('Résultats limités aux plus récents.'),
    ];
    return parts.isEmpty
        ? t('Recherche sur le serveur terminée.')
        : parts.join(' ');
  }

  /// Lance la recherche du moteur après une courte pause de frappe ; l'affichage local sert en attendant.
  void _scheduleSearch(String v) {
    _searchTimer?.cancel();
    final src = source;
    if (src == null || v.trim().length < 2) {
      _hits = const [];
      _hitsFor = '';
      return;
    }
    _searchTimer = Timer(const Duration(milliseconds: 250), () async {
      try {
        final found = await src.search(v.trim());
        if (q != v) return;
        final loaded = {
          for (final l in _boxes.values)
            for (final m in l) m.id: m,
        };
        _hits = [for (final m in found) loaded[m.id] ?? m];
        _hitsFor = v;
        notifyListeners();
      } catch (_) {
        /* recherche indisponible : le filtre local continue */
      }
    });
  }

  bool hidden(Mail m) => m.folder == 'trash' || m.folder == 'spam';
  int nonRead(Iterable<Mail> l) => l.where((m) => m.unread).length;

  List<Mail> scope() {
    if (label != null) {
      return mails
          .where((m) => m.labels.contains(label) && !hidden(m))
          .toList();
    }
    if (folder == 'starred') {
      return mails.where((m) => m.starred && !hidden(m)).toList();
    }
    return mails.where((m) => m.folder == folder).toList();
  }

  bool _matchesLocally(Mail m, String qq) =>
      ('${m.subject} ${m.from.name} ${m.from.email} ${plainMsgs(m.msgs)}')
          .toLowerCase()
          .contains(qq);

  /// Résultats de recherche : ceux du moteur (tout l'index) puis ceux déjà chargés, hors corbeille et spam.
  List<Mail> _searchResults(String qq) {
    final seen = <String>{};
    final inScope = [
      if (_hitsFor == q) ..._hits,
      ...mails.where((m) => _matchesLocally(m, qq)),
    ];
    return [
      for (final m in inScope)
        if ((!hidden(m) || m.folder == folder) &&
            (unified || m.account == activeId) &&
            seen.add(m.id))
          m,
    ];
  }

  List<Mail> visible() {
    final qq = q.trim().toLowerCase();
    var l = qq.isEmpty ? scope() : _searchResults(qq);
    if (pill != 'all') l = l.where((m) => m.flags.contains(pill)).toList();
    l.sort(switch (sort) {
      'old' => (Mail a, Mail b) => a.date - b.date,
      'unread' => (Mail a, Mail b) {
        final u = (b.unread ? 1 : 0) - (a.unread ? 1 : 0);
        return u != 0 ? u : b.date - a.date;
      },
      _ => (Mail a, Mail b) => b.date - a.date,
    });
    return l;
  }

  List<String> targets() {
    bool actionable(String id) => byId(id) != null && byId(id)!.outbox == null;
    if (sel.isNotEmpty) return sel.where(actionable).toList();
    if (open != null && actionable(open!)) return [open!];
    return [];
  }

  int _unreadIn(String folder) {
    final local = nonRead(mails.where((m) => m.folder == folder));
    final src = source;
    if (src == null) return local;
    var exact = 0;
    for (final a in accounts) {
      if (!unified && a.id != activeId) continue;
      exact += src.unreadOf(a.id, folder) ?? 0;
    }
    return exact > local ? exact : local;
  }

  int navCount(String id) {
    switch (id) {
      case 'inbox':
        return _unreadIn('inbox');
      case 'starred':
        return mails.where((m) => m.starred && !hidden(m)).length;
      case 'drafts':
        return mails.where((m) => m.folder == 'drafts').length;
      case 'outbox':
        return mails.where((m) => m.folder == 'outbox').length;
      case 'spam':
        return _unreadIn('spam');
      case 'snoozed':
        return mails.where((m) => m.folder == 'snoozed').length;
    }
    return 0;
  }

  int get needsCount => mails
      .where((m) => m.folder == 'inbox' && m.flags.contains('needs'))
      .length;

  // ---- toasts ----
  void toast(
    String msg, {
    String? label,
    VoidCallback? cb,
    bool err = false,
    int? ms,
  }) {
    final t = ToastItem(++_toastId, msg, label, cb, err);
    toasts.add(t);
    while (toasts.length > 3) {
      toasts.removeAt(0);
    }
    _after(ms ?? (label != null ? 6500 : 3200), () => killToast(t));
    notifyListeners();
  }

  void killToast(ToastItem t) {
    if (t.out || !toasts.contains(t)) return;
    t.out = true;
    notifyListeners();
    _after(260, () {
      toasts.remove(t);
      notifyListeners();
    });
  }

  void hint() => toast('Sélectionne d’abord un message', err: true);

  // ---- préférences ----
  void setSetting(String k, String v) {
    switch (k) {
      case 'theme':
        theme = v;
        prefs.setString('theme', v);
      case 'dense':
        dense = v == '1';
        prefs.setString('dense', v);
      case 'follow':
        followWallpaper = v == '1';
        prefs.setString('follow', v);
      case 'poll':
        prefs.setString('poll', v);
      case 'live':
        live = v == '1';
        prefs.setString('live', v);
        if (live) scheduleIncoming(rand(8000, 14000).round());
      case 'reset':
        resetDemo();
        return;
    }
    notifyListeners();
  }

  /// Change la couleur principale (`null` : retour à l'orange d'origine).
  void setAccent(String? hex) {
    accentHex = hex?.toLowerCase();
    if (accentHex == null) {
      prefs.remove('accent');
    } else {
      prefs.setString('accent', accentHex!);
    }
    notifyListeners();
  }

  void setLang(AppLang l) {
    if (l == lang) return;
    lang = l;
    appLang = l;
    prefs.setString('lang', l.code);
    notifyListeners();
  }

  void setSig(String s) {
    sig = s;
    prefs.setString('sig', s);
  }

  void resetDemo() {
    for (final t in _timers) {
      t.cancel();
    }
    _timers.clear();
    toasts.clear();
    _boxes.clear();
    if (unified) {
      mails = _mergedBoxes();
    } else {
      mails = seedAccount(active);
    }
    _useIdentity(active);
    folder = 'inbox';
    label = null;
    pill = 'all';
    q = '';
    searchCtl.clear();
    sort = 'new';
    sel.clear();
    open = null;
    view = PaneView.empty;
    c = null;
    chat = [_welcome()];
    busy = false;
    incIdx = 0;
    freshId = null;
    start();
  }

  /// Démarrage (après le premier build, `narrow` est connu).
  void start() {
    if (source != null && source!.accounts.isEmpty) {
      toast(
        'Aucun compte : ajoutes-en un avec « mira account add »',
        err: true,
      );
    }
    if (!narrow) {
      final l = visible();
      if (l.isNotEmpty) openMail(l.first.id);
    }
    notifyListeners();
    unawaited(_prefetchTop());
    if (source == null) scheduleIncoming(rand(15000, 22000).round());
  }

  // ---- comptes ----
  void _useIdentity(Account a) {
    me = Person(a.name, a.email);
    currentAccountId = a.id;
  }

  /// Crée (sans changer l'identité active) la boîte fictive d'un compte.
  List<Mail> _newBox(Account a) {
    if (source != null) return _boxes[a.id] ?? [];
    final keepMe = me, keepId = currentAccountId;
    final l = seedAccount(a);
    me = keepMe;
    currentAccountId = keepId;
    return l;
  }

  /// Range les messages affichés dans leurs boîtes respectives.
  void _stash() {
    if (unified) {
      for (final a in accounts) {
        final l = mails.where((m) => m.account == a.id).toList();
        if (l.isNotEmpty || _boxes.containsKey(a.id)) _boxes[a.id] = l;
      }
    } else {
      _boxes[activeId] = mails;
    }
  }

  List<Mail> _mergedBoxes() {
    for (final a in accounts) {
      _boxes.putIfAbsent(a.id, () => _newBox(a));
    }
    return [for (final a in accounts) ...?_boxes[a.id]];
  }

  void _resetView() {
    folder = 'inbox';
    label = null;
    pill = 'all';
    q = '';
    searchCtl.clear();
    sel.clear();
    open = null;
    view = PaneView.empty;
    navOpen = false;
    chat = [_welcome()];
    busy = false;
  }

  void switchAccount(String id) {
    if (!accounts.any((a) => a.id == id) || (!unified && id == activeId)) {
      return;
    }
    if (c != null) closeCompose(true);
    _stash();
    unified = false;
    activeId = id;
    mails = _boxes[id] ?? _newBoxFor(active);
    _useIdentity(active);
    prefs.setString('account', id);
    _resetView();
    notifyListeners();
    toast(t('Compte actif : {0}', [active.email]));
  }

  List<Mail> _newBoxFor(Account a) => _boxes[a.id] = _newBox(a);

  /// Affiche la boîte unifiée (tous les comptes).
  void showAllAccounts() {
    if (unified) return;
    if (c != null) closeCompose(true);
    _stash();
    unified = true;
    mails = _mergedBoxes();
    prefs.setString('account', 'all');
    _resetView();
    notifyListeners();
    toast('Toutes les boîtes');
  }

  /// Adresse d'expédition d'un compte (par défaut : le compte actif).
  Person senderFor(String email) {
    final e = email.toLowerCase();
    final id = knownSenders
        .where((x) => x.email.toLowerCase() == e)
        .firstOrNull;
    if (id != null) return Person(id.name, id.email);
    final a = accountByEmail(email);
    return a == null ? me : Person(a.name, a.email);
  }

  // ---- identités d'envoi ----
  void _applyIdentities() {
    final src = source;
    if (src == null) return;
    knownSenders = [
      for (final e in src.identities.entries)
        for (final i in e.value)
          Sender(
            i.address,
            i.displayName,
            e.key,
            signature: i.signature,
            isDefault: i.isDefault,
          ),
    ];
  }

  /// Relit les identités du moteur (après ajout, modification ou changement par défaut).
  Future<void> identitiesChanged() async {
    final src = source;
    if (src == null) return;
    await src.refreshIdentities();
    _applyIdentities();
    notifyListeners();
  }

  /// Messages déjà chargés d'un compte.
  List<Mail> mailsOf(String accountId) => _boxes[accountId] ?? const [];

  /// Adresses d'envoi proposées dans le composeur : identités, puis comptes sans identité ; compte actif d'abord.
  List<Sender> senderChoices() {
    final out = [...knownSenders];
    for (final a in accounts) {
      if (a.email.isNotEmpty &&
          !out.any((x) => x.email.toLowerCase() == a.email.toLowerCase())) {
        out.add(Sender(a.email, a.name, a.id));
      }
    }
    out.sort((x, y) {
      final ax = x.accountId == activeId ? 0 : 1;
      final ay = y.accountId == activeId ? 0 : 1;
      if (ax != ay) return ax.compareTo(ay);
      return (y.isDefault ? 1 : 0).compareTo(x.isDefault ? 1 : 0);
    });
    return out;
  }

  /// Adresse d'envoi pour écrire ou répondre : celle à laquelle le message a été reçu, sinon l'identité par défaut du compte.
  String fromFor([Mail? original]) {
    final accId = original?.account ?? activeId;
    final mine = senderChoices().where((x) => x.accountId == accId).toList();
    if (original != null) {
      final got = {
        for (final p in [...original.to, ...original.cc]) p.email.toLowerCase(),
      };
      final hit = mine.where((x) => got.contains(x.email.toLowerCase()));
      if (hit.isNotEmpty) return hit.first.email;
    }
    final def = mine.where((x) => x.isDefault).firstOrNull ?? mine.firstOrNull;
    return def?.email ?? accountById(accId)?.email ?? me.email;
  }

  /// Signature à utiliser pour cette adresse : celle de l'identité, sinon la signature générale.
  String signatureFor(String email) {
    final e = email.toLowerCase();
    final own = knownSenders
        .where((x) => x.email.toLowerCase() == e)
        .firstOrNull
        ?.signature;
    return own == null || own.trim().isEmpty ? sig : own;
  }

  /// Ajoute un compte (fictif) et bascule dessus. Renvoie `false` si l'adresse est déjà connectée.
  bool addAccount(String name, String email) {
    final em = email.trim().toLowerCase();
    if (accounts.any((a) => a.email.toLowerCase() == em)) return false;
    final nm = name.trim().isEmpty ? em.split('@').first : name.trim();
    final a = Account('x${DateTime.now().microsecondsSinceEpoch}', nm, em);
    accounts = [...accounts, a];
    knownAccounts = accounts;
    final custom = accounts
        .where((x) => !defaultAccounts.any((d) => d.id == x.id))
        .map((x) => x.toJson())
        .toList();
    prefs.setString('accounts', jsonEncode(custom));
    switchAccount(a.id);
    return true;
  }

  // ---- navigation ----
  void toggleTheme() {
    theme = theme == 'light' ? 'dark' : 'light';
    prefs.setString('theme', theme);
    notifyListeners();
  }

  void selectFolder(String id) {
    navOpen = false;
    folder = id;
    label = null;
    _afterNav();
  }

  void selectLabel(String k) {
    navOpen = false;
    label = k;
    folder = 'inbox';
    _afterNav();
  }

  void _afterNav() {
    pill = 'all';
    sel.clear();
    if (view != PaneView.compose && !visible().any((m) => m.id == open)) {
      open = null;
      view = PaneView.empty;
    }
    notifyListeners();
  }

  void setPill(String p) {
    pill = p;
    notifyListeners();
  }

  void setSort(String s) {
    sort = s;
    notifyListeners();
  }

  void setQuery(String v) {
    q = v;
    serverNote = null;
    _scheduleSearch(v);
    notifyListeners();
  }

  void clearQuery() {
    q = '';
    searchCtl.clear();
    notifyListeners();
    searchFocus.requestFocus();
  }

  void toggleNav() {
    navOpen = !navOpen;
    notifyListeners();
  }

  void closeNav() {
    if (!navOpen) return;
    navOpen = false;
    notifyListeners();
  }

  void openHelp() {
    showHelp = true;
    notifyListeners();
  }

  void closeHelp() {
    showHelp = false;
    notifyListeners();
  }

  void openMail(String id) {
    final m = byId(id);
    if (m == null) return;
    if (c != null) closeCompose(true);
    open = id;
    if (m.folder == 'drafts') {
      final d = m.draft;
      List<Block> bl = m.msgs[0].body;
      List<Block>? quote;
      if (bl.isNotEmpty && bl.last is Quote) {
        quote = [bl.last];
        bl = bl.sublist(0, bl.length - 1);
      }
      openCompose(
        ComposeInit(
          id: m.id,
          mode: 'draft',
          to: m.to,
          cc: m.cc,
          subject: m.subject == '(sans objet)' ? '' : m.subject,
          blocks: d?.rawText == null ? bl : null,
          text: d?.rawText,
          flags: d?.rawFlags,
          quote: quote,
          atts: m.msgs[0].att,
          from: d?.from,
          bcc: d?.bcc ?? const [],
          replyTo: d?.replyTo,
        ),
      );
      return;
    }
    if (m.unread) {
      m.unread = false;
      _syncAction([m], (a) => a.read(true));
    }
    _hydrate(m);
    _prefetchAround(m);
    if (view == PaneView.assistant) {
      _scrollTick++;
      notifyListeners();
      return;
    }
    view = PaneView.read;
    _scrollTick++;
    notifyListeners();
  }

  /// Charge d'avance, une par une, les premières conversations de la liste (le réseau n'attend pas l'ouverture).
  Future<void> _prefetchTop([int count = 8]) async {
    final src = source;
    if (src == null) return;
    for (final m in visible().take(count).toList()) {
      try {
        await src.hydrate(m);
      } catch (_) {
        continue;
      }
    }
  }

  /// Charge d'avance les deux conversations voisines pour que j/k soient instantanés.
  void _prefetchAround(Mail m) {
    final src = source;
    if (src == null) return;
    final l = visible();
    final i = l.indexWhere((x) => x.id == m.id);
    for (final j in [i + 1, i - 1]) {
      if (i < 0 || j < 0 || j >= l.length) continue;
      src.hydrate(l[j]).catchError((_) => false);
    }
  }

  /// Charge corps et pièces jointes depuis le moteur, puis redessine.
  Future<void> _hydrate(Mail m) async {
    final src = source;
    if (src == null) return;
    try {
      if (await src.hydrate(m, onProgress: notifyListeners)) {
        notifyListeners();
      }
    } catch (_) {
      /* corps indisponible : l'aperçu reste affiché */
    }
  }

  /// Affiche les images distantes d'un message (ou de son expéditeur), après accord de l'utilisateur.
  Future<void> allowRemote(Msg msg, {bool sender = false}) async {
    if (sender) {
      imagePolicy = imagePolicy.withRule(
        ImageRule(msg.from.email.toLowerCase(), ImageAction.always),
      );
      prefs.setString('imgPolicy', imagePolicy.encode());
    }
    try {
      if (await source?.allowRemote(msg, sender: sender) ?? false) {
        notifyListeners();
      }
    } catch (_) {
      toast('Impossible d’autoriser le contenu distant', err: true);
    }
  }

  /// Relit la boîte d'un compte après un événement du moteur.
  Future<void> _reloadAccount(String id) async {
    if (!_reloading.add(id)) {
      _reloadAgain.add(id);
      return;
    }
    try {
      await _reloadOnce(id);
    } finally {
      _reloading.remove(id);
      if (_reloadAgain.remove(id)) _reloadAccount(id);
    }
  }

  final Set<String> _reloading = {}, _reloadAgain = {};

  Future<void> _reloadOnce(String id) async {
    final src = source;
    final acc = accounts.where((a) => a.id == id).firstOrNull;
    if (src == null || acc == null) return;
    try {
      final fresh = _withOutgoing(id, await src.reload(acc));
      _applySnoozes(fresh);
      _keepLocalState(id, fresh);
      if (_signature(fresh) == _signature(_boxes[id] ?? const [])) return;
      _boxes[id] = fresh;
      _rememberContacts();
      if (unified) {
        mails = _mergedBoxes();
      } else if (id == activeId) {
        mails = fresh;
      }
      notifyListeners();
    } catch (_) {
      /* lecture impossible : on garde l'affichage actuel */
    }
  }

  /// Empreinte de ce que la liste affiche : identique avant et après = rien à redessiner.
  int _signature(List<Mail> l) => Object.hashAll([
    for (final m in l)
      Object.hash(
        m.id,
        m.date,
        m.unread,
        m.starred,
        m.folder,
        m.subject,
        m.msgs.length,
        Object.hashAll(m.labels),
      ),
  ]);

  /// Conserve corps, HTML, images et état « lu » déjà connus des messages relus.
  void _keepLocalState(String id, List<Mail> fresh) {
    final old = {for (final m in _boxes[id] ?? <Mail>[]) m.id: m};
    for (final m in fresh) {
      final prev = old[m.id];
      if (prev == null) continue;
      if (!prev.unread) m.unread = false;
      final byRef = {for (final x in prev.msgs) x.ref: x};
      for (final x in m.msgs) {
        final p = byRef[x.ref];
        if (p != null && p.hydrated) {
          x.body = p.body;
          x.att = p.att;
          x.html = p.html;
          x.blocked = p.blocked;
          x.inline = p.inline;
          x.hydrated = true;
          x.loaded = p.loaded;
        }
      }
    }
  }

  void closePane() {
    if (c != null) {
      closeCompose(true);
      return;
    }
    if (view == PaneView.read || view == PaneView.empty) open = null;
    view =
        (view == PaneView.assistant ||
            view == PaneView.settings ||
            view == PaneView.addAccount)
        ? ((open != null && byId(open) != null && !narrow)
              ? PaneView.read
              : PaneView.empty)
        : PaneView.empty;
    notifyListeners();
  }

  void showView(PaneView v) {
    if (c != null) closeCompose(true);
    view = v;
    notifyListeners();
  }

  void toggleAssistant() =>
      view == PaneView.assistant ? closePane() : showView(PaneView.assistant);

  /// Demande Réglages › Comptes avec l'onglet d'ajout ouvert ; consommé par l'écran des comptes.
  bool wantsAddAccount = false;

  /// Ouvre l'ajout de compte : onglet « Ajouter » des réglages des comptes (moteur réel).
  void openAddAccount() {
    wantsAddAccount = true;
    showView(PaneView.settings);
  }

  /// Prend en compte un compte que le moteur vient d'ajouter : liste relue, bascule dessus.
  Future<void> accountAdded(String id) async {
    final src = source;
    if (src == null) return;
    await src.refreshAccounts();
    accounts = src.accounts.isEmpty ? accounts : src.accounts;
    knownAccounts = accounts;
    for (final a in accounts) {
      _boxes.putIfAbsent(a.id, () => src.boxes[a.id] ?? []);
    }
    switchAccount(id);
  }

  /// Reprend la liste des comptes après une modification (nom, suppression…) sans quitter les réglages.
  Future<void> accountsChanged() async {
    final src = source;
    if (src == null) return;
    await src.refreshAccounts();
    final keepView = view;
    final ids = {for (final a in src.accounts) a.id};
    _boxes.removeWhere((id, _) => !ids.contains(id));
    accounts = src.accounts.isEmpty
        ? [const Account('none', 'Aucun compte', '')]
        : src.accounts;
    knownAccounts = accounts;
    for (final a in accounts) {
      _boxes.putIfAbsent(a.id, () => src.boxes[a.id] ?? []);
    }
    if (!accounts.any((a) => a.id == activeId)) {
      unified = false;
      activeId = accounts.first.id;
      mails = _boxes[activeId] ?? [];
      _resetView();
      view = keepView;
    } else if (unified) {
      mails = _mergedBoxes();
    }
    _useIdentity(active);
    prefs.setString('account', unified ? 'all' : activeId);
    notifyListeners();
  }

  void toggleSettings() =>
      view == PaneView.settings ? closePane() : showView(PaneView.settings);

  void reveal(Mail m) {
    q = '';
    searchCtl.clear();
    pill = 'all';
    if (!visible().contains(m) || !scope().contains(m)) {
      label = null;
      folder = m.folder;
    }
    notifyListeners();
  }

  // ---- actions sur les messages ----
  void _afterChange(int prevOpenIdx) {
    sel.removeWhere((id) => byId(id) == null);
    if (open != null && !visible().any((m) => m.id == open)) {
      final l = visible();
      final n = l.isEmpty ? null : l[prevOpenIdx.clamp(0, l.length - 1)];
      final free = view == PaneView.read || view == PaneView.empty;
      if (n != null && !narrow && free) {
        openMail(n.id);
        return;
      }
      if (free) {
        open = null;
        view = PaneView.empty;
      } else if (view != PaneView.compose) {
        open = null;
      }
    }
    notifyListeners();
  }

  void moveTo(List<String> ids, String to, String verb, [String? verbPl]) {
    final idx = visible().indexWhere((m) => m.id == open);
    final prev = ids.map((id) {
      final m = byId(id)!;
      return (m, m.folder, m.snooze);
    }).toList();
    for (final (m, _, _) in prev) {
      m.folder = to;
      m.snooze = null;
      _snoozed.remove(m.id);
    }
    _saveSnoozes();
    sel.clear();
    _afterChange(idx);
    _pushMoves([for (final (m, f, _) in prev) (m, f, to)]);
    toast(
      '${tn(ids.length, 'message', 'messages')} ${t(ids.length > 1 ? (verbPl ?? verb) : verb)}',
      label: t('Annuler'),
      cb: () {
        for (final (m, f, s) in prev) {
          m.folder = f;
          if (s != null) m.snooze = s;
        }
        notifyListeners();
        _pushMoves([for (final (m, f, _) in prev) (m, to, f)]);
      },
    );
  }

  void removeForever(List<String> ids) {
    final idx = visible().indexWhere((m) => m.id == open);
    final gone = ids.map(byId).whereType<Mail>().toList();
    mails = mails.where((m) => !ids.contains(m.id)).toList();
    sel.clear();
    _afterChange(idx);
    final pending = _after(6500, () {
      _syncAction(gone, (a) => a.deleteForever);
    });
    toast(
      '${tn(gone.length, 'message', 'messages')} ${t(gone.length > 1 ? 'supprimés définitivement' : 'supprimé définitivement')}',
      label: t('Annuler'),
      cb: () {
        pending.cancel();
        mails.addAll(gone);
        notifyListeners();
      },
    );
  }

  // ---- reports (côté app : le moteur n'en a pas) ----

  /// Messages reportés : identifiant → (instant du rappel, libellé affiché).
  final Map<String, (int, String)> _snoozed = {};

  void _loadSnoozes() {
    try {
      final raw = jsonDecode(prefs.getString('snoozes') ?? '{}') as Map;
      for (final e in raw.entries) {
        final v = e.value as List;
        _snoozed['${e.key}'] = ((v[0] as num).toInt(), '${v[1]}');
      }
    } catch (_) {
      /* reports illisibles : on repart de zéro */
    }
  }

  Timer? _wake;

  /// Programme le réveil du prochain message reporté (un seul minuteur, rien s'il n'y en a pas).
  void _armWake() {
    _wake?.cancel();
    if (_snoozed.isEmpty) return;
    final next = _snoozed.values
        .map((v) => v.$1)
        .reduce((a, b) => a < b ? a : b);
    final wait = (next - nowMs()).clamp(0, 3600000) + 50;
    _wake = Timer(Duration(milliseconds: wait), wakeSnoozed);
  }

  void _saveSnoozes() {
    if (source == null) return;
    _armWake();
    prefs.setString(
      'snoozes',
      jsonEncode({
        for (final e in _snoozed.entries) e.key: [e.value.$1, e.value.$2],
      }),
    );
  }

  /// Remet en « Reportés » les messages relus dont le rappel n'est pas échu.
  void _applySnoozes(List<Mail> l) {
    final now = nowMs();
    for (final m in l) {
      final s = _snoozed[m.id];
      if (s == null || m.folder != 'inbox' || s.$1 <= now) continue;
      m.folder = 'snoozed';
      m.snooze = s.$2;
    }
  }

  /// Rappelle les messages reportés dont l'heure est venue : retour en réception, non lus.
  void wakeSnoozed() {
    final now = nowMs();
    final due = [
      for (final e in _snoozed.entries)
        if (e.value.$1 <= now) e.key,
    ];
    if (due.isEmpty) {
      _armWake();
      return;
    }
    Mail? first;
    for (final id in due) {
      _snoozed.remove(id);
      final m =
          byId(id) ??
          _boxes.values.expand((l) => l).where((x) => x.id == id).firstOrNull;
      if (m == null) continue;
      m.folder = 'inbox';
      m.unread = true;
      m.snooze = null;
      first ??= m;
    }
    _saveSnoozes();
    notifyListeners();
    final woken = first;
    if (woken != null) {
      toast(
        t('Rappel : {0}', [woken.subject]),
        label: t('Ouvrir'),
        cb: () {
          reveal(woken);
          openMail(woken.id);
        },
      );
    }
  }

  void snoozeTo(List<String> ids, String lbl, int ms) {
    final idx = visible().indexWhere((m) => m.id == open);
    final prev = ids.map((id) {
      final m = byId(id)!;
      return (m, m.folder);
    }).toList();
    for (final (m, _) in prev) {
      m.folder = 'snoozed';
      m.snooze = lbl;
    }
    if (source != null) {
      final until = ms > 0 ? nowMs() + ms : scheduledMs(lbl);
      for (final id in ids) {
        _snoozed[id] = (until, lbl);
      }
      _saveSnoozes();
    }
    sel.clear();
    _afterChange(idx);
    toast(
      t('Reporté · {0}', [t(lbl)]),
      label: t('Annuler'),
      cb: () {
        for (final (m, f) in prev) {
          m.folder = f;
          m.snooze = null;
          _snoozed.remove(m.id);
        }
        _saveSnoozes();
        notifyListeners();
      },
    );
    if (ms > 0 && source == null) {
      _after(ms, () {
        for (final id in ids) {
          final m = byId(id);
          if (m != null && m.folder == 'snoozed') {
            m.folder = 'inbox';
            m.unread = true;
            m.date = nowMs();
            m.snooze = null;
          }
        }
        notifyListeners();
        final first = byId(ids.first);
        toast(
          t('Rappel : {0}', [
            first != null ? first.subject : t('message reporté'),
          ]),
          label: t('Ouvrir'),
          cb: () {
            final m = byId(ids.first);
            if (m != null) {
              reveal(m);
              openMail(m.id);
            }
          },
        );
      });
    }
  }

  void snooze(String v) {
    final ids = targets();
    if (ids.isEmpty) return hint();
    switch (v) {
      case '1m':
        snoozeTo(ids, 'dans 1 minute (démo)', 60000);
      case '1h':
        snoozeTo(ids, 'dans 1 heure', 0);
      case 'tom':
        snoozeTo(ids, 'demain 08:00', 0);
      default:
        snoozeTo(ids, 'lundi 08:00', 0);
    }
  }

  void archive() {
    final ids = targets();
    if (ids.isEmpty) return hint();
    final back = ids.every((id) => byId(id)!.folder == 'archive');
    moveTo(
      ids,
      back ? 'inbox' : 'archive',
      back ? 'déplacé dans la réception' : 'archivé',
      back ? 'déplacés dans la réception' : 'archivés',
    );
  }

  void spam() {
    final ids = targets();
    if (ids.isEmpty) return hint();
    final back = ids.every((id) => byId(id)!.folder == 'spam');
    moveTo(
      ids,
      back ? 'inbox' : 'spam',
      back ? 'déplacé dans la réception' : 'signalé comme spam',
      back ? 'déplacés dans la réception' : 'signalés comme spam',
    );
  }

  void trash() {
    final ids = targets();
    if (ids.isEmpty) return hint();
    if (ids.every((id) => byId(id)!.folder == 'trash')) {
      return removeForever(ids);
    }
    moveTo(ids, 'trash', 'mis à la corbeille');
  }

  void toggleRead() {
    final ids = targets();
    if (ids.isEmpty) return hint();
    final ms = ids.map((i) => byId(i)!).toList();
    final allRead = ms.every((m) => !m.unread);
    for (final m in ms) {
      m.unread = allRead;
    }
    sel.clear();
    notifyListeners();
    _syncAction(ms, (a) => a.read(!allRead));
    toast(
      '${tn(ms.length, 'message', 'messages')} ${t(allRead ? (ms.length > 1 ? 'marqués comme non lus' : 'marqué comme non lu') : (ms.length > 1 ? 'marqués comme lus' : 'marqué comme lu'))}',
    );
  }

  /// Libellés connus (moteur, ou démonstration) ; ceux du compte actif en vue simple.
  Map<String, LabelDef> get labelDefs {
    final all = source?.labels ?? seedLabelDefs;
    if (source == null || unified) return all;
    return {
      for (final e in all.entries)
        if (e.value.account == activeId) e.key: e.value,
    };
  }

  /// Libellé d'identifiant donné (`null` : supprimé ou d'un autre compte).
  LabelDef? labelDef(String k) => (source?.labels ?? seedLabelDefs)[k];

  /// Nom affiché d'un libellé ; en vue unifiée, suivi du compte (« Factures · Perso »).
  String labelTitle(String k) {
    final def = labelDef(k);
    if (def == null) return '';
    final owner = accounts.where((a) => a.id == def.account).firstOrNull;
    if (!unified || accounts.length < 2 || owner == null) return t(def.n);
    return '${t(def.n)} · ${owner.name.isEmpty ? owner.email : owner.name}';
  }

  /// Libellés proposés pour la sélection : ceux des comptes des conversations choisies.
  Map<String, LabelDef> labelsForTargets() {
    final all = source?.labels ?? seedLabelDefs;
    if (source == null) return all;
    final accs = {for (final i in targets()) byId(i)?.account};
    return {
      for (final e in all.entries)
        if (accs.isEmpty || accs.contains(e.value.account)) e.key: e.value,
    };
  }

  void toggleLabel(String k) {
    final ids = targets();
    if (ids.isEmpty) return hint();
    final all0 = ids.map((i) => byId(i)!).toList();
    final def = labelDef(k);
    if (def == null) return;
    final admin = source?.labelAdmin;
    final ms = admin == null
        ? all0
        : all0.where((m) => m.account == def.account).toList();
    final skipped = all0.length - ms.length;
    if (ms.isEmpty) {
      toast('Libellé d’un autre compte : rien à modifier', err: true);
      return;
    }
    final all = ms.every((m) => m.labels.contains(k));
    for (final m in ms) {
      m.labels = all
          ? m.labels.where((x) => x != k).toList()
          : {...m.labels, k}.toList();
    }
    notifyListeners();
    final done = t(all ? 'Libellé « {0} » retiré' : 'Libellé « {0} » ajouté', [
      labelTitle(k),
    ]);
    toast(
      skipped == 0
          ? done
          : '$done · ${t('{0} d’un autre compte ignorée(s)', [skipped])}',
    );
    if (admin != null) _pushLabel(admin, ms, k, !all);
  }

  /// Envoie la pose du libellé au moteur ; en cas d'échec, rétablit l'état précédent.
  Future<void> _pushLabel(
    LabelAdmin admin,
    List<Mail> ms,
    String k,
    bool on,
  ) async {
    try {
      for (final m in ms) {
        await admin.setOn(m, k, on);
      }
    } catch (e) {
      for (final m in ms) {
        m.labels = on
            ? m.labels.where((x) => x != k).toList()
            : {...m.labels, k}.toList();
      }
      toast('Libellé non modifié', err: true);
      notifyListeners();
    }
  }

  /// Rattache une boîte du serveur à un dossier de l'interface (`null` : redevient un libellé).
  Future<void> mapFolder(String mailboxId, String? folder) async {
    final src = source;
    final account = src?.mappable[mailboxId]?.account;
    if (src == null || account == null) return;
    if (folder == null) {
      src.folderMap.remove(mailboxId);
    } else {
      src.folderMap[mailboxId] = folder;
    }
    await prefs.setString('folderMap', encodeFolderMap(src.folderMap));
    if (label == mailboxId) selectFolder('inbox');
    await _reloadAccount(account);
    notifyListeners();
  }

  /// Crée un libellé sur un compte du moteur ; rend `false` si le nom est refusé.
  Future<bool> createLabel(String account, String name) =>
      _labelOp(() => source!.labelAdmin!.create(account, name.trim()), account);

  /// Renomme un libellé du moteur.
  Future<bool> renameLabel(String key, String name) => _labelOp(
    () => source!.labelAdmin!.rename(key, name.trim()),
    labelDef(key)?.account,
  );

  /// Supprime un libellé du moteur ; si on le consultait, retour à la réception.
  Future<bool> deleteLabel(String key) async {
    final account = labelDef(key)?.account;
    final ok = await _labelOp(() => source!.labelAdmin!.delete(key), account);
    if (ok && label == key) selectFolder('inbox');
    return ok;
  }

  Future<bool> _labelOp(Future<void> Function() job, String? account) async {
    if (source?.labelAdmin == null || account == null) return false;
    try {
      await job();
    } catch (e) {
      toast(e is EngineException ? e.message : 'Libellé refusé', err: true);
      return false;
    }
    await _reloadAccount(account);
    notifyListeners();
    return true;
  }

  void toggleStar([String? id]) {
    final m = byId(id ?? open);
    if (m == null) return;
    m.starred = !m.starred;
    notifyListeners();
    _syncAction([m], (a) => a.flag(m.starred));
  }

  void toggleSel(String id) {
    sel.contains(id) ? sel.remove(id) : sel.add(id);
    notifyListeners();
  }

  void selAll() {
    final vis = visible();
    final all = vis.isNotEmpty && vis.every((m) => sel.contains(m.id));
    for (final m in vis) {
      all ? sel.remove(m.id) : sel.add(m.id);
    }
    notifyListeners();
  }

  void selNone() {
    sel.clear();
    notifyListeners();
  }

  void readAllVisible() {
    final l = visible();
    final unread = l.where((m) => m.unread).toList();
    for (final m in l) {
      m.unread = false;
    }
    notifyListeners();
    _syncAction(unread, (a) => a.read(true));
    toast(
      '${tn(l.length, 'message', 'messages')} ${t(l.length > 1 ? 'marqués comme lus' : 'marqué comme lu')}',
    );
  }

  final unsubscriber = Unsubscriber();

  /// Moyen de désabonnement de la conversation ouverte (`null` : l'expéditeur n'en annonce pas).
  Unsub? unsubFor(Mail m) => Unsubscriber.of(m.msgs);

  /// Se désabonne : en un clic si possible, sinon page web ou message de désabonnement prérempli.
  Future<void> unsubscribe(Mail m) async {
    final u = unsubFor(m);
    if (u == null) return;
    final url = u.url == null ? null : Uri.tryParse(u.url!);
    if (url != null && u.oneClick) {
      final ok = await unsubscriber.postOneClick(url);
      toast(
        ok ? t('Désabonnement envoyé') : t('Désabonnement impossible'),
        err: !ok,
      );
    } else if (url != null) {
      await unsubscriber.openPage(url);
    } else if (u.mailto != null) {
      _mailtoUnsubscribe(m, u.mailto!);
    }
  }

  void _mailtoUnsubscribe(Mail m, String mailto) {
    final uri = Uri.tryParse(
      mailto.startsWith('mailto:') ? mailto : 'mailto:$mailto',
    );
    if (uri == null || uri.path.isEmpty) return;
    openCompose(
      ComposeInit(
        from: fromFor(m),
        to: [Person('', uri.path)],
        subject: uri.queryParameters['subject'] ?? 'unsubscribe',
        text: uri.queryParameters['body'],
      ),
    );
  }

  /// Vide la corbeille ou le spam : le moteur vide chaque boîte entière, y compris ce qui n'est pas chargé.
  void emptyFolder() {
    final actions = source?.messageActions;
    final boxes = <String>[
      for (final a in accounts)
        if (unified || a.id == activeId) ?source?.mailboxOf(a.id, folder),
    ];
    if (actions == null || label != null || boxes.isEmpty) {
      return removeForever(scope().map((m) => m.id).toList());
    }
    final gone = mails.where((m) => m.folder == folder).toList();
    final ids = {for (final m in gone) m.id};
    final idx = visible().indexWhere((m) => m.id == open);
    mails = mails.where((m) => !ids.contains(m.id)).toList();
    sel.clear();
    _afterChange(idx);
    final pending = _after(6500, () => _emptyBoxes(actions, boxes));
    toast(
      t('Dossier vidé'),
      label: t('Annuler'),
      cb: () {
        pending.cancel();
        mails.addAll(gone);
        notifyListeners();
      },
    );
  }

  Future<void> _emptyBoxes(MessageActions actions, List<String> boxes) async {
    try {
      for (final b in boxes) {
        await actions.emptyMailbox(b);
      }
    } catch (_) {
      toast(t('Action non appliquée sur le serveur'), err: true);
      for (final a in accounts) {
        await _reloadOnce(a.id);
      }
    }
  }

  /// Enregistre un fichier à l'endroit choisi par l'utilisateur (remplaçable dans les tests).
  Future<Uri?> Function(String name, Uint8List bytes) saveFile = (
    name,
    bytes,
  ) => FilePicker.saveFile(fileName: name, bytes: bytes);

  /// Télécharge une pièce jointe : contenu lu dans le moteur, puis enregistré où l'utilisateur le choisit.
  Future<void> download(Attachment a) async {
    final src = source;
    final known = a.bytes != null || a.id != null;
    if (src == null || !known) {
      toast(t('Téléchargement de {0}…', [a.n]));
      _after(1200, () => toast(t('{0} téléchargé', [a.n])));
      return;
    }
    try {
      final bytes = a.bytes ?? await src.attachmentBytes(a);
      if (bytes == null) throw const EngineException('unavailable', '');
      final saved = await saveFile(a.n, bytes);
      if (saved != null) toast(t('{0} téléchargé', [a.n]));
    } catch (_) {
      toast(t('Téléchargement impossible'), err: true);
    }
  }

  void step(int d) {
    final l = visible();
    if (l.isEmpty) return;
    final i = l.indexWhere((m) => m.id == open);
    final n = l[(i + d).clamp(0, l.length - 1)];
    if (n.id != open) openMail(n.id);
  }

  // ---- rédaction ----
  String sigText() => sig;

  List<Block> quoteOf(Mail m) {
    final x = m.msgs.last;
    return [
      Quote([
        Para([
          Span(t('Le {0}, {1} a écrit :', [fullDate(x.date), x.from.name])),
          const Span('\n'),
        ], loose: true),
        ...x.body,
      ]),
    ];
  }

  void replyTo(String mode, {String intro = ''}) {
    final m = byId(open);
    if (m == null || m.folder == 'drafts') return hint();
    final last = m.msgs.last;
    if (mode == 'fwd') {
      final tmp = RichController();
      tmp.loadBlocks(last.body);
      final head =
          '\n${t('---------- Message transféré ----------')}\n${t('De : {0} <{1}>', [last.from.name, last.from.email])}\n${t('Objet : {0}', [m.subject])}\n\n';
      final sign = '\n\n${signatureFor(fromFor(m))}';
      openCompose(
        ComposeInit(
          from: fromFor(m),
          mode: mode,
          replyTo: m.id,
          to: const [],
          subject: 'Fwd: ${stripRe(m.subject)}',
          atts: last.att.toList(),
          text: sign + head + tmp.text,
          flags: [
            ...RichController.gapFlags(sign),
            ...RichController.gapFlags(head),
            ...tmp.flags,
          ],
        ),
      );
      return;
    }
    final other = isMine(last.from.email) ? m.to[0] : last.from;
    final to = [other];
    var cc = <Person>[];
    if (mode == 'all') {
      final everyone = [
        ...m.to,
        ...m.cc,
        last.from,
      ].where((p) => !isMine(p.email) && p.email != other.email).toList();
      final seen = <String>{};
      cc = everyone.where((p) => seen.add(p.email)).toList();
    }
    final txt =
        (intro.isEmpty ? '\n\n' : '$intro\n\n') + signatureFor(fromFor(m));
    openCompose(
      ComposeInit(
        from: fromFor(m),
        mode: mode,
        replyTo: m.id,
        to: to,
        cc: cc,
        subject: 'Re: ${stripRe(m.subject)}',
        text: txt,
        flags: RichController.gapFlags(txt),
        quote: quoteOf(m),
      ),
    );
  }

  void openCompose(ComposeInit i) {
    if (c != null) {
      saveDraft();
      _copyToServer(c!);
      c!.timer?.cancel();
    }
    final cs = ComposeState(
      id: i.id,
      replyTo: i.replyTo,
      mode: i.mode,
      to: [...i.to],
      cc: [...i.cc],
      bcc: [...i.bcc],
      atts: [...i.atts],
      from: i.from ?? fromFor(),
      quote: i.quote,
    );
    if (source?.composer != null && (i.id ?? '').startsWith('draft:')) {
      cs.engineId = i.id!.substring('draft:'.length);
      cs.engineAtts = [...i.atts];
    }
    cs.subject.text = i.subject;
    final blank = i.mode == 'new' && i.text == null && i.blocks == null;
    final text = blank ? '\n\n${signatureFor(cs.from)}' : i.text;
    if (i.blocks != null) {
      cs.body.loadBlocks(i.blocks!);
    } else {
      cs.body.setContent(
        text ?? '',
        blank
            ? RichController.gapFlags(text!)
            : i.flags ?? List<int>.filled((text ?? '').length, 0),
      );
    }
    if (i.mode != 'draft') {
      final sig = signatureFor(cs.from);
      if (sig.trim().isNotEmpty && cs.body.text.contains(sig)) cs.sigText = sig;
    }
    cs.body.addListener(() {});
    c = cs;
    view = PaneView.compose;
    if (cs.id != null) cs.saved = t('Brouillon enregistré');
    notifyListeners();
  }

  void touch() {
    final cs = c;
    if (cs == null) return;
    cs.dirty = true;
    cs.saved = t('Enregistrement…');
    cs.timer?.cancel();
    cs.timer = Timer(const Duration(milliseconds: 1200), () {
      if (identical(c, cs) && saveDraft()) {
        cs.saved = t('Brouillon enregistré · {0}', [hhmm(nowMs())]);
        notifyListeners();
      }
    });

    notifyListeners();
  }

  bool saveDraft() {
    final cs = c;
    if (cs == null || !cs.dirty) return false;
    final subject = cs.subject.text;
    final blocks = cs.body.toBlocks();
    final plainTxt =
        cs.sigText.isNotEmpty && cs.body.text.trim() == cs.sigText.trim()
        ? ''
        : cs.body.text.trim();
    final empty =
        !cs.fromChosen &&
        cs.to.isEmpty &&
        cs.cc.isEmpty &&
        subject.trim().isEmpty &&
        plainTxt.isEmpty &&
        cs.atts.isEmpty;
    var d = byId(cs.id);
    if (empty) {
      if (d != null) {
        mails = mails.where((x) => x != d).toList();
        cs.id = null;
        notifyListeners();
      }
      _dropEngineDraft(cs);
      return false;
    }
    if (d == null) {
      d = Mail(
        id: cs.engineId != null ? 'draft:${cs.engineId}' : 'd${nextUid()}',
        folder: 'drafts',
        from: senderFor(cs.from),
        to: [],
        subject: '',
        date: 0,
        msgs: [Msg(from: senderFor(cs.from), date: 0, body: [])],
      );
      mails.add(d);
      cs.id = d.id;
    }
    d
      ..account = accountByEmail(cs.from)?.id ?? d.account
      ..from = senderFor(cs.from)
      ..to = [...cs.to]
      ..cc = [...cs.cc]
      ..subject = subject.trim().isEmpty ? '(sans objet)' : subject.trim()
      ..date = nowMs()
      ..draft = DraftMeta(
        from: cs.from,
        bcc: [...cs.bcc],
        replyTo: cs.replyTo,
        mode: cs.mode,
        rawText: cs.body.text,
        rawFlags: List<int>.from(cs.body.flags),
      );
    d.msgs[0] = Msg(
      from: senderFor(cs.from),
      date: nowMs(),
      body: [...blocks, ...?cs.quote],
      att: [...cs.atts],
    );
    _queuePush(cs);
    notifyListeners();
    return true;
  }

  /// Change l'adresse d'envoi ; la signature insérée devient celle de la nouvelle adresse.
  void changeFrom(String email) {
    final cs = c;
    if (cs == null) return;
    cs.from = email;
    cs.fromChosen = true;
    final next = signatureFor(email);
    final old = cs.sigText;
    final at = old.isEmpty ? -1 : cs.body.text.lastIndexOf(old);
    if (at >= 0 && next != old) {
      final text = cs.body.text;
      final flags = cs.body.flags;
      final keep = cs.body.selection;
      final end = at + old.length;
      cs.body.setContent(text.replaceRange(at, end, next), [
        ...flags.take(at),
        ...List<int>.filled(next.length, 0),
        ...flags.skip(end),
      ]);
      final cursor = keep.baseOffset.clamp(0, cs.body.text.length);
      cs.body.selection = TextSelection.collapsed(offset: cursor);
      cs.sigText = next;
    }
    touch();
  }

  // ---- actions sur les messages, envoyées au moteur ----

  /// Applique l'action au moteur ; si elle échoue, relit les comptes touchés pour retrouver l'état réel.
  Future<void> _syncAction(
    List<Mail> ms,
    Map<String, dynamic> Function(MessageActions a) kind,
  ) async {
    final actions = source?.messageActions;
    if (actions == null || ms.isEmpty) return;
    try {
      await actions.run(ms, kind(actions));
    } catch (_) {
      toast(t('Action non appliquée sur le serveur'), err: true);
      for (final id in {for (final m in ms) m.account}) {
        await _reloadOnce(id);
      }
    }
  }

  /// Action du moteur pour passer un message d'un dossier à un autre (`null` : rien à faire côté serveur).
  Map<String, dynamic>? _moveKind(
    MessageActions a,
    Mail m,
    String from,
    String to,
  ) {
    if (to == 'archive') return a.archive;
    if (to == 'trash') return a.trash;
    if (to == 'spam') return a.spam(true);
    if (from == 'spam' && to == 'inbox') return a.spam(false);
    final box = source?.mailboxOf(m.account, to);
    return box == null ? null : a.move(box);
  }

  /// Répercute des changements de dossier au moteur, un appel par type d'action.
  void _pushMoves(List<(Mail, String, String)> moves) {
    final actions = source?.messageActions;
    if (actions == null) return;
    final groups = <String, (Map<String, dynamic>, List<Mail>)>{};
    for (final (m, from, to) in moves) {
      final kind = _moveKind(actions, m, from, to);
      if (kind == null) continue;
      groups.putIfAbsent('${m.account}|$kind', () => (kind, [])).$2.add(m);
    }
    for (final (kind, ms) in groups.values) {
      _syncAction(ms, (_) => kind);
    }
  }

  // ---- boîte d'envoi ----

  /// Exécute une opération sur un envoi de la boîte d'envoi, puis relit le compte.
  Future<void> _outboxOp(
    Mail m,
    Future<void> Function() op,
    String done,
  ) async {
    try {
      await op();
      toast(t(done));
    } catch (_) {
      toast(t('Action impossible sur cet envoi'), err: true);
    }
    await _reloadOnce(m.account);
    if (byId(m.id) == null) {
      open = null;
      view = PaneView.empty;
      notifyListeners();
    }
  }

  /// Annule un envoi en attente : le message redevient un brouillon.
  Future<void> cancelOutbox(Mail m) {
    final o = m.outbox;
    final admin = source?.composer;
    if (o == null || admin == null) return Future.value();
    return _outboxOp(m, () => admin.cancelSend(o.action), 'Envoi annulé');
  }

  /// Relance un envoi échoué.
  Future<void> retryOutbox(Mail m) {
    final o = m.outbox;
    final admin = source?.composer;
    if (o == null || admin == null) return Future.value();
    return _outboxOp(m, () => admin.retry(o.action), 'Nouvel essai lancé');
  }

  /// Envoi incertain : le renvoyer, ou le tenir pour parti.
  Future<void> resolveOutbox(Mail m, {required bool resend}) {
    final o = m.outbox;
    final admin = source?.composer;
    if (o == null || admin == null) return Future.value();
    return _outboxOp(
      m,
      () => admin.resolve(o.action, resend: resend),
      resend ? 'Message renvoyé' : 'Message tenu pour envoyé',
    );
  }

  // ---- contacts ----

  /// Messages envoyés connus, tous comptes confondus.
  Iterable<Mail> _sentMails() => [
    for (final l in source != null ? _boxes.values : [mails])
      for (final m in l)
        if (m.folder == 'sent') m,
  ];

  /// Met l'historique des destinataires à jour d'après les messages envoyés.
  void _rememberContacts() {
    if (contactBook.remember(_sentMails(), isMine)) {
      prefs.setString('contactHistory', contactBook.encodeHistory());
    }
  }

  /// Correspondants à montrer dans les réglages, après mise à jour de l'historique.
  List<ContactEntry> contactEntries() {
    _rememberContacts();
    return contactBook.entries();
  }

  /// Personnes proposées à la saisie d'un destinataire : l'historique, puis (démonstration) les contacts fictifs.
  List<Person> contactPeople() {
    final known = [for (final e in contactEntries()) e.person];
    if (source != null) return known;
    final have = {for (final p in known) p.email.toLowerCase()};
    return [
      ...known,
      for (final c in contacts)
        if (!have.contains(c.email.toLowerCase()))
          Person(contactBook.names[c.email.toLowerCase()] ?? c.name, c.email),
    ];
  }

  /// Nom à afficher pour une personne : celui choisi dans Contacts, sinon celui du message.
  String shown(Person p) => contactBook.names[p.email.toLowerCase()] ?? p.name;

  /// Donne un nom à une adresse ; un nom vide rend le nom vu dans les messages.
  void setContactName(String email, String name) {
    final key = email.toLowerCase();
    if (name.trim().isEmpty) {
      contactBook.names.remove(key);
    } else {
      contactBook.names[key] = name.trim();
    }
    prefs.setString('contactNames', contactBook.encodeNames());
    notifyListeners();
  }

  // ---- brouillons du moteur ----

  /// Met l'enregistrement au moteur à la suite des précédents (jamais deux en même temps).
  void _queuePush(ComposeState cs) {
    if (source?.composer == null) return;
    cs.push = cs.push.then((_) => _pushDraft(cs)).catchError((_) {
      cs.saved = t('Enregistrement impossible');
      notifyListeners();
    });
  }

  /// Identité du moteur pour une adresse d'envoi.
  String? _identityFor(String email, String account) => source
      ?.identities[account]
      ?.where((i) => i.address.toLowerCase() == email.toLowerCase())
      .firstOrNull
      ?.id;

  /// Crée le brouillon dans le moteur au premier enregistrement, puis y recopie le contenu.
  Future<void> _pushDraft(ComposeState cs) async {
    final admin = source!.composer!;
    final account = accountByEmail(cs.from)?.id ?? activeId;
    final identity = _identityFor(cs.from, account);
    if (cs.engineId == null) {
      final origin = byId(cs.replyTo)?.msgs.last.ref;
      final mode = origin == null
          ? 'new'
          : switch (cs.mode) {
              'all' => 'reply_all',
              'fwd' => 'forward',
              _ => 'reply',
            };
      final created = await admin.create(
        account,
        identity: identity,
        mode: mode,
        message: origin,
      );
      cs.engineId = created.id;
      _rekeyDraft(cs, 'draft:${created.id}');
    }
    final quote = cs.quote ?? const <Block>[];
    final blocks = [...cs.body.toBlocks(), ...quote];
    await admin.update(
      cs.engineId!,
      identity: identity,
      to: cs.to,
      cc: cs.cc,
      bcc: cs.bcc,
      subject: cs.subject.text,
      text: blocksToText(blocks),
      html: blocksToHtml(blocks),
    );
    await _syncAttachments(cs, admin);
    cs.copyStale = true;
  }

  /// En quittant la rédaction : le brouillon est copié dans « Brouillons » sur le serveur.
  void _copyToServer(ComposeState cs) {
    final admin = source?.composer;
    if (admin == null) return;
    cs.push = cs.push
        .then((_) async {
          final id = cs.engineId;
          if (id == null || !cs.copyStale) return;
          cs.copyStale = false;
          await admin.saveCopy(id);
        })
        .catchError((_) {
          cs.copyStale = true;
        });
  }

  /// Retire puis ajoute les pièces jointes pour que le moteur ait la même liste que l'écran.
  Future<void> _syncAttachments(ComposeState cs, ComposeAdmin admin) async {
    final wanted = cs.atts;
    for (var i = cs.engineAtts.length - 1; i >= 0; i--) {
      if (wanted.contains(cs.engineAtts[i])) continue;
      await admin.detach(cs.engineId!, i);
      cs.engineAtts.removeAt(i);
    }
    for (final a in wanted) {
      if (cs.engineAtts.contains(a) || a.path == null) continue;
      await admin.attach(cs.engineId!, a.path!);
      cs.engineAtts.add(a);
    }
  }

  /// Le brouillon local prend l'identifiant du moteur, pour que la relecture le retrouve.
  void _rekeyDraft(ComposeState cs, String id) {
    final d = byId(cs.id);
    if (d != null) {
      if (open == d.id) open = id;
      d.id = id;
    }
    cs.id = id;
  }

  /// Supprime le brouillon du moteur (écran vidé ou abandonné).
  void _dropEngineDraft(ComposeState cs) {
    final admin = source?.composer;
    if (admin == null) return;
    cs.push = cs.push
        .then((_) async {
          final id = cs.engineId;
          if (id == null) return;
          cs.engineId = null;
          await admin.delete(id);
        })
        .catchError((_) {});
  }

  void _leaveCompose() {
    final m = byId(open);
    if (m != null && m.folder != 'drafts') {
      view = PaneView.read;
    } else {
      view = PaneView.empty;
      open = null;
    }
  }

  void closeCompose([bool save = true]) {
    final cs = c;
    if (cs == null) return;
    cs.timer?.cancel();
    final saved = save ? saveDraft() : false;
    if (save) _copyToServer(cs);
    c = null;
    _leaveCompose();
    notifyListeners();
    if (saved) toast(t('Brouillon enregistré'));
  }

  void discard() {
    final cs = c;
    if (cs == null) return;
    cs.timer?.cancel();
    final had = byId(cs.id);
    if (had != null) mails = mails.where((x) => x != had).toList();
    _dropEngineDraft(cs);
    c = null;
    _leaveCompose();
    notifyListeners();
    toast(had != null ? t('Brouillon supprimé') : t('Message abandonné'));
  }

  Future<void> send([String? sched]) async {
    final cs = c;
    if (cs == null) return;
    if (cs.to.isEmpty) {
      cs.shake++;
      notifyListeners();
      toast('Ajoute au moins un destinataire', err: true);
      return;
    }
    if (source != null) return _sendReal(cs, sched);
    cs.timer?.cancel();
    final blocks = [...cs.body.toBlocks(), ...?cs.quote];
    final init = ComposeInit(
      mode: cs.mode,
      replyTo: cs.replyTo,
      to: [...cs.to],
      cc: [...cs.cc],
      bcc: [...cs.bcc],
      subject: cs.subject.text,
      text: cs.body.text,
      flags: List<int>.from(cs.body.flags),
      quote: cs.quote,
      atts: [...cs.atts],
      from: cs.from,
    );
    final subj = cs.subject.text.trim().isEmpty
        ? '(sans objet)'
        : cs.subject.text.trim();
    final orig = byId(cs.replyTo);
    final rmsg = Msg(
      from: senderFor(cs.from),
      date: nowMs(),
      body: blocks,
      att: [...cs.atts],
    );
    final sent = Mail(
      id: 'm${nextUid()}',
      folder: 'sent',
      account: accountByEmail(cs.from)?.id,
      from: senderFor(cs.from),
      to: [...cs.to],
      cc: [...cs.cc],
      subject: subj,
      date: nowMs(),
      sched: sched,
      msgs: [rmsg],
    );
    if (cs.id != null) mails = mails.where((m) => m.id != cs.id).toList();
    mails.add(sent);
    List<String>? flagsBackup;
    if (orig != null) {
      flagsBackup = [...orig.flags];
      orig.msgs.add(rmsg.copy());
      orig.flags = orig.flags.where((f) => f != 'needs').toList();
      if (!orig.flags.contains('waiting')) orig.flags.add('waiting');
    }
    c = null;
    _leaveCompose();
    notifyListeners();
    final timer = sched != null
        ? null
        : _after(rand(7000, 11000).round(), () => autoReply(sent, orig));
    toast(
      sched != null
          ? t('Message programmé · {0}', [t(sched)])
          : t('Message envoyé'),
      label: t('Annuler'),
      cb: () {
        timer?.cancel();
        mails = mails.where((x) => x.id != sent.id).toList();
        if (orig != null) {
          orig.msgs.removeLast();
          orig.flags = flagsBackup!;
        }
        openCompose(init);
      },
    );
  }

  /// Envoi par le moteur : brouillon enregistré, puis mis en file (annulable pendant le délai du moteur).
  Future<void> _sendReal(ComposeState cs, String? sched) async {
    final admin = source?.composer;
    if (admin == null || _sending) return;
    _sending = true;
    cs.timer?.cancel();
    cs.dirty = true;
    try {
      saveDraft();
      await cs.push;
      final id = cs.engineId;
      if (id == null) throw const EngineException('draft_missing', '');
      final action = await admin.send(
        id,
        at: sched == null ? null : scheduledMs(sched),
      );
      _afterSend(cs, id, action, sched);
    } catch (_) {
      toast(t('Envoi impossible, le message reste en brouillon'), err: true);
    } finally {
      _sending = false;
    }
  }

  bool _sending = false;

  /// Envois mis en file et pas encore visibles dans « Envoyés » : brouillon → (message affiché, fin de validité).
  final Map<String, (Mail, int)> _outgoing = {};

  void _afterSend(ComposeState cs, String draft, String action, String? sched) {
    final local = byId(cs.id);
    final account = accountByEmail(cs.from)?.id ?? activeId;
    final shown = Mail(
      id: 'sending:$draft',
      folder: 'sent',
      account: account,
      from: senderFor(cs.from),
      to: [...cs.to],
      cc: [...cs.cc],
      subject: cs.subject.text.trim().isEmpty
          ? '(sans objet)'
          : cs.subject.text.trim(),
      date: nowMs(),
      sched: sched,
      msgs: [
        Msg(
          from: senderFor(cs.from),
          date: nowMs(),
          body: [...cs.body.toBlocks(), ...?cs.quote],
          att: [...cs.atts],
        ),
      ],
    );
    final at = sched == null ? nowMs() : scheduledMs(sched);
    if (sched == null) _outgoing[draft] = (shown, at + 120000);
    mails = [...mails.where((m) => m != local), if (sched == null) shown];
    if (!unified) _boxes[account] = mails;
    _rememberContacts();
    c = null;
    _leaveCompose();
    notifyListeners();
    unawaited(_reloadOnce(account));
    toast(
      sched != null
          ? t('Message programmé · {0}', [t(sched)])
          : t('Message envoyé'),
      label: t('Annuler'),
      cb: () => _cancelSend(action, draft),
      ms: 10000,
    );
    _after(at - nowMs() + 11000, () => _settleSend(draft, account));
  }

  /// Après le départ du message : synchronise puis relit jusqu'à ce que « Envoyés » le contienne.
  Future<void> _settleSend(String draft, String account) async {
    try {
      await source?.syncNow();
    } catch (_) {
      /* la relecture suivante réessaie */
    }
    for (var i = 0; i < 8 && _outgoing.containsKey(draft); i++) {
      await Future<void>.delayed(const Duration(seconds: 2));
      await _reloadOnce(account);
    }
  }

  /// Ajoute aux messages relus les envois pas encore revenus du serveur.
  List<Mail> _withOutgoing(String account, List<Mail> fresh) {
    final now = nowMs();
    final keep = <Mail>[];
    for (final e in _outgoing.entries.toList()) {
      final (m, until) = e.value;
      if (m.account != account) continue;
      final arrived = fresh.any(
        (x) =>
            x.folder == 'sent' &&
            x.subject == m.subject &&
            x.date >= m.date - 60000,
      );
      if (arrived || now > until) {
        _outgoing.remove(e.key);
      } else {
        keep.add(m);
      }
    }
    final immediate = {
      for (final e in _outgoing.entries)
        if (e.value.$1.sched == null) 'outbox:${e.key}',
    };
    final shown = fresh.where((m) => !immediate.contains(m.id)).toList();
    return keep.isEmpty ? shown : [...shown, ...keep];
  }

  Future<void> _cancelSend(String action, String draft) async {
    try {
      await source!.composer!.cancelSend(action);
      final gone = _outgoing.remove(draft)?.$1;
      if (gone != null) mails = mails.where((m) => m != gone).toList();
      for (final a in accounts) {
        await _reloadOnce(a.id);
      }
      openMail('draft:$draft');
    } catch (_) {
      toast(t('Annulation impossible, le message est déjà parti'), err: true);
    }
  }

  void autoReply(Mail sent, Mail? orig) {
    if (source != null) return;
    if (!mails.contains(sent)) return;
    if (sent.to.isEmpty) return;
    final who = sent.to[0];
    if (isMine(who.email)) return;
    final html = pick(replies);
    if (orig != null && mails.contains(orig)) {
      orig.msgs.add(Msg(from: who, date: nowMs(), body: parseHtml(html)));
      orig.unread = true;
      orig.date = nowMs();
      if (!hidden(orig)) orig.folder = 'inbox';
      orig.flags = {
        ...orig.flags.where((f) => f != 'waiting'),
        'needs',
      }.toList();
      _notify(orig, who);
    } else {
      final m = mk(
        'inbox',
        who,
        RegExp(r'^re\s*:', caseSensitive: false).hasMatch(sent.subject)
            ? sent.subject
            : 'Re: ${sent.subject}',
        0,
        html,
        unread: true,
        flags: ['needs'],
        to: [sent.from],
      );
      m.account = sent.account;
      mails.add(m);
      _notify(m, who);
    }
  }

  void _notify(Mail m, Person who) {
    if (view == PaneView.read && open == m.id) m.unread = false;
    freshId = m.id;
    _after(700, () {
      if (freshId == m.id) freshId = null;
    });
    notifyListeners();
    toast(
      t('Nouveau message de {0}', [who.name]),
      label: t('Ouvrir'),
      cb: () {
        reveal(m);
        openMail(m.id);
      },
    );
  }

  void scheduleIncoming([int? first]) {
    _after(first ?? rand(35000, 60000).round(), () {
      if (!live) return;
      if (incIdx < incoming.length) {
        final d = incoming[incIdx++];
        final m = mk(
          'inbox',
          d.from,
          d.subject,
          0,
          d.html,
          unread: true,
          labels: d.labels,
          flags: d.flags,
          atts: d.atts,
        );
        mails.add(m);
        _notify(m, d.from);
        scheduleIncoming();
      }
    });
  }

  // ---- assistant ----
  void clearChat() {
    chat = chat.take(1).toList();
    notifyListeners();
  }

  void sendChat(String text) {
    final t = text.trim();
    if (t.isEmpty || busy) return;
    chat.add(ChatMsg(role: 'me', text: t));
    final ans = answerTo(this, t);
    final msg = ChatMsg(
      role: 'ai',
      done: false,
      text: ans.text,
      draft: ans.draft,
      refs: ans.refs,
    );
    chat.add(msg);
    busy = true;
    notifyListeners();
    var i = 0;
    void tick() {
      i = (i + rand(2, 5).round()).clamp(0, msg.text.length);
      msg.shown = msg.text.substring(0, i);
      if (i >= msg.text.length) {
        msg.done = true;
        busy = false;
      }
      notifyListeners();
      if (!msg.done) _after(22, tick);
    }

    _after(rand(500, 900).round(), tick);
  }

  void scheduleCleanup(List<String> ids) =>
      _after(1800, () => moveTo(ids, 'archive', 'archivé', 'archivés'));

  void useDraft(int i) {
    final msg = chat[i];
    final m = byId(open);
    if (msg.draft == null || m == null) return hint();
    view = PaneView.read;
    notifyListeners();
    replyTo('reply', intro: msg.draft!.join('\n\n'));
  }

  void openRef(String id) {
    final m = byId(id);
    if (m == null) return;
    view = PaneView.read;
    reveal(m);
    openMail(m.id);
  }

  void openCtx() {
    final m = byId(open);
    if (m != null) {
      view = PaneView.read;
      notifyListeners();
    }
  }
}

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:intl/date_symbol_data_local.dart';
import 'package:mira/data/format.dart';
import 'package:mira/data/html_lite.dart';
import 'package:mira/data/models.dart';
import 'package:mira/data/seed.dart';
import 'package:mira/state/assistant.dart';
import 'package:mira/state/rich_controller.dart';
import 'package:mira/i18n/i18n.dart';
import 'package:mira/state/search_filters.dart';
import 'package:mira/state/store.dart';
import 'package:mira/theme/wall_theme.dart';
import 'package:shared_preferences/shared_preferences.dart';

Future<MailStore> newStore([Map<String, Object> prefs = const {}]) async {
  SharedPreferences.setMockInitialValues(prefs);
  final s = MailStore(await SharedPreferences.getInstance());
  s.narrow = true; // pas d'ouverture automatique du premier message
  return s;
}

Mail byTitle(MailStore s, String sub) =>
    s.mails.firstWhere((m) => m.subject.contains(sub));

void main() {
  setUpAll(() => initializeDateFormatting('fr_FR'));

  group('HTML léger', () {
    test('paragraphes, gras, mentions et sauts de ligne', () {
      final b = parseHtml(
        '<p>Salut <strong>Noah</strong>,</p><p><span class="at">@Noah</span> ok<br>fin</p>',
      );
      expect(b.length, 2);
      expect((b[0] as Para).spans.map((s) => s.text).join(), 'Salut Noah,');
      expect((b[0] as Para).spans[1].bold, isTrue);
      expect((b[1] as Para).spans.first.at, isTrue);
      expect(plainOf(b), 'Salut Noah , @Noah ok fin');
    });

    test('blockquote imbriqué et contenu en ligne sans marge', () {
      final b = parseHtml(
        '<blockquote>Le X a écrit :<br><p>Hello</p></blockquote>',
      );
      final q = b.single as Quote;
      expect((q.children.first as Para).loose, isTrue);
      expect((q.children.last as Para).loose, isFalse);
    });
  });

  group('Formats', () {
    test('initiales, couleur stable, taille', () {
      expect(initials('Noah Toffa'), 'NT');
      expect(initials('GitHub'), 'GI');
      expect(initials('Équipe KERNEL'), 'ÉK');
      expect(colorOf('Awa Diallo'), colorOf('Awa Diallo'));
      expect(sizeOf(184 * 1024), '184 Ko');
      expect(sizeOf(1572864), '1,5 Mo');
    });

    test('regroupement par date', () {
      expect(groupOf(nowMs()), "Aujourd'hui");
      final now = DateTime.now();
      final yesterdayNoon = DateTime(now.year, now.month, now.day - 1, 12);
      expect(groupOf(yesterdayNoon.millisecondsSinceEpoch), 'Hier');
      expect(groupOf(nowMs() - 4 * dayMs), 'Cette semaine');
      expect(groupOf(nowMs() - 20 * dayMs), 'Plus ancien');
    });
  });

  group('Boîte de réception', () {
    test('compteurs de départ identiques à la maquette', () async {
      final s = await newStore();
      expect(s.navCount('inbox'), 6);
      expect(s.needsCount, 5);
      expect(s.navCount('starred'), 3);
      expect(s.navCount('drafts'), 1);
      expect(s.navCount('spam'), 2);
      expect(s.visible().length, 14);
    });

    test('filtres, recherche et tri', () async {
      final s = await newStore();
      s.setPill('needs');
      expect(s.visible().every((m) => m.flags.contains('needs')), isTrue);
      s.setPill('all');
      s.setQuery('cotonou');
      expect(s.visible().isNotEmpty, isTrue);
      s.setQuery('zzzz');
      expect(s.visible(), isEmpty);
      s.setQuery('');
      s.setSort('old');
      final l = s.visible();
      expect(l.first.date <= l.last.date, isTrue);
      s.setSort('unread');
      expect(s.visible().first.unread, isTrue);
    });

    test('libellés et dossiers', () async {
      final s = await newStore();
      s.selectLabel('projets');
      expect(s.visible().every((m) => m.labels.contains('projets')), isTrue);
      s.selectFolder('starred');
      expect(s.visible().every((m) => m.starred), isTrue);
      expect(s.label, isNull);
    });
  });

  group('Actions', () {
    test('archiver puis annuler', () async {
      final s = await newStore();
      final m = byTitle(s, 'Planning');
      s.openMail(m.id);
      s.archive();
      expect(m.folder, 'archive');
      expect(s.toasts.last.msg, '1 message archivé');
      s.toasts.last.cb!();
      expect(m.folder, 'inbox');
      s.dispose();
    });

    test('corbeille → suppression définitive', () async {
      final s = await newStore();
      final m = byTitle(s, 'frameworks');
      s.selectFolder('trash');
      s.toggleSel(m.id);
      s.trash();
      expect(s.byId(m.id), isNull);
      s.toasts.last.cb!();
      expect(s.byId(m.id), isNotNull);
      s.dispose();
    });

    test('sans sélection : message d’aide', () async {
      final s = await newStore();
      s.archive();
      expect(s.toasts.last.err, isTrue);
      s.dispose();
    });

    test('lu / non lu et libellés sur sélection', () async {
      final s = await newStore();
      final m = byTitle(s, 'Retour sur la maquette');
      s.toggleSel(m.id);
      s.toggleRead();
      expect(m.unread, isFalse);
      s.toggleSel(m.id);
      s.toggleLabel('factures');
      expect(m.labels.contains('factures'), isTrue);
      s.toggleLabel('factures');
      expect(m.labels.contains('factures'), isFalse);
      s.dispose();
    });

    test('libellé local : pose sur une conversation et mémorisation', () async {
      final s = await newStore();
      expect(s.addLocalLabel('Perso'), isTrue);
      expect(s.addLocalLabel('perso'), isFalse);
      final key = s.localLabels.defs.keys.single;
      final m = byTitle(s, 'Retour sur la maquette');
      s.toggleSel(m.id);
      s.toggleLabel(key);
      expect(m.labels.contains(key), isTrue);
      expect(s.labelTitle(key), 'Perso');
      final saved = s.prefs.getString('localLabels');
      expect(saved, contains('Perso'));
      s.toggleLabel(key);
      expect(m.labels.contains(key), isFalse);
      s.deleteLocalLabel(key);
      expect(s.localLabels.defs, isEmpty);
      s.dispose();
    });

    test(
      'filtres de recherche sans texte : seuls les messages retenus restent',
      () async {
        final s = await newStore();
        expect(s.searching, isFalse);
        s.setSearchFilters(const SearchFilters(unread: true));
        expect(s.searching, isTrue);
        final shown = s.visible();
        expect(shown, isNotEmpty);
        expect(shown.every((m) => m.unread), isTrue);
        s.setSearchFilters(
          const SearchFilters(unread: true, from: 'zzz-aucun'),
        );
        expect(s.visible(), isEmpty);
        s.clearQuery();
        expect(s.searchFilters.isEmpty, isTrue);
        expect(s.searching, isFalse);
        s.dispose();
      },
    );

    test('reporter déplace dans « Reportés »', () async {
      final s = await newStore();
      final m = byTitle(s, 'Planning');
      s.openMail(m.id);
      s.snooze('tom');
      expect(m.folder, 'snoozed');
      expect(m.snooze, 'demain 08:00');
      s.dispose();
    });
  });

  group('Rédaction', () {
    test('répondre à tous : destinataires sans doublon ni moi', () async {
      final s = await newStore();
      final m = byTitle(s, 'démo jeudi'); // Awa, cc Kofi
      s.openMail(m.id);
      s.replyTo('all');
      final c = s.c!;
      expect(c.to.single.email, awa.email);
      expect(c.cc.map((p) => p.email), [kofi.email]);
      expect(c.subject.text, 'Re: Bot WhatsApp stock — démo jeudi');
      expect(c.body.text.endsWith('Cordialement,\nNoah'), isTrue);
      expect(c.quote, isNotNull);
      s.dispose();
    });

    test('transférer : objet, pièces jointes, corps', () async {
      final s = await newStore();
      final m = byTitle(s, 'plan de projet');
      s.openMail(m.id);
      s.replyTo('fwd');
      final c = s.c!;
      expect(c.subject.text, 'Fwd: Ton plan de projet a été validé');
      expect(c.atts.single.n, 'plan-validation.pdf');
      expect(c.body.text.contains('Message transféré'), isTrue);
      s.dispose();
    });

    test('envoi : message envoyé, fil mis à jour, annulation', () async {
      final s = await newStore();
      final m = byTitle(s, 'plan de projet');
      s.openMail(m.id);
      s.replyTo('reply');
      s.send();
      final sent = s.mails.firstWhere(
        (x) => x.folder == 'sent' && x.subject.startsWith('Re: Ton plan'),
      );
      expect(m.msgs.length, 2);
      expect(m.flags.contains('waiting'), isTrue);
      expect(m.flags.contains('needs'), isFalse);
      expect(s.c, isNull);
      expect(sent.to.single.email, kernel.email);
      s.toasts.last.cb!(); // Annuler
      expect(s.mails.contains(sent), isFalse);
      expect(m.msgs.length, 1);
      expect(m.flags.contains('needs'), isTrue);
      expect(s.c, isNotNull);
      s.dispose();
    });

    test('envoi sans destinataire refusé', () async {
      final s = await newStore();
      s.openCompose(const ComposeInit());
      s.send();
      expect(s.toasts.last.msg, 'Ajoute au moins un destinataire');
      expect(s.c, isNotNull);
      s.dispose();
    });

    test('brouillon enregistré puis rouvert', () async {
      final s = await newStore();
      s.openCompose(const ComposeInit());
      s.c!.subject.text = 'Idée';
      s.c!.body.setContent('Bonjour', List<int>.filled(7, 0));
      s.touch();
      s.closeCompose();
      final d = s.mails.firstWhere(
        (m) => m.folder == 'drafts' && m.subject == 'Idée',
      );
      s.openMail(d.id);
      expect(s.c!.subject.text, 'Idée');
      expect(s.c!.body.text, 'Bonjour');
      s.dispose();
    });

    test('brouillon vide non conservé', () async {
      final s = await newStore();
      final before = s.mails.length;
      s.openCompose(const ComposeInit());
      s.touch();
      s.closeCompose();
      expect(s.mails.length, before);
      s.dispose();
    });

    test('réponse automatique simulée', () async {
      final s = await newStore();
      final m = byTitle(s, 'plan de projet');
      s.openMail(m.id);
      s.replyTo('reply');
      s.send();
      final sent = s.mails.firstWhere(
        (x) => x.folder == 'sent' && x.subject.startsWith('Re: Ton plan'),
      );
      s.autoReply(sent, m);
      expect(m.msgs.length, 3);
      expect(
        m.unread,
        isFalse,
      ); // le fil est ouvert : marqué lu à l'arrivée (comme l'original)
      expect(m.flags.contains('needs'), isTrue);
      expect(m.flags.contains('waiting'), isFalse);
      s.dispose();
    });
  });

  group('Éditeur riche', () {
    test('toBlocks : paragraphes, retours à la ligne et styles', () {
      final r = RichController()
        ..setContent('Salut\nNoah\n\nMerci', List<int>.filled(17, 0));
      r.flags[0] = Fx.bold;
      final b = r.toBlocks();
      expect(b.length, 2);
      expect((b[0] as Para).spans.first.bold, isTrue);
      expect((b[0] as Para).spans.any((s) => s.text == '\n'), isTrue);
      expect((b[1] as Para).spans.single.text, 'Merci');
    });

    test('saisie : les styles suivent le texte inséré', () {
      final r = RichController()..setContent('ab', [Fx.bold, Fx.bold]);
      r.value = const TextEditingValueX('abc', 3).v;
      expect(r.flags, [Fx.bold, Fx.bold, Fx.bold]);
      r.value = const TextEditingValueX('abXc', 3).v;
      expect(r.flags.length, 4);
    });

    test('sauts de paragraphe programmatiques', () {
      expect(RichController.gapFlags('a\n\nb'), [0, 0, Fx.gap, 0]);
    });

    test('puces', () {
      final r = RichController()
        ..setContent('un\ndeux', List<int>.filled(7, 0));
      r.selection = const TextSelection(baseOffset: 0, extentOffset: 7);
      r.toggleBullets();
      expect(r.text, '• un\n• deux');
      expect(r.flags.length, r.text.length);
      r.toggleBullets();
      expect(r.text, 'un\ndeux');
    });
  });

  group('Assistant simulé', () {
    test('sans contexte, demande de sélectionner un mail', () async {
      final s = await newStore();
      expect(
        answerTo(s, 'Résume ce mail').text,
        contains('Sélectionne d’abord'),
      );
      s.dispose();
    });

    test('résumé, brouillon, urgent, non lus, traduction', () async {
      final s = await newStore();
      s.openMail(byTitle(s, 'plan de projet').id);
      expect(answerTo(s, 'Résume ce mail').text, contains('Résumé de'));
      final d = answerTo(s, 'Rédige une réponse');
      expect(d.draft!.first, 'Bonjour Équipe,');
      expect(answerTo(s, 'Qu’est-ce qui est urgent ?').refs, isNotEmpty);
      expect(answerTo(s, 'Montre-moi mes mails non lus').refs, isNotEmpty);
      expect(
        answerTo(s, 'Traduis ce mail en anglais').text,
        contains('Traduction (EN)'),
      );
      expect(answerTo(s, 'ok').text, contains('Je peux'));
      s.dispose();
    });

    test('recherche par mots-clés', () async {
      final s = await newStore();
      final a = answerTo(s, 'Retrouve les messages sur hébergement');
      expect(a.refs, isNotEmpty);
      s.dispose();
    });
  });

  group('Comptes', () {
    test('bascule de compte : boîte, identité et état remis à zéro', () async {
      final s = await newStore();
      final primary = s.mails;
      s.selectFolder('archive');
      s.switchAccount('a2');
      expect(s.active.email, 'toffanoah@gmail.com');
      expect(me.email, 'toffanoah@gmail.com');
      expect(s.folder, 'inbox');
      expect(identical(s.mails, primary), isFalse);
      expect(s.prefs.getString('account'), 'a2');
      s.switchAccount('a1');
      expect(
        identical(s.mails, primary),
        isTrue,
      ); // la boîte du compte principal est conservée
      expect(me.email, 'noah@kernel.epitech.africa');
      s.dispose();
    });

    test('ajouter un compte : actif, persisté, doublon refusé', () async {
      final s = await newStore();
      expect(s.addAccount('Awa Diallo', 'awa@exemple.com'), isTrue);
      expect(s.active.email, 'awa@exemple.com');
      expect(s.mails.first.subject, 'Bienvenue sur MIRA');
      expect(s.addAccount('Autre', 'AWA@exemple.com'), isFalse);
      final s2 = await newStore({'accounts': s.prefs.getString('accounts')!});
      expect(s2.accounts.length, 4);
      s.dispose();
      s2.dispose();
      me = Person(defaultAccounts[0].name, defaultAccounts[0].email);
    });

    test(
      'boîte unifiée : tous les messages, comptes d’origine, retour',
      () async {
        final s = await newStore();
        final nPrimary = s.mails.length;
        s.showAllAccounts();
        expect(s.unified, isTrue);
        expect(s.mails.length, greaterThan(nPrimary));
        expect(s.mails.map((m) => m.account).toSet(), {'a1', 'a2', 'a3'});
        expect(s.prefs.getString('account'), 'all');
        // les compteurs cumulent tous les comptes
        expect(
          s.navCount('inbox'),
          s.mails.where((m) => m.folder == 'inbox' && m.unread).length,
        );
        // répondre depuis un message d'un autre compte : expéditeur = ce compte
        final g = s.mails.firstWhere(
          (m) =>
              m.account == 'a2' &&
              m.folder == 'inbox' &&
              m.subject.contains('Photos'),
        );
        s.openMail(g.id);
        s.replyTo('reply');
        expect(s.c!.from, 'toffanoah@gmail.com');
        expect(s.c!.to.single.email, awa.email);
        s.send();
        final sent = s.mails.firstWhere(
          (m) => m.folder == 'sent' && m.subject.startsWith('Re: Photos'),
        );
        expect(sent.account, 'a2');
        expect(sent.from.email, 'toffanoah@gmail.com');
        // retour sur un compte : le message envoyé est rangé dans la boîte de ce compte
        s.switchAccount('a2');
        expect(s.unified, isFalse);
        expect(s.mails.contains(sent), isTrue);
        expect(s.mails.every((m) => m.account == 'a2'), isTrue);
        s.dispose();
        me = Person(defaultAccounts[0].name, defaultAccounts[0].email);
      },
    );

    test('le « De » du composeur propose les comptes', () async {
      final s = await newStore();
      expect(
        s.accounts.map((a) => a.email),
        contains('contact@stockbot.africa'),
      );
      s.dispose();
    });
  });

  group('Thème du fond d’écran', () {
    const bg = Color(0xFF1E2C2E), fg = Color(0xFF959F9B);
    final cands = [
      for (final h in [
        '373E41',
        '33463C',
        '48453D',
        '3C6344',
        '5E6058',
        '6B706F',
        '3C663B',
        '718969',
      ])
        Color(0xFF000000 | int.parse(h, radix: 16)),
    ];

    test('fond sombre : surfaces par ratios, texte lisible, accent coloré', () {
      final r = palFromSeed(WallSeed(bg, fg, cands));
      expect(r.dark, isTrue);
      expect(r.pal.bg, bg);
      // surfaces de plus en plus claires, comme dans ShellArie
      expect(
        r.pal.bgSide.computeLuminance(),
        greaterThan(bg.computeLuminance()),
      );
      expect(
        r.pal.surface.computeLuminance(),
        greaterThan(r.pal.bgSide.computeLuminance()),
      );
      expect(
        r.pal.surface2.computeLuminance(),
        greaterThan(r.pal.surface.computeLuminance()),
      );
      expect(contrast(r.pal.text, bg), greaterThanOrEqualTo(10));
      expect(contrast(r.pal.muted, bg), greaterThanOrEqualTo(6.4));
      expect(contrast(r.pal.dim, bg), greaterThanOrEqualTo(4.4));
      // l'accent est une vraie couleur (teinte verte du fond d'écran), pas le texte
      final h = HSLColor.fromColor(r.pal.accent);
      expect(h.saturation, greaterThan(.45));
      expect(h.hue, inInclusiveRange(90, 160));
      expect(contrast(r.pal.accent, bg), greaterThanOrEqualTo(3.1));
    });

    test('fond clair : thème clair, texte sombre', () {
      final r = palFromSeed(
        const WallSeed(Color(0xFFEFEBE3), Color(0xFF3A3834), [
          Color(0xFFD6431A),
        ]),
      );
      expect(r.dark, isFalse);
      expect(
        r.pal.text.computeLuminance(),
        lessThan(r.pal.bg.computeLuminance()),
      );
      expect(contrast(r.pal.text, r.pal.bg), greaterThanOrEqualTo(10));
    });

    test('fond d’écran gris : l’accent retombe sur la couleur du texte', () {
      final r = palFromSeed(
        const WallSeed(Color(0xFF202020), Color(0xFFB0B0B0), [
          Color(0xFF444444),
          Color(0xFF888888),
        ]),
      );
      expect(contrast(r.pal.accent, r.pal.bg), greaterThanOrEqualTo(3.1));
      expect(HSLColor.fromColor(r.pal.accent).saturation, lessThan(.05));
    });

    test(
      'le magasin suit la palette, sauf si on choisit « Ignorer »',
      () async {
        final s = await newStore();
        expect(s.wallPal, isNull);
        s.debugSetWallSeed(WallSeed(bg, fg, cands));
        expect(s.wallPal, isNotNull);
        expect(s.isDark, isTrue);
        s.setSetting('follow', '0');
        expect(s.wallPal, isNull);
        expect(s.prefs.getString('follow'), '0');
        s.dispose();
      },
    );
  });

  group('Langue', () {
    tearDown(() => appLang = AppLang.fr);

    test('t() et tn() : français par défaut, anglais sur demande, repli sur la clé', () {
      expect(t('Réception'), 'Réception');
      expect(tn(3, 'message', 'messages'), '3 messages');
      appLang = AppLang.en;
      expect(t('Réception'), 'Inbox');
      expect(t('Compte actif : {0}', ['a@b.c']), 'Active account: a@b.c');
      expect(tn(1, 'message', 'messages'), '1 message');
      expect(t('Texte sans traduction'), 'Texte sans traduction');
    });

    test('toutes les clés de chaque langue ont les mêmes paramètres que le texte français', () {
      final ph = RegExp(r'\{\d+\}');
      for (final lang in AppLang.values) {
        for (final e in lang.strings.entries) {
          if (lang.fallbackCode == null) {
            expect(
              e.value.trim(),
              isNotEmpty,
              reason: '${lang.code}: ${e.key}',
            );
          }
          final a = ph.allMatches(e.key).map((m) => m.group(0)).toSet();
          final b = ph.allMatches(e.value).map((m) => m.group(0)).toSet();
          // « qui attend{1} » : le paramètre de pluriel français peut ne pas exister dans la langue cible
          expect(b.difference(a), isEmpty, reason: '${lang.code}: ${e.key}');
        }
      }
    });

    test('le magasin mémorise la langue et les dates suivent', () async {
      final s = await newStore();
      s.setLang(AppLang.en);
      expect(s.prefs.getString('lang'), 'en');
      expect(appLang, AppLang.en);
      s.setLang(AppLang.fr);
      expect(appLang, AppLang.fr);
      s.dispose();
    });

    test('l’assistant comprend l’anglais', () async {
      final s = await newStore();
      s.openMail(byTitle(s, 'plan de projet').id);
      expect(answerTo(s, 'Summarize this mail').text, contains('Résumé de'));
      expect(answerTo(s, 'Show me my unread mails').refs, isNotEmpty);
      appLang = AppLang.en;
      expect(answerTo(s, 'Summarize this mail').text, contains('Summary of'));
      expect(
        answerTo(s, 'What is urgent?').text,
        contains('waiting for a reply'),
      );
      s.dispose();
    });
  });

  group('Préférences', () {
    test('thème, densité et signature persistés', () async {
      final s = await newStore({
        'theme': 'light',
        'dense': '1',
        'sig': 'Salut',
      });
      expect(s.theme, 'light');
      expect(s.dense, isTrue);
      expect(s.sig, 'Salut');
      s.setSetting('dense', '0');
      expect(s.dense, isFalse);
      expect(s.prefs.getString('dense'), '0');
      s.dispose();
    });
  });
}

/// Petit utilitaire pour fabriquer des `TextEditingValue` avec le curseur à la fin.
class TextEditingValueX {
  final String t;
  final int c;
  const TextEditingValueX(this.t, this.c);
  TextEditingValue get v => TextEditingValue(
    text: t,
    selection: TextSelection.collapsed(offset: c),
  );
}

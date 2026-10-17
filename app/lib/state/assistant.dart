import '../data/format.dart';
import '../data/html_lite.dart';
import '../data/models.dart';
import 'store.dart';
import '../i18n/i18n.dart';

class Answer {
  final String text;
  final List<String>? draft;
  final List<String>? refs;
  Answer(this.text, {this.draft, this.refs});
}

List<List<String>> get suggestions => [
  [t('Résume ce mail'), t('Résume ce mail')],
  [t('Rédige une réponse'), t('Rédige une réponse')],
  [t('Qu’est-ce qui est urgent ?'), t('Qu’est-ce qui est urgent ?')],
  [t('Mes non lus'), t('Montre-moi mes mails non lus')],
  [t('Traduis en anglais'), t('Traduis ce mail en anglais')],
  [t('Nettoie ma boîte'), t('Nettoie ma boîte')],
];

List<String> _sentences(String body) {
  final r = RegExp(r'[^.!?]+[.!?]')
      .allMatches(body)
      .map((m) => m.group(0)!)
      .toList();
  return r.isEmpty ? [body] : r;
}

Answer answerTo(MailStore s, String q) {
  final ql = q.toLowerCase();
  final m = s.byId(s.open);
  final inbox = s.mails.where((x) => x.folder == 'inbox').toList();
  final last = m?.msgs.last;
  final needCtx = t(
    'Sélectionne d’abord un mail dans la liste, et je m’en occupe.',
  );
  bool has(String re) => RegExp(re).hasMatch(ql);

  if (has(r'r[ée]sum|summar')) {
    if (m == null) return Answer(needCtx);
    final body = plainOf(last!.body);
    final atts = m.msgs.expand((x) => x.att).toList();
    return Answer(
      t('**Résumé de « {0} »**\n{1} écrit : {2}', [
            m.subject,
            m.from.name,
            _sentences(body).take(2).join(' ').trim(),
          ]) +
          (atts.isNotEmpty
              ? t('\n\nPièce(s) jointe(s) : {0}.', [
                  atts.map((a) => a.n).join(', '),
                ])
              : '') +
          (m.flags.contains('needs')
              ? t('\n\nUne réponse est attendue de ta part.')
              : ''),
    );
  }
  if (has(
    r'r[ée]dig|r[ée]ponds|r[ée]ponse|brouillon|[ée]cris|write|draft|repl',
  )) {
    if (m == null || m.folder == 'drafts') return Answer(needCtx);
    final first = (isMine(last!.from.email) ? m.to[0] : last.from).name.split(
      ' ',
    )[0];
    var core = t(
      'Merci pour ton message, je regarde ça et je reviens vers toi très vite.',
    );
    if (m.labels.contains('candidatures')) {
      core = t(
        'Merci pour le rappel. Je finalise mon dossier et dépose mon CV mis à jour avant la date limite.',
      );
    } else if (m.labels.contains('projets')) {
      core = t(
        'Merci pour le retour ! Je démarre la suite dès demain et je te tiens au courant de l’avancement chaque semaine.',
      );
    } else if (m.labels.contains('factures')) {
      core = t(
        'Bien reçu, merci. Je m’en occupe et je te confirme le règlement dans les prochains jours.',
      );
    } else if (m.labels.contains('equipe')) {
      core = t(
        'Super, merci ! Je jette un œil dans la journée et je te dis ce que j’en pense.',
      );
    }
    return Answer(
      t('Voici une proposition de réponse pour **{0}** :', [first]),
      draft: [
        t('Bonjour {0},', [first]),
        core,
      ],
    );
  }
  if (has(r'urgent|prioritaire|[àa] r[ée]pondre|trie|priority|to reply')) {
    final l = inbox.where((x) => x.flags.contains('needs')).toList()
      ..sort((a, b) => b.date - a.date);
    return l.isNotEmpty
        ? Answer(
            t(
              'Tu as **{0}** qui attend{1} une réponse. Je te les ai classés du plus récent au plus ancien :',
              [tn(l.length, 'message', 'messages'), l.length > 1 ? 'ent' : ''],
            ),
            refs: l.take(5).map((x) => x.id).toList(),
          )
        : Answer(t('Rien d’urgent : tu es à jour sur tes réponses. 🎉'));
  }
  if (has(r'non lu|nouveau|unread')) {
    final l = inbox.where((x) => x.unread).toList()
      ..sort((a, b) => b.date - a.date);
    return l.isNotEmpty
        ? Answer(
            t('Tu as **{0}** en réception :', [
              tn(l.length, 'mail non lu', 'mails non lus'),
            ]),
            refs: l.take(6).map((x) => x.id).toList(),
          )
        : Answer(t('Aucun mail non lu pour le moment.'));
  }
  if (has(r'traduis|anglais|english|translat')) {
    if (m == null) return Answer(needCtx);
    final body = plainOf(last!.body);
    final s1 = _sentences(body).take(2).join(' ').trim();
    return Answer(
      t(
        '**Traduction (EN) de « {0} »**\nHi Noah, {1}this message from {2} says: “{3}”\n\n(Traduction simulée pour la démo.)',
        [
          m.subject,
          m.flags.contains('needs') ? 'we are waiting for your answer: ' : '',
          m.from.name,
          s1,
        ],
      ),
    );
  }
  if (has(r'nettoi|archiv|range|clean|tidy')) {
    final l = inbox
        .where(
          (x) =>
              !x.unread &&
              !x.starred &&
              x.flags.isEmpty &&
              diffDays(x.date) >= 2,
        )
        .toList();
    if (l.isEmpty) {
      return Answer(t('Ta boîte est déjà propre : rien à archiver.'));
    }
    s.scheduleCleanup(l.map((x) => x.id).toList());
    return Answer(
      t(
        'J’archive **{0}**, déjà lus et sans action en attente depuis plus de 2 jours. Tu pourras annuler.',
        [tn(l.length, 'message', 'messages')],
      ),
    );
  }
  const stop = {
    'avec',
    'dans',
    'pour',
    'mais',
    'cherche',
    'trouve',
    'retrouve',
    'montre',
    'moi',
    'mail',
    'mails',
    'message',
    'messages',
    'quel',
    'quels',
    'quelle',
    'with',
    'this',
    'that',
    'find',
    'search',
    'show',
    'what',
    'which',
    'about',
  };
  final words = ql
      .replaceAll(RegExp(r'[^\p{L}\p{N}\s@.]', unicode: true), ' ')
      .split(RegExp(r'\s+'))
      .where((w) => w.length > 3 && !stop.contains(w))
      .toList();
  if (words.isNotEmpty) {
    final l = s.mails.where((x) {
      if (s.hidden(x)) return false;
      final hay =
          '${x.subject} ${x.from.name} ${plainOf(x.msgs.expand((y) => y.body).toList())}'
              .toLowerCase();
      return words.any(hay.contains);
    }).toList()..sort((a, b) => b.date - a.date);
    if (l.isNotEmpty) {
      return Answer(
        t('J’ai trouvé **{0}** autour de « {1} » :', [
          tn(l.length, 'message', 'messages'),
          words.join(' '),
        ]),
        refs: l.take(5).map((x) => x.id).toList(),
      );
    }
  }
  return Answer(
    t(
      'Je peux **résumer** un mail, **rédiger** une réponse, **lister l’urgent**, afficher tes **non lus**, **traduire** ou **nettoyer** ta boîte. Essaie une des suggestions ci-dessous.',
    ),
  );
}

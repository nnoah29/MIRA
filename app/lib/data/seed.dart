import 'dart:math';

import 'format.dart';
import '../i18n/i18n.dart';
import 'html_lite.dart';
import 'models.dart';

const kernel = Person(
  'Équipe KERNEL', // i18n:ignore
  'equipe@kernel.epitech.africa',
); // i18n:ignore
const awa = Person('Awa Diallo', 'awa@stockbot.africa');
const gh = Person('GitHub', 'notifications@github.com');
const supa = Person('Supabase', 'billing@supabase.io');
const kofi = Person('Kofi Agbodjan', 'kofi@stockbot.africa');
const epi = Person('Epitech Africa', 'admissions@epitech.africa');
const sand = Person('Sandrine Mensah', 'sandrine@lumen.studio');
const vercel = Person('Vercel', 'no-reply@vercel.com');
const yao = Person('Yao Kpadonou', 'yao@cotonou-market.bj');
const linda = Person('Linda Hounsou', 'linda@epitech.africa');
const ovh = Person('OVHcloud', 'support@ovhcloud.com');

const contacts = <Person>[
  kernel,
  awa,
  gh,
  supa,
  kofi,
  epi,
  sand,
  vercel,
  yao,
  linda,
  ovh,
  Person('Jean Adjovi', 'jean@cotonou-tech.bj'),
  Person('Mariam Sow', 'mariam@dakar-lab.sn'),
];

String ps(List<String> a) => a.map((t) => '<p>$t</p>').join();
Attachment att(String n, String t, String s) => Attachment(n, t, s);

int _uid = 100;
int nextUid() => ++_uid;

Mail mk(
  String folder,
  Person from,
  String subject,
  int ago,
  String html, {
  List<Person>? to,
  List<Person>? cc,
  bool unread = false,
  bool star = false,
  List<String>? labels,
  List<String>? flags,
  String? snooze,
  List<Attachment>? atts,
}) {
  final date = nowMs() - ago;
  return Mail(
    id: 'm${nextUid()}',
    folder: folder,
    from: from,
    to: to ?? [me],
    cc: cc ?? [],
    subject: subject,
    date: date,
    unread: unread,
    starred: star,
    labels: labels,
    flags: flags,
    snooze: snooze,
    msgs: [Msg(from: from, date: date, body: parseHtml(html), att: atts)],
  );
}

List<Mail> seed() {
  final m = <Mail>[];
  Mail add(Mail x) {
    m.add(x);
    return x;
  }

  add(
    mk(
      'inbox',
      kernel,
      t('Ton plan de projet a été validé'),
      25 * minMs,
      ps([
        t('Salut Noah,'),
        t(
          'Le plan est validé en amont, avant que tu lances le projet. <strong>Bot WhatsApp stock</strong> a passé la relecture : le problème est clair, la solution est réaliste et le périmètre tient en quelques semaines.',
        ),
        t(
          'Tu peux maintenant passer à la réalisation. Le projet fini sera validé une seconde fois. Deux crédits sont réservés pour toi.',
        ),
        t('À très vite,<br><strong>L’équipe KERNEL</strong>'),
      ]),
      unread: true,
      labels: ['projets'],
      flags: ['needs'],
      star: true,
      atts: [att('plan-validation.pdf', 'PDF', t('184 Ko'))],
    ),
  );

  final thread = add(
    mk(
      'inbox',
      awa,
      t('Re : Bot WhatsApp stock — démo jeudi'),
      95 * minMs,
      ps([
        t('Super, merci pour le point d’hier !'),
        t(
          'Je serai dispo à <strong>15 h jeudi</strong> pour voir la démo avec le commerçant de Cotonou. Peux-tu confirmer que le bot gère bien les ruptures de stock par SMS, même sans smartphone ?',
        ),
        t(
          '<span class="at">@Noah</span> dis-moi aussi si je dois préparer quelque chose de mon côté.',
        ),
        'Awa',
      ]),
      unread: true,
      labels: ['projets'],
      flags: ['needs', 'mention'],
      cc: [kofi],
    ),
  );
  thread.msgs.insert(
    0,
    Msg(
      from: me,
      date: nowMs() - 20 * hourMs,
      body: parseHtml(
        ps([
          t('Salut Awa,'),
          t(
            'Voici l’avancement : le bot répond déjà aux commandes de stock par SMS. Je termine la partie relances et on peut caler une démo jeudi.',
          ),
          'Noah',
        ]),
      ),
    ),
  );

  add(
    mk(
      'inbox',
      gh,
      t('[stock-bot] Pull request #12 fusionnée'),
      190 * minMs,
      ps([
        t(
          '<strong>kofi-a</strong> a fusionné <em>« feat: relances SMS »</em> dans <code>main</code>.',
        ),
        t('3 fichiers modifiés · +184 −27'),
        t('Aucun conflit détecté. Les vérifications CI sont passées.'),
      ]),
      unread: true,
      labels: ['equipe'],
    ),
  );

  add(
    mk(
      'inbox',
      sand,
      t('Retour sur la maquette'),
      260 * minMs,
      ps([
        t('Hello Noah,'),
        t(
          'J’ai regardé la maquette de la boîte mail : le rythme typographique est très bon. Un point : <span class="at">@Noah</span> peux-tu augmenter le contraste des libellés mono sur fond clair ?',
        ),
        t('Sinon je valide pour ma part.'),
        'Sandrine',
      ]),
      unread: true,
      flags: ['needs', 'mention'],
      labels: ['equipe'],
    ),
  );

  add(
    mk(
      'inbox',
      epi,
      t('Rappel : dépôt de candidature avant le 20'),
      20 * hourMs,
      ps([
        t('Bonjour Noah,'),
        t(
          'Il te reste quelques jours pour finaliser ton dossier de candidature. Pense à compléter ton profil et à joindre ton <strong>CV mis à jour</strong>.',
        ),
        t(
          'Sans dépôt avant le 20 à minuit, ton dossier ne sera pas examiné pour cette session.',
        ),
        t('Le service des admissions'),
      ]),
      unread: true,
      labels: ['candidatures'],
      flags: ['needs'],
    ),
  );

  add(
    mk(
      'inbox',
      supa,
      t('Votre facture de septembre est disponible'),
      26 * hourMs,
      ps([
        t('Bonjour,'),
        t(
          'Votre facture pour la période du 1er au 30 septembre est disponible.',
        ),
        t(
          'Montant : <strong>0,00 €</strong> — plan Free. Aucune action requise de votre part.',
        ),
      ]),
      labels: ['factures'],
      atts: [att('facture-septembre.pdf', 'PDF', t('92 Ko'))],
    ),
  );

  add(
    mk(
      'inbox',
      kofi,
      t('Idées pour la landing page'),
      30 * hourMs,
      ps([
        t('Salut,'),
        t(
          'Je t’ai mis trois maquettes en pièce jointe pour la landing du bot. Dis-moi laquelle tu préfères avant mardi, que j’attaque l’intégration.',
        ),
        t('La 2 est ma préférée (le titre en condensé passe vraiment bien).'),
        'Kofi',
      ]),
      unread: true,
      labels: ['equipe'],
      flags: ['needs'],
      atts: [
        att('landing-A.png', 'PNG', t('480 Ko')),
        att('landing-B.png', 'PNG', t('512 Ko')),
        att('landing-C.png', 'PNG', t('455 Ko')),
      ],
    ),
  );

  add(
    mk(
      'inbox',
      vercel,
      t('Déploiement réussi — kernel-mail'),
      50 * hourMs,
      ps([
        t(
          'Votre projet <strong>kernel-mail</strong> a été déployé avec succès.',
        ),
        t(
          'Production : kernel-mail.vercel.app · Durée : 41 s · Commit : 3f9c2a1',
        ),
      ]),
    ),
  );

  add(
    mk(
      'inbox',
      sand,
      t('Merci pour ton retour !'),
      54 * hourMs,
      ps([
        t(
          'C’était super clair, je transmets au reste de l’équipe dès demain matin.',
        ),
        t('Bonne soirée !'),
        'Sandrine',
      ]),
      labels: ['equipe'],
    ),
  );

  add(
    mk(
      'inbox',
      awa,
      t('Planning de la semaine'),
      80 * hourMs,
      ps([
        t('Voici le planning :'),
        t(
          '• Lundi : tests terrain à Cotonou<br>• Mercredi : point d’avancement<br>• Jeudi : démo commerçant (15 h)<br>• Vendredi : rétrospective',
        ),
        'Awa',
      ]),
      labels: ['projets'],
      star: true,
    ),
  );

  add(
    mk(
      'inbox',
      supa,
      t('Nouvelle connexion détectée'),
      3 * dayMs + 2 * hourMs,
      ps([
        t(
          'Une nouvelle connexion à votre compte a été détectée depuis <strong>Cotonou, Bénin</strong> (Chrome sur Linux).',
        ),
        t('Si c’était vous, aucune action n’est nécessaire.'),
      ]),
    ),
  );

  add(
    mk(
      'inbox',
      yao,
      t('Devis hébergement boutique'),
      4 * dayMs,
      ps([
        t('Bonjour Noah,'),
        t(
          'Suite à notre appel, voici le devis pour l’hébergement de la boutique : 12 000 FCFA / mois, sauvegardes incluses. J’attends ton retour pour lancer la mise en place.',
        ),
        'Yao',
      ]),
      flags: ['waiting'],
      labels: ['factures'],
      atts: [att('devis-hebergement.pdf', 'PDF', t('230 Ko'))],
    ),
  );

  add(
    mk(
      'inbox',
      linda,
      t('Compte rendu de la réunion du 8'),
      5 * dayMs + 3 * hourMs,
      ps([
        t('Salut à tous,'),
        t(
          'Voici le compte rendu : priorité à la démo, relecture des specs d’ici vendredi, et choix de l’hébergeur à trancher.',
        ),
        'Linda',
      ]),
      flags: ['waiting'],
      labels: ['equipe'],
    ),
  );

  add(
    mk(
      'inbox',
      epi,
      t('Bienvenue sur KERNEL'),
      9 * dayMs,
      ps([
        t('Bienvenue Noah !'),
        t(
          'Ton espace est prêt. Dépose ton premier projet, gagne des points de score et débloque tes premiers badges.',
        ),
      ]),
      star: true,
      labels: ['projets'],
    ),
  );

  // Envoyés
  add(
    mk(
      'sent',
      me,
      t('Re : Idées pour la landing page'),
      29 * hourMs,
      ps([
        t(
          'Merci Kofi, je regarde ça ce soir et je te dis laquelle je retiens.',
        ),
        'Noah',
      ]),
      to: [kofi],
    ),
  );
  add(
    mk(
      'sent',
      me,
      t('Proposition de démo — Bot WhatsApp stock'),
      2 * dayMs + 4 * hourMs,
      ps([
        t('Bonjour Awa,'),
        t(
          'Je te propose une démo jeudi à 15 h avec le commerçant. Dis-moi si ça te convient.',
        ),
        'Noah',
      ]),
      to: [awa],
    ),
  );

  // Brouillon
  final d = add(
    mk(
      'drafts',
      me,
      t('Candidature — lettre de motivation'),
      5 * hourMs,
      ps([
        t('Madame, Monsieur,'),
        t(
          'Passionné par le développement et l’impact local, je souhaite rejoindre la prochaine session…',
        ),
      ]),
      to: [epi],
    ),
  );
  d.draft = DraftMeta(from: me.email, bcc: [], replyTo: null, mode: 'new');

  // Archives
  add(
    mk(
      'archive',
      ovh,
      t('Contrat hébergement — confirmation'),
      14 * dayMs,
      ps([
        t(
          'Votre contrat d’hébergement est actif. Retrouvez votre facture dans votre espace client.',
        ),
      ]),
      labels: ['factures'],
      atts: [att('contrat.pdf', 'PDF', t('310 Ko'))],
    ),
  );
  add(
    mk(
      'archive',
      kernel,
      t('Ton badge « Premier pas » est débloqué'),
      12 * dayMs,
      ps([
        t(
          'Félicitations ! Tu viens de débloquer ton premier badge sur KERNEL.',
        ),
      ]),
    ),
  );

  // Reportés
  add(
    mk(
      'snoozed',
      ovh,
      t('Relance facture mensuelle'),
      6 * dayMs,
      ps([t('Nous n’avons pas reçu le règlement de votre facture mensuelle.')]),
      snooze: 'demain 08:00',
      labels: ['factures'],
    ),
  );

  // Spam
  add(
    mk(
      'spam',
      const Person('Promo Crypto', 'win@crypto-gains.xyz'),
      t('Gagnez 1 000 € par jour sans effort !!!'),
      7 * hourMs,
      ps([t('Cliquez ici pour découvrir notre méthode secrète…')]),
      unread: true,
    ),
  );
  add(
    mk(
      'spam',
      const Person('Livraison Express', 'colis@dhl-track.top'),
      t('Votre colis est bloqué — action requise'),
      2 * dayMs,
      ps([t('Payez 1,99 € de frais pour débloquer votre colis.')]),
      unread: true,
    ),
  );
  add(
    mk(
      'spam',
      const Person('Offre Exclusive', 'deal@mega-promo.click'),
      t('-90 % sur tout, aujourd’hui seulement'),
      6 * dayMs,
      ps([t('Dernière chance, ne manquez pas cette offre incroyable.')]),
    ),
  );

  // Corbeille
  add(
    mk(
      'trash',
      const Person('Newsletter Tech', 'news@techweekly.io'),
      t('Les 10 frameworks JS à connaître'),
      8 * dayMs,
      ps([t('Cette semaine : encore un nouveau framework…')]),
    ),
  );

  return m;
}

class IncomingDef {
  final Person from;
  final String subject;
  final List<String> labels;
  final List<String> flags;
  final String html;
  final List<Attachment>? atts;
  const IncomingDef(
    this.from,
    this.subject,
    this.labels,
    this.flags,
    this.html, [
    this.atts,
  ]);
}

List<IncomingDef> get incoming => <IncomingDef>[
  IncomingDef(
    awa,
    t('Démo confirmée jeudi 15 h'),
    ['projets'],
    ['needs'],
    ps([
      t('Bonne nouvelle : le commerçant est partant pour jeudi à 15 h !'),
      t(
        'Il aimerait voir la commande de stock par SMS en direct. Tu peux prévoir un scénario court ?',
      ),
      'Awa',
    ]),
  ),
  IncomingDef(
    gh,
    t('[stock-bot] Nouvelle issue #14 : relance SMS en double'),
    ['equipe'],
    [],
    ps([
      t('<strong>kofi-a</strong> a ouvert l’issue <em>#14</em>.'),
      t(
        'Les relances SMS partent parfois deux fois quand le stock passe sous le seuil pendant la synchro.',
      ),
    ]),
  ),
  IncomingDef(
    epi,
    t('Entretien : choisis ton créneau'),
    ['candidatures'],
    ['needs'],
    ps([
      t('Bonjour Noah,'),
      t(
        'Ton dossier a retenu notre attention ! Choisis un créneau d’entretien parmi ceux proposés dans ton espace avant vendredi.',
      ),
      t('Les admissions'),
    ]),
  ),
  IncomingDef(
    kofi,
    t('Tu as vu le nouveau logo ?'),
    ['equipe'],
    [],
    ps([
      t(
        'Regarde ce que j’ai sorti ce matin, je trouve que le carré orange fonctionne vraiment bien.',
      ),
      'Kofi',
    ]),
    [Attachment('logo-final.png', 'PNG', '128 Ko')],
  ),
  IncomingDef(
    supa,
    t('Votre projet approche de la limite gratuite'),
    ['factures'],
    [],
    ps([
      t(
        'Votre projet <strong>stock-bot</strong> a utilisé 82 % de son quota mensuel de requêtes.',
      ),
    ]),
  ),
  IncomingDef(
    vercel,
    t('Build échoué — kernel-mail'),
    [],
    ['needs'],
    ps([
      t('Le dernier déploiement de <strong>kernel-mail</strong> a échoué.'),
      t('<code>Error: Cannot find module "./theme"</code>'),
    ]),
  ),
];

final replies = <String>[
  ps([
    t('Parfait, merci pour ta réponse !'),
    t('Je regarde ça et je reviens vers toi dans la journée.'),
  ]),
  ps([
    t('Reçu 👍'),
    t(
      'C’est exactement ce qu’il me fallait. On en reparle demain à la première heure ?',
    ),
  ]),
  ps([
    t('Merci beaucoup, c’est très clair.'),
    t('Je transmets à l’équipe et on te confirme rapidement.'),
  ]),
  ps([
    t(
      'Top ! Juste une question : peux-tu m’envoyer la version finale en pièce jointe quand elle est prête ?',
    ),
  ]),
];

final rng = Random();
T pick<T>(List<T> a) => a[rng.nextInt(a.length)];
double rand(double a, double b) => a + rng.nextDouble() * (b - a);

/// Boîte fictive d'un compte secondaire (l'identité active [me] doit déjà être celle du compte).
List<Mail> seedFor(Account a) {
  final m = <Mail>[];
  Mail add(Mail x) {
    m.add(x);
    return x;
  }

  if (a.id == 'a2') {
    add(
      mk(
        'inbox',
        const Person('Google', 'no-reply@accounts.google.com'),
        t('Alerte de sécurité'),
        40 * minMs,
        ps([
          t(
            'Une nouvelle connexion à votre compte Google a été détectée sur <strong>Linux</strong>.',
          ),
          t('Si c’était vous, vous n’avez rien à faire.'),
        ]),
        unread: true,
      ),
    );
    add(
      mk(
        'inbox',
        awa,
        t('Photos du déplacement à Cotonou'),
        5 * hourMs,
        ps([
          t('Salut Noah,'),
          t(
            'Je t’envoie les photos des tests terrain. Dis-moi lesquelles tu veux garder pour la présentation.',
          ),
          'Awa',
        ]),
        unread: true,
        flags: ['needs'],
        atts: [
          att('terrain-01.png', 'PNG', t('1,2 Mo')),
          att('terrain-02.png', 'PNG', t('980 Ko')),
        ],
      ),
    );
    add(
      mk(
        'inbox',
        const Person('Newsletter Tech', 'news@techweekly.io'),
        t('Cette semaine : Flutter 3.47'),
        30 * hourMs,
        ps([
          t(
            'Les nouveautés de la semaine : performances, outils et ressources à ne pas manquer.',
          ),
        ]),
        labels: ['projets'],
      ),
    );
    add(
      mk(
        'inbox',
        yao,
        t('Photo de famille'),
        3 * dayMs,
        ps([t('Voici la photo de dimanche, à bientôt !'), 'Yao']),
        star: true,
      ),
    );
    add(
      mk(
        'sent',
        me,
        t('Re : Photos du déplacement à Cotonou'),
        4 * hourMs,
        ps([t('Merci Awa, je trie ça ce soir.'), 'Noah']),
        to: [awa],
      ),
    );
  } else if (a.id == 'a3') {
    add(
      mk(
        'inbox',
        yao,
        t('Commande de 40 unités'),
        2 * hourMs,
        ps([
          t('Bonjour,'),
          t(
            'Nous souhaiterions passer commande de <strong>40 unités</strong> pour la boutique de Cotonou. Pouvez-vous confirmer le délai de livraison ?',
          ),
          'Yao',
        ]),
        unread: true,
        flags: ['needs'],
        labels: ['factures'],
      ),
    );
    add(
      mk(
        'inbox',
        const Person('Mariam Sow', 'mariam@dakar-lab.sn'),
        t('Partenariat Dakar'),
        26 * hourMs,
        ps([
          t('Bonjour,'),
          t(
            'Notre laboratoire serait ravi de tester le bot WhatsApp de stock. Pouvons-nous planifier un appel cette semaine ?',
          ),
          'Mariam',
        ]),
        unread: true,
        flags: ['needs'],
        labels: ['projets'],
      ),
    );
    add(
      mk(
        'inbox',
        supa,
        t('Votre projet stock-bot est en pause'),
        4 * dayMs,
        ps([
          t(
            'Votre projet <strong>stock-bot</strong> a été mis en pause après 7 jours d’inactivité.',
          ),
        ]),
        labels: ['equipe'],
      ),
    );
    add(
      mk(
        'sent',
        me,
        t('Devis hébergement — réponse'),
        3 * dayMs,
        ps([
          t('Bonjour Yao, voici notre devis détaillé.'),
          t('L’équipe StockBot'),
        ]),
        to: [yao],
      ),
    );
  } else {
    final first = a.name.trim().split(RegExp(r'\s+')).first;
    add(
      mk(
        'inbox',
        const Person('Équipe MIRA', 'equipe@mira.app'), // i18n:ignore
        t('Bienvenue sur MIRA'),
        5 * minMs,
        ps([
          t('Bonjour {0},', [first]),
          t(
            'Votre compte <strong>{0}</strong> est maintenant connecté. Retrouvez ici tous vos messages, libellés et brouillons.',
            [a.email],
          ),
          t('Bonne découverte !'),
        ]),
        unread: true,
        star: true,
      ),
    );
    add(
      mk(
        'inbox',
        const Person('Équipe MIRA', 'equipe@mira.app'), // i18n:ignore
        t('Astuce : raccourcis clavier'),
        2 * hourMs,
        ps([
          t(
            'Appuyez sur <strong>?</strong> pour afficher tous les raccourcis, et sur <strong>c</strong> pour écrire un nouveau message.',
          ),
        ]),
      ),
    );
  }
  return m;
}

/// Boîte fictive d'un compte, quel qu'il soit.
List<Mail> seedAccount(Account a) {
  me = Person(a.name, a.email);
  currentAccountId = a.id;
  return a.id == 'a1' ? seed() : seedFor(a);
}

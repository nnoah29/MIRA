import 'dart:math';

import 'format.dart';
import 'html_lite.dart';
import 'models.dart';

const kernel = Person('Équipe KERNEL', 'equipe@kernel.epitech.africa');
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
      'Ton plan de projet a été validé',
      25 * minMs,
      ps([
        'Salut Noah,',
        'Le plan est validé en amont, avant que tu lances le projet. <strong>Bot WhatsApp stock</strong> a passé la relecture : le problème est clair, la solution est réaliste et le périmètre tient en quelques semaines.',
        'Tu peux maintenant passer à la réalisation. Le projet fini sera validé une seconde fois. Deux crédits sont réservés pour toi.',
        'À très vite,<br><strong>L’équipe KERNEL</strong>',
      ]),
      unread: true,
      labels: ['projets'],
      flags: ['needs'],
      star: true,
      atts: [att('plan-validation.pdf', 'PDF', '184 Ko')],
    ),
  );

  final t = add(
    mk(
      'inbox',
      awa,
      'Re : Bot WhatsApp stock — démo jeudi',
      95 * minMs,
      ps([
        'Super, merci pour le point d’hier !',
        'Je serai dispo à <strong>15 h jeudi</strong> pour voir la démo avec le commerçant de Cotonou. Peux-tu confirmer que le bot gère bien les ruptures de stock par SMS, même sans smartphone ?',
        '<span class="at">@Noah</span> dis-moi aussi si je dois préparer quelque chose de mon côté.',
        'Awa',
      ]),
      unread: true,
      labels: ['projets'],
      flags: ['needs', 'mention'],
      cc: [kofi],
    ),
  );
  t.msgs.insert(
    0,
    Msg(
      from: me,
      date: nowMs() - 20 * hourMs,
      body: parseHtml(
        ps([
          'Salut Awa,',
          'Voici l’avancement : le bot répond déjà aux commandes de stock par SMS. Je termine la partie relances et on peut caler une démo jeudi.',
          'Noah',
        ]),
      ),
    ),
  );

  add(
    mk(
      'inbox',
      gh,
      '[stock-bot] Pull request #12 fusionnée',
      190 * minMs,
      ps([
        '<strong>kofi-a</strong> a fusionné <em>« feat: relances SMS »</em> dans <code>main</code>.',
        '3 fichiers modifiés · +184 −27',
        'Aucun conflit détecté. Les vérifications CI sont passées.',
      ]),
      unread: true,
      labels: ['equipe'],
    ),
  );

  add(
    mk(
      'inbox',
      sand,
      'Retour sur la maquette',
      260 * minMs,
      ps([
        'Hello Noah,',
        'J’ai regardé la maquette de la boîte mail : le rythme typographique est très bon. Un point : <span class="at">@Noah</span> peux-tu augmenter le contraste des libellés mono sur fond clair ?',
        'Sinon je valide pour ma part.',
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
      'Rappel : dépôt de candidature avant le 20',
      20 * hourMs,
      ps([
        'Bonjour Noah,',
        'Il te reste quelques jours pour finaliser ton dossier de candidature. Pense à compléter ton profil et à joindre ton <strong>CV mis à jour</strong>.',
        'Sans dépôt avant le 20 à minuit, ton dossier ne sera pas examiné pour cette session.',
        'Le service des admissions',
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
      'Votre facture de septembre est disponible',
      26 * hourMs,
      ps([
        'Bonjour,',
        'Votre facture pour la période du 1er au 30 septembre est disponible.',
        'Montant : <strong>0,00 €</strong> — plan Free. Aucune action requise de votre part.',
      ]),
      labels: ['factures'],
      atts: [att('facture-septembre.pdf', 'PDF', '92 Ko')],
    ),
  );

  add(
    mk(
      'inbox',
      kofi,
      'Idées pour la landing page',
      30 * hourMs,
      ps([
        'Salut,',
        'Je t’ai mis trois maquettes en pièce jointe pour la landing du bot. Dis-moi laquelle tu préfères avant mardi, que j’attaque l’intégration.',
        'La 2 est ma préférée (le titre en condensé passe vraiment bien).',
        'Kofi',
      ]),
      unread: true,
      labels: ['equipe'],
      flags: ['needs'],
      atts: [
        att('landing-A.png', 'PNG', '480 Ko'),
        att('landing-B.png', 'PNG', '512 Ko'),
        att('landing-C.png', 'PNG', '455 Ko'),
      ],
    ),
  );

  add(
    mk(
      'inbox',
      vercel,
      'Déploiement réussi — kernel-mail',
      50 * hourMs,
      ps([
        'Votre projet <strong>kernel-mail</strong> a été déployé avec succès.',
        'Production : kernel-mail.vercel.app · Durée : 41 s · Commit : 3f9c2a1',
      ]),
    ),
  );

  add(
    mk(
      'inbox',
      sand,
      'Merci pour ton retour !',
      54 * hourMs,
      ps([
        'C’était super clair, je transmets au reste de l’équipe dès demain matin.',
        'Bonne soirée !',
        'Sandrine',
      ]),
      labels: ['equipe'],
    ),
  );

  add(
    mk(
      'inbox',
      awa,
      'Planning de la semaine',
      80 * hourMs,
      ps([
        'Voici le planning :',
        '• Lundi : tests terrain à Cotonou<br>• Mercredi : point d’avancement<br>• Jeudi : démo commerçant (15 h)<br>• Vendredi : rétrospective',
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
      'Nouvelle connexion détectée',
      3 * dayMs + 2 * hourMs,
      ps([
        'Une nouvelle connexion à votre compte a été détectée depuis <strong>Cotonou, Bénin</strong> (Chrome sur Linux).',
        'Si c’était vous, aucune action n’est nécessaire.',
      ]),
    ),
  );

  add(
    mk(
      'inbox',
      yao,
      'Devis hébergement boutique',
      4 * dayMs,
      ps([
        'Bonjour Noah,',
        'Suite à notre appel, voici le devis pour l’hébergement de la boutique : 12 000 FCFA / mois, sauvegardes incluses. J’attends ton retour pour lancer la mise en place.',
        'Yao',
      ]),
      flags: ['waiting'],
      labels: ['factures'],
      atts: [att('devis-hebergement.pdf', 'PDF', '230 Ko')],
    ),
  );

  add(
    mk(
      'inbox',
      linda,
      'Compte rendu de la réunion du 8',
      5 * dayMs + 3 * hourMs,
      ps([
        'Salut à tous,',
        'Voici le compte rendu : priorité à la démo, relecture des specs d’ici vendredi, et choix de l’hébergeur à trancher.',
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
      'Bienvenue sur KERNEL',
      9 * dayMs,
      ps([
        'Bienvenue Noah !',
        'Ton espace est prêt. Dépose ton premier projet, gagne des points de score et débloque tes premiers badges.',
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
      'Re : Idées pour la landing page',
      29 * hourMs,
      ps([
        'Merci Kofi, je regarde ça ce soir et je te dis laquelle je retiens.',
        'Noah',
      ]),
      to: [kofi],
    ),
  );
  add(
    mk(
      'sent',
      me,
      'Proposition de démo — Bot WhatsApp stock',
      2 * dayMs + 4 * hourMs,
      ps([
        'Bonjour Awa,',
        'Je te propose une démo jeudi à 15 h avec le commerçant. Dis-moi si ça te convient.',
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
      'Candidature — lettre de motivation',
      5 * hourMs,
      ps([
        'Madame, Monsieur,',
        'Passionné par le développement et l’impact local, je souhaite rejoindre la prochaine session…',
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
      'Contrat hébergement — confirmation',
      14 * dayMs,
      ps([
        'Votre contrat d’hébergement est actif. Retrouvez votre facture dans votre espace client.',
      ]),
      labels: ['factures'],
      atts: [att('contrat.pdf', 'PDF', '310 Ko')],
    ),
  );
  add(
    mk(
      'archive',
      kernel,
      'Ton badge « Premier pas » est débloqué',
      12 * dayMs,
      ps([
        'Félicitations ! Tu viens de débloquer ton premier badge sur KERNEL.',
      ]),
    ),
  );

  // Reportés
  add(
    mk(
      'snoozed',
      ovh,
      'Relance facture mensuelle',
      6 * dayMs,
      ps(['Nous n’avons pas reçu le règlement de votre facture mensuelle.']),
      snooze: 'demain 08:00',
      labels: ['factures'],
    ),
  );

  // Spam
  add(
    mk(
      'spam',
      const Person('Promo Crypto', 'win@crypto-gains.xyz'),
      'Gagnez 1 000 € par jour sans effort !!!',
      7 * hourMs,
      ps(['Cliquez ici pour découvrir notre méthode secrète…']),
      unread: true,
    ),
  );
  add(
    mk(
      'spam',
      const Person('Livraison Express', 'colis@dhl-track.top'),
      'Votre colis est bloqué — action requise',
      2 * dayMs,
      ps(['Payez 1,99 € de frais pour débloquer votre colis.']),
      unread: true,
    ),
  );
  add(
    mk(
      'spam',
      const Person('Offre Exclusive', 'deal@mega-promo.click'),
      '-90 % sur tout, aujourd’hui seulement',
      6 * dayMs,
      ps(['Dernière chance, ne manquez pas cette offre incroyable.']),
    ),
  );

  // Corbeille
  add(
    mk(
      'trash',
      const Person('Newsletter Tech', 'news@techweekly.io'),
      'Les 10 frameworks JS à connaître',
      8 * dayMs,
      ps(['Cette semaine : encore un nouveau framework…']),
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

final incoming = <IncomingDef>[
  IncomingDef(
    awa,
    'Démo confirmée jeudi 15 h',
    ['projets'],
    ['needs'],
    ps([
      'Bonne nouvelle : le commerçant est partant pour jeudi à 15 h !',
      'Il aimerait voir la commande de stock par SMS en direct. Tu peux prévoir un scénario court ?',
      'Awa',
    ]),
  ),
  IncomingDef(
    gh,
    '[stock-bot] Nouvelle issue #14 : relance SMS en double',
    ['equipe'],
    [],
    ps([
      '<strong>kofi-a</strong> a ouvert l’issue <em>#14</em>.',
      'Les relances SMS partent parfois deux fois quand le stock passe sous le seuil pendant la synchro.',
    ]),
  ),
  IncomingDef(
    epi,
    'Entretien : choisis ton créneau',
    ['candidatures'],
    ['needs'],
    ps([
      'Bonjour Noah,',
      'Ton dossier a retenu notre attention ! Choisis un créneau d’entretien parmi ceux proposés dans ton espace avant vendredi.',
      'Les admissions',
    ]),
  ),
  IncomingDef(
    kofi,
    'Tu as vu le nouveau logo ?',
    ['equipe'],
    [],
    ps([
      'Regarde ce que j’ai sorti ce matin, je trouve que le carré orange fonctionne vraiment bien.',
      'Kofi',
    ]),
    [Attachment('logo-final.png', 'PNG', '128 Ko')],
  ),
  IncomingDef(
    supa,
    'Votre projet approche de la limite gratuite',
    ['factures'],
    [],
    ps([
      'Votre projet <strong>stock-bot</strong> a utilisé 82 % de son quota mensuel de requêtes.',
    ]),
  ),
  IncomingDef(
    vercel,
    'Build échoué — kernel-mail',
    [],
    ['needs'],
    ps([
      'Le dernier déploiement de <strong>kernel-mail</strong> a échoué.',
      '<code>Error: Cannot find module "./theme"</code>',
    ]),
  ),
];

final replies = <String>[
  ps([
    'Parfait, merci pour ta réponse !',
    'Je regarde ça et je reviens vers toi dans la journée.',
  ]),
  ps([
    'Reçu 👍',
    'C’est exactement ce qu’il me fallait. On en reparle demain à la première heure ?',
  ]),
  ps([
    'Merci beaucoup, c’est très clair.',
    'Je transmets à l’équipe et on te confirme rapidement.',
  ]),
  ps([
    'Top ! Juste une question : peux-tu m’envoyer la version finale en pièce jointe quand elle est prête ?',
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
        'Alerte de sécurité',
        40 * minMs,
        ps([
          'Une nouvelle connexion à votre compte Google a été détectée sur <strong>Linux</strong>.',
          'Si c’était vous, vous n’avez rien à faire.',
        ]),
        unread: true,
      ),
    );
    add(
      mk(
        'inbox',
        awa,
        'Photos du déplacement à Cotonou',
        5 * hourMs,
        ps([
          'Salut Noah,',
          'Je t’envoie les photos des tests terrain. Dis-moi lesquelles tu veux garder pour la présentation.',
          'Awa',
        ]),
        unread: true,
        flags: ['needs'],
        atts: [
          att('terrain-01.png', 'PNG', '1,2 Mo'),
          att('terrain-02.png', 'PNG', '980 Ko'),
        ],
      ),
    );
    add(
      mk(
        'inbox',
        const Person('Newsletter Tech', 'news@techweekly.io'),
        'Cette semaine : Flutter 3.47',
        30 * hourMs,
        ps([
          'Les nouveautés de la semaine : performances, outils et ressources à ne pas manquer.',
        ]),
        labels: ['projets'],
      ),
    );
    add(
      mk(
        'inbox',
        yao,
        'Photo de famille',
        3 * dayMs,
        ps(['Voici la photo de dimanche, à bientôt !', 'Yao']),
        star: true,
      ),
    );
    add(
      mk(
        'sent',
        me,
        'Re : Photos du déplacement à Cotonou',
        4 * hourMs,
        ps(['Merci Awa, je trie ça ce soir.', 'Noah']),
        to: [awa],
      ),
    );
  } else if (a.id == 'a3') {
    add(
      mk(
        'inbox',
        yao,
        'Commande de 40 unités',
        2 * hourMs,
        ps([
          'Bonjour,',
          'Nous souhaiterions passer commande de <strong>40 unités</strong> pour la boutique de Cotonou. Pouvez-vous confirmer le délai de livraison ?',
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
        'Partenariat Dakar',
        26 * hourMs,
        ps([
          'Bonjour,',
          'Notre laboratoire serait ravi de tester le bot WhatsApp de stock. Pouvons-nous planifier un appel cette semaine ?',
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
        'Votre projet stock-bot est en pause',
        4 * dayMs,
        ps([
          'Votre projet <strong>stock-bot</strong> a été mis en pause après 7 jours d’inactivité.',
        ]),
        labels: ['equipe'],
      ),
    );
    add(
      mk(
        'sent',
        me,
        'Devis hébergement — réponse',
        3 * dayMs,
        ps(['Bonjour Yao, voici notre devis détaillé.', 'L’équipe StockBot']),
        to: [yao],
      ),
    );
  } else {
    final first = a.name.trim().split(RegExp(r'\s+')).first;
    add(
      mk(
        'inbox',
        const Person('Équipe MIRA', 'equipe@mira.app'),
        'Bienvenue sur MIRA',
        5 * minMs,
        ps([
          'Bonjour $first,',
          'Votre compte <strong>${a.email}</strong> est maintenant connecté. Retrouvez ici tous vos messages, libellés et brouillons.',
          'Bonne découverte !',
        ]),
        unread: true,
        star: true,
      ),
    );
    add(
      mk(
        'inbox',
        const Person('Équipe MIRA', 'equipe@mira.app'),
        'Astuce : raccourcis clavier',
        2 * hourMs,
        ps([
          'Appuyez sur <strong>?</strong> pour afficher tous les raccourcis, et sur <strong>c</strong> pour écrire un nouveau message.',
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

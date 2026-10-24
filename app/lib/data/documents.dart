import 'package:intl/intl.dart';

import '../i18n/i18n.dart';
import 'models.dart';

/// Contenu fictif des documents joints (aucun vrai fichier n'existe dans la démo).
sealed class DocBlock {
  const DocBlock();
}

class DocTitle extends DocBlock {
  final String title;
  final String? sub;
  const DocTitle(this.title, [this.sub]);
}

class DocHeading extends DocBlock {
  final String text;
  const DocHeading(this.text);
}

class DocPara extends DocBlock {
  final String text;
  const DocPara(this.text);
}

class DocBullets extends DocBlock {
  final List<String> items;
  const DocBullets(this.items);
}

class DocMeta extends DocBlock {
  final List<(String, String)> rows;
  const DocMeta(this.rows);
}

class DocTable extends DocBlock {
  final List<String> head;
  final List<List<String>> rows;
  final List<(String, String)> totals;
  const DocTable(this.head, this.rows, {this.totals = const []});
}

class DocSign extends DocBlock {
  final String name, role;
  const DocSign(this.name, this.role);
}

enum AttKind { pdf, image, text, other }

const _imageExt = {'PNG', 'JPG', 'JPEG', 'GIF', 'WEBP', 'BMP'};
const _textExt = {'TXT', 'MD', 'CSV', 'JSON', 'LOG'};

AttKind kindOf(Attachment a) {
  final t = a.t.toUpperCase();
  if (_imageExt.contains(t)) return AttKind.image;
  if (_textExt.contains(t)) return AttKind.text;
  if (t == 'PDF') return AttKind.pdf;
  return AttKind.other;
}

bool isImageAtt(Attachment a) => kindOf(a) == AttKind.image;
bool isTextAtt(Attachment a) => kindOf(a) == AttKind.text;

String _date(int daysAgo) => DateFormat(
  'd MMMM y',
  appLang.dateLocale,
).format(DateTime.now().subtract(Duration(days: daysAgo)));

/// Pages d'un document PDF fictif, selon son nom.
List<List<DocBlock>> documentPages(Attachment a) {
  final n = a.n.toLowerCase();
  if (n.contains('plan-validation')) {
    return [
      [
        DocTitle(
          t('Validation du plan de projet'),
          t('KERNEL · Programme de réalisation'),
        ),
        DocMeta([
          (t('Projet'), 'Bot WhatsApp stock'),
          (t('Porteur'), 'Noah Toffa'),
          ('Date', _date(0)),
          (t('Statut'), t('Plan validé')),
        ]),
        DocHeading(t('1. Résumé')),
        DocPara(
          t(
            'Le plan a été relu avant le lancement de la réalisation. Le problème est clairement posé : les commerçants perdent des ventes faute de suivi de stock fiable. La solution proposée, un bot WhatsApp répondant aussi par SMS, est réaliste et son périmètre tient en quelques semaines.',
          ),
        ),
        DocHeading(t('2. Critères de relecture')),
        DocBullets([
          t('Problème clair et mesurable — validé'),
          t('Solution réaliste avec les moyens disponibles — validé'),
          t('Périmètre limité à quelques semaines — validé'),
          t('Plan de test terrain à Cotonou — validé'),
        ]),
        DocHeading(t('3. Prochaines étapes')),
        DocPara(
          t(
            'Vous pouvez passer à la réalisation. Le projet terminé sera validé une seconde fois avant publication. Deux crédits sont réservés pour votre projet.',
          ),
        ),
        DocSign(t('L’équipe KERNEL'), t('Relecture des plans')),
      ],
    ];
  }
  if (n.contains('facture')) {
    return [
      [
        DocTitle(t('Facture'), t('Supabase · N° 2026-09-0412')),
        DocMeta([
          (t('Période'), t('1er au 30 septembre')),
          (t('Émise le'), _date(1)),
          (t('Client'), 'Noah Toffa'),
          ('Plan', 'Free'),
        ]),
        DocTable(
          [t('Désignation'), t('Qté'), 'Prix', t('Total')],
          [
            [t('Projet stock-bot — plan Free'), '1', '0,00 €', '0,00 €'],
            [t('Base de données (500 Mo inclus)'), '1', '0,00 €', '0,00 €'],
            [t('Requêtes API (2 M inclus)'), '1', '0,00 €', '0,00 €'],
          ],
          totals: [
            (t('Sous-total'), '0,00 €'),
            ('TVA', '0,00 €'),
            (t('Total à payer'), '0,00 €'),
          ],
        ),
        DocPara(
          t('Aucune action requise de votre part. Merci d’utiliser Supabase.'),
        ),
      ],
    ];
  }
  if (n.contains('devis')) {
    return [
      [
        DocTitle(
          t('Devis — Hébergement de la boutique'),
          t('Cotonou Market · Réf. DV-0187'),
        ),
        DocMeta([
          ('Date', _date(4)),
          (t('Validité'), t('30 jours')),
          (t('Destinataire'), 'Noah Toffa'),
          (t('Émetteur'), 'Yao Kpadonou'),
        ]),
        DocTable(
          [t('Désignation'), t('Qté'), 'Prix', t('Total')],
          [
            [
              t('Hébergement mensuel de la boutique'),
              '1',
              '12 000 FCFA',
              '12 000 FCFA',
            ],
            [t('Sauvegardes quotidiennes'), '1', t('incluses'), '0 FCFA'],
            [t('Mise en place initiale'), '1', t('offerte'), '0 FCFA'],
          ],
          totals: [(t('Total mensuel'), '12 000 FCFA')],
        ),
        DocPara(
          t(
            'Dans l’attente de votre retour pour lancer la mise en place de la boutique.',
          ),
        ),
        DocSign('Yao Kpadonou', 'Cotonou Market'),
      ],
    ];
  }
  if (n.contains('contrat')) {
    return [
      [
        DocTitle(
          t('Contrat d’hébergement'),
          t('OVHcloud · Confirmation de souscription'),
        ),
        DocMeta([
          (t('Contrat'), 'HB-5520-77'),
          (t('Début'), _date(14)),
          (t('Durée'), t('12 mois reconductibles')),
        ]),
        DocHeading(t('Article 1 — Objet')),
        DocPara(
          t(
            'Le présent contrat définit les conditions d’hébergement des services web du client sur l’infrastructure OVHcloud.',
          ),
        ),
        DocHeading(t('Article 2 — Disponibilité')),
        DocPara(
          t(
            'Le prestataire s’engage sur une disponibilité mensuelle de 99,9 %, hors maintenances planifiées annoncées 48 heures à l’avance.',
          ),
        ),
      ],
      [
        DocHeading(t('Article 3 — Facturation')),
        DocPara(
          t(
            'Les factures sont émises mensuellement et disponibles dans l’espace client. Tout retard de règlement donne lieu à une relance.',
          ),
        ),
        DocHeading(t('Article 4 — Résiliation')),
        DocPara(
          t(
            'Chaque partie peut résilier le contrat par écrit avec un préavis de 30 jours.',
          ),
        ),
        DocSign('OVHcloud', t('Service clients')),
      ],
    ];
  }
  final title = a.n
      .replaceAll(RegExp(r'\.[^.]+$'), '')
      .replaceAll(RegExp(r'[-_]+'), ' ');
  return [
    [
      DocTitle(
        title[0].toUpperCase() + title.substring(1),
        t('Document joint · {0}', [a.s]),
      ),
      DocMeta([(t('Fichier'), a.n), ('Type', a.t), (t('Taille'), a.s)]),
      DocPara(
        t(
          'Ceci est un aperçu fictif : la démo ne contient pas le fichier réel. Dans une vraie boîte mail, le contenu du document s’afficherait ici.',
        ),
      ),
    ],
  ];
}

/// Variante de maquette pour une image jointe fictive.
int mockImageVariant(Attachment a) {
  final n = a.n.toLowerCase();
  if (n.contains('logo')) return 9;
  if (n.endsWith('a.png')) return 0;
  if (n.endsWith('b.png')) return 1;
  if (n.endsWith('c.png')) return 2;
  return 3;
}

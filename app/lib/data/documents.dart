// i18n:ignore-file — contenu fictif de la démonstration, volontairement en français.
import 'package:intl/intl.dart';

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
  'fr_FR',
).format(DateTime.now().subtract(Duration(days: daysAgo)));

/// Pages d'un document PDF fictif, selon son nom.
List<List<DocBlock>> documentPages(Attachment a) {
  final n = a.n.toLowerCase();
  if (n.contains('plan-validation')) {
    return [
      [
        const DocTitle(
          'Validation du plan de projet',
          'KERNEL · Programme de réalisation',
        ),
        DocMeta([
          ('Projet', 'Bot WhatsApp stock'),
          ('Porteur', 'Noah Toffa'),
          ('Date', _date(0)),
          ('Statut', 'Plan validé'),
        ]),
        const DocHeading('1. Résumé'),
        const DocPara(
          'Le plan a été relu avant le lancement de la réalisation. Le problème est clairement posé : les commerçants perdent des ventes faute de suivi de stock fiable. La solution proposée, un bot WhatsApp répondant aussi par SMS, est réaliste et son périmètre tient en quelques semaines.',
        ),
        const DocHeading('2. Critères de relecture'),
        const DocBullets([
          'Problème clair et mesurable — validé',
          'Solution réaliste avec les moyens disponibles — validé',
          'Périmètre limité à quelques semaines — validé',
          'Plan de test terrain à Cotonou — validé',
        ]),
        const DocHeading('3. Prochaines étapes'),
        const DocPara(
          'Vous pouvez passer à la réalisation. Le projet terminé sera validé une seconde fois avant publication. Deux crédits sont réservés pour votre projet.',
        ),
        const DocSign('L’équipe KERNEL', 'Relecture des plans'),
      ],
    ];
  }
  if (n.contains('facture')) {
    return [
      [
        const DocTitle('Facture', 'Supabase · N° 2026-09-0412'),
        DocMeta([
          ('Période', '1er au 30 septembre'),
          ('Émise le', _date(1)),
          ('Client', 'Noah Toffa'),
          ('Plan', 'Free'),
        ]),
        const DocTable(
          ['Désignation', 'Qté', 'Prix', 'Total'],
          [
            ['Projet stock-bot — plan Free', '1', '0,00 €', '0,00 €'],
            ['Base de données (500 Mo inclus)', '1', '0,00 €', '0,00 €'],
            ['Requêtes API (2 M inclus)', '1', '0,00 €', '0,00 €'],
          ],
          totals: [
            ('Sous-total', '0,00 €'),
            ('TVA', '0,00 €'),
            ('Total à payer', '0,00 €'),
          ],
        ),
        const DocPara(
          'Aucune action requise de votre part. Merci d’utiliser Supabase.',
        ),
      ],
    ];
  }
  if (n.contains('devis')) {
    return [
      [
        const DocTitle(
          'Devis — Hébergement de la boutique',
          'Cotonou Market · Réf. DV-0187',
        ),
        DocMeta([
          ('Date', _date(4)),
          ('Validité', '30 jours'),
          ('Destinataire', 'Noah Toffa'),
          ('Émetteur', 'Yao Kpadonou'),
        ]),
        const DocTable(
          ['Désignation', 'Qté', 'Prix', 'Total'],
          [
            [
              'Hébergement mensuel de la boutique',
              '1',
              '12 000 FCFA',
              '12 000 FCFA',
            ],
            ['Sauvegardes quotidiennes', '1', 'incluses', '0 FCFA'],
            ['Mise en place initiale', '1', 'offerte', '0 FCFA'],
          ],
          totals: [('Total mensuel', '12 000 FCFA')],
        ),
        const DocPara(
          'Dans l’attente de votre retour pour lancer la mise en place de la boutique.',
        ),
        const DocSign('Yao Kpadonou', 'Cotonou Market'),
      ],
    ];
  }
  if (n.contains('contrat')) {
    return [
      [
        const DocTitle(
          'Contrat d’hébergement',
          'OVHcloud · Confirmation de souscription',
        ),
        DocMeta([
          ('Contrat', 'HB-5520-77'),
          ('Début', _date(14)),
          ('Durée', '12 mois reconductibles'),
        ]),
        const DocHeading('Article 1 — Objet'),
        const DocPara(
          'Le présent contrat définit les conditions d’hébergement des services web du client sur l’infrastructure OVHcloud.',
        ),
        const DocHeading('Article 2 — Disponibilité'),
        const DocPara(
          'Le prestataire s’engage sur une disponibilité mensuelle de 99,9 %, hors maintenances planifiées annoncées 48 heures à l’avance.',
        ),
      ],
      [
        const DocHeading('Article 3 — Facturation'),
        const DocPara(
          'Les factures sont émises mensuellement et disponibles dans l’espace client. Tout retard de règlement donne lieu à une relance.',
        ),
        const DocHeading('Article 4 — Résiliation'),
        const DocPara(
          'Chaque partie peut résilier le contrat par écrit avec un préavis de 30 jours.',
        ),
        const DocSign('OVHcloud', 'Service clients'),
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
        'Document joint · ${a.s}',
      ),
      DocMeta([('Fichier', a.n), ('Type', a.t), ('Taille', a.s)]),
      const DocPara(
        'Ceci est un aperçu fictif : la démo ne contient pas le fichier réel. Dans une vraie boîte mail, le contenu du document s’afficherait ici.',
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

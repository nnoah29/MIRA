import 'package:flutter/material.dart';
import 'package:provider/provider.dart';

import '../data/format.dart';
import '../i18n/i18n.dart';
import '../state/contacts.dart';
import '../state/store.dart';
import '../theme/app_theme.dart';
import 'account_form_widgets.dart';
import 'widgets.dart';

/// Historique des destinataires : un nom au choix pour chaque adresse.
class ContactsSettings extends StatefulWidget {
  const ContactsSettings({super.key});

  @override
  State<ContactsSettings> createState() => _ContactsSettingsState();
}

class _ContactsSettingsState extends State<ContactsSettings> {
  final search = TextEditingController();

  @override
  void dispose() {
    search.dispose();
    super.dispose();
  }

  bool _matches(ContactEntry e) {
    final q = search.text.trim().toLowerCase();
    return q.isEmpty || e.email.contains(q) || e.name.toLowerCase().contains(q);
  }

  @override
  Widget build(BuildContext context) {
    final s = context.watch<MailStore>();
    final p = context.pal;
    final all = s.contactEntries();
    final shown = all.where(_matches).toList();
    return Padding(
      padding: const EdgeInsets.all(20),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          Text(
            t('Contacts'),
            style: Fonts.sans(size: 15, w: FontWeight.w500, color: p.text),
          ),
          const SizedBox(height: 2),
          Text(
            t(
              'Les personnes à qui tu as écrit, du plus récent au plus ancien. Le nom choisi ici est proposé à la saisie des destinataires.',
            ),
            style: Fonts.sans(size: 13.5, color: p.dim),
          ),
          const SizedBox(height: 14),
          FormField2(
            fieldKey: const ValueKey('contacts-search'),
            label: '',
            hint: t('Rechercher un contact'),
            controller: search,
            onChanged: (_) => setState(() {}),
          ),
          const SizedBox(height: 14),
          if (shown.isEmpty)
            Text(
              all.isEmpty
                  ? t('Aucun destinataire pour le moment.')
                  : t('Aucun contact ne correspond.'),
              style: Fonts.sans(size: 13.5, color: p.dim),
            ),
          for (final e in shown)
            _ContactRow(key: ValueKey('contact-${e.email}'), entry: e),
        ],
      ),
    );
  }
}

class _ContactRow extends StatefulWidget {
  final ContactEntry entry;
  const _ContactRow({super.key, required this.entry});

  @override
  State<_ContactRow> createState() => _ContactRowState();
}

class _ContactRowState extends State<_ContactRow> {
  late final name = TextEditingController(text: widget.entry.name);

  @override
  void dispose() {
    name.dispose();
    super.dispose();
  }

  String get _typed => name.text.trim();
  bool get _changed => _typed != widget.entry.name;

  void _save(MailStore s) {
    final seen = widget.entry.seen;
    s.setContactName(widget.entry.email, _typed == seen ? '' : _typed);
    s.toast(t('Nom enregistré'));
    setState(() {});
  }

  @override
  Widget build(BuildContext context) {
    final s = context.read<MailStore>();
    final e = widget.entry;
    final mobile = MediaQuery.sizeOf(context).width <= 760;
    return Padding(
      padding: EdgeInsets.only(bottom: mobile ? 18 : 10),
      child: mobile ? _stacked(s, e) : _inline(s, e),
    );
  }

  /// Ligne sur une seule rangée (grand écran).
  Widget _inline(MailStore s, ContactEntry e) => Row(
    children: [
      Avatar(_typed.isEmpty ? e.email : _typed, size: 36),
      const SizedBox(width: 14),
      SizedBox(width: 230, child: _nameField(e)),
      const SizedBox(width: 14),
      Expanded(child: _info(e)),
      const SizedBox(width: 12),
      _deleteButton(s, e),
      const SizedBox(width: 8),
      _saveButton(s, e),
    ],
  );

  /// Ligne sur trois niveaux : nom et corbeille, détails, enregistrement (mobile).
  Widget _stacked(MailStore s, ContactEntry e) => Column(
    crossAxisAlignment: CrossAxisAlignment.start,
    children: [
      Row(
        children: [
          Avatar(_typed.isEmpty ? e.email : _typed, size: 36),
          const SizedBox(width: 14),
          Expanded(child: _nameField(e)),
          const SizedBox(width: 8),
          _deleteButton(s, e),
        ],
      ),
      const SizedBox(height: 8),
      Padding(padding: const EdgeInsets.only(left: 50), child: _info(e)),
      const SizedBox(height: 8),
      Padding(
        padding: const EdgeInsets.only(left: 50),
        child: _saveButton(s, e),
      ),
    ],
  );

  /// Champ de saisie du nom affiché du contact.
  Widget _nameField(ContactEntry e) => FormField2(
    fieldKey: ValueKey('contact-name-${e.email}'),
    label: '',
    hint: e.email,
    controller: name,
    onChanged: (_) => setState(() {}),
  );

  /// Adresse du contact, nombre de messages et date du dernier.
  Widget _info(ContactEntry e) => Column(
    crossAxisAlignment: CrossAxisAlignment.start,
    children: [
      Text(
        e.email,
        maxLines: 1,
        overflow: TextOverflow.ellipsis,
        style: Fonts.sans(size: 14, color: context.pal.text),
      ),
      Mono(
        '${t('{0} messages', ['${e.count}'])} · ${rowTime(e.last)}',
        size: 10.5,
        height: null,
        color: context.pal.dim,
      ),
    ],
  );

  /// Bouton d'oubli du contact.
  Widget _deleteButton(MailStore s, ContactEntry e) => IBtn(
    'trash',
    key: ValueKey('contact-delete-${e.email}'),
    tip: t('Supprimer ce contact'),
    onTap: () {
      s.forgetContact(e.email);
      s.toast(t('Contact supprimé'));
    },
  );

  /// Bouton d'enregistrement du nom, actif seulement s'il a changé.
  Widget _saveButton(MailStore s, ContactEntry e) => Btn(
    key: ValueKey('contact-save-${e.email}'),
    label: t('Enregistrer'),
    onTap: _changed ? () => _save(s) : null,
  );
}

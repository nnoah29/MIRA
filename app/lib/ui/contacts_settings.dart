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
    final p = context.pal;
    final e = widget.entry;
    return Padding(
      padding: const EdgeInsets.only(bottom: 10),
      child: Row(
        children: [
          Avatar(_typed.isEmpty ? e.email : _typed, size: 36),
          const SizedBox(width: 14),
          SizedBox(
            width: 230,
            child: FormField2(
              fieldKey: ValueKey('contact-name-${e.email}'),
              label: '',
              hint: e.email,
              controller: name,
              onChanged: (_) => setState(() {}),
            ),
          ),
          const SizedBox(width: 14),
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(
                  e.email,
                  maxLines: 1,
                  overflow: TextOverflow.ellipsis,
                  style: Fonts.sans(size: 14, color: p.text),
                ),
                Mono(
                  '${t('{0} messages', ['${e.count}'])} · ${rowTime(e.last)}',
                  size: 10.5,
                  height: null,
                  color: p.dim,
                ),
              ],
            ),
          ),
          const SizedBox(width: 12),
          Btn(
            key: ValueKey('contact-save-${e.email}'),
            label: t('Enregistrer'),
            onTap: _changed ? () => _save(s) : null,
          ),
        ],
      ),
    );
  }
}

import 'package:flutter/material.dart';
import 'package:provider/provider.dart';

import '../../data/models.dart';
import '../../i18n/i18n.dart';
import '../../state/store.dart';
import '../../theme/app_theme.dart';
import '../add_account_panel.dart';
import '../widgets.dart';
import 'onboarding_controller.dart';
import 'step_layout.dart';

/// Page 4 : ajout du premier compte avec le formulaire existant, ou récapitulatif des comptes déjà là.
class AccountStep extends StatefulWidget {
  final OnboardingController controller;
  const AccountStep({super.key, required this.controller});

  @override
  State<AccountStep> createState() => _AccountStepState();
}

class _AccountStepState extends State<AccountStep> {
  late bool _showForm = _real(context.read<MailStore>()).isEmpty;

  List<Account> _real(MailStore s) => [
    for (final a in s.accounts)
      if (a.id != 'none') a,
  ];

  Future<void> _added(String id) async {
    final store = context.read<MailStore>();
    await store.accountAdded(id);
    if (!mounted) return;
    setState(() => _showForm = false);
    widget.controller.next();
  }

  Widget _form() => Column(
    crossAxisAlignment: CrossAxisAlignment.start,
    children: [
      Btn(
        key: const ValueKey('onb-account-later'),
        label: t('Plus tard'),
        onTap: widget.controller.next,
      ),
      const SizedBox(height: 16),
      AddAccountPanel(
        key: const ValueKey('onb-account-panel'),
        embedded: true,
        onAdded: _added,
      ),
    ],
  );

  Widget _summary(Pal p, List<Account> accounts) => Column(
    crossAxisAlignment: CrossAxisAlignment.start,
    children: [
      Text(
        t('Comptes déjà ajoutés : {0}', ['${accounts.length}']),
        style: Fonts.sans(size: 17, w: FontWeight.w500, color: p.text),
      ),
      const SizedBox(height: 8),
      for (final a in accounts)
        Text(a.email, style: Fonts.sans(size: 15, color: p.dim)),
      const SizedBox(height: 20),
      Btn(
        key: const ValueKey('onb-account-add-another'),
        icon: 'plus',
        label: t('Ajouter un autre compte'),
        onTap: () => setState(() => _showForm = true),
      ),
    ],
  );

  @override
  Widget build(BuildContext context) {
    final store = context.watch<MailStore>();
    final p = context.pal;
    final accounts = _real(store);
    return StepLayout(
      title: t('TON PREMIER COMPTE'),
      lead: t(
        'Ajoute ta boîte mail : Gmail, Outlook, Yahoo ou n’importe quel serveur IMAP.',
      ),
      children: [_showForm ? _form() : _summary(p, accounts)],
    );
  }
}

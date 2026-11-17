//! Champ « Ce que tu dois savoir de moi » : un texte libre, ajouté à la consigne de l'assistant à chaque discussion.

import 'package:flutter/material.dart';

import '../engine/ai_about_admin.dart';
import '../i18n/i18n.dart';
import '../state/about_me_state.dart';
import '../theme/app_theme.dart';
import 'widgets.dart';

/// Zone de saisie, compteur, avertissement d'envoi et bouton d'enregistrement ; chaque changement passe par [state].
class AboutMeSection extends StatefulWidget {
  final AboutMeState state;
  const AboutMeSection({super.key, required this.state});

  @override
  State<AboutMeSection> createState() => _AboutMeSectionState();
}

class _AboutMeSectionState extends State<AboutMeSection> {
  late final TextEditingController text = TextEditingController(
    text: widget.state.draft,
  );

  AboutMeState get s => widget.state;

  @override
  void initState() {
    super.initState();
    s.addListener(_sync);
  }

  /// Recopie le texte lu ou enregistré dans la zone, sans écraser une frappe en cours.
  void _sync() {
    if (!mounted) return;
    if (text.text != s.draft && !s.dirty) text.text = s.draft;
    setState(() {});
  }

  @override
  void dispose() {
    s.removeListener(_sync);
    text.dispose();
    super.dispose();
  }

  Widget _status(Pal p) {
    if (s.error != null) {
      return Text(
        s.error!,
        key: const ValueKey('about-me-error'),
        style: Fonts.sans(size: 13.5, color: p.accent),
      );
    }
    if (s.justSaved) {
      return Mono(
        t('Enregistré'),
        key: const ValueKey('about-me-saved'),
        color: p.accent,
        height: null,
      );
    }
    return const SizedBox.shrink();
  }

  @override
  Widget build(BuildContext context) {
    final p = context.pal;
    final count = s.draft.trim().runes.length;
    return Padding(
      key: const ValueKey('about-me-section'),
      padding: const EdgeInsets.only(top: 16),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text(
            t('Ce que tu dois savoir de moi'),
            style: Fonts.sans(size: 14, w: FontWeight.w500, color: p.text),
          ),
          const SizedBox(height: 2),
          Text(
            t(
              'Écris ici ce que l’assistant doit toujours savoir de toi : ton métier, comment tu signes, tes projets, ce qu’il doit éviter.',
            ),
            style: Fonts.sans(size: 13.5, color: p.dim),
          ),
          const SizedBox(height: 10),
          TextField(
            key: const ValueKey('about-me-field'),
            controller: text,
            minLines: 4,
            maxLines: 10,
            onChanged: s.edit,
            style: Fonts.sans(size: 14, color: p.text),
            cursorColor: p.accent,
            decoration: InputDecoration(
              hintText: t(
                'Je suis médecin à Cotonou. Je signe toujours « Noah ».',
              ),
            ),
          ),
          const SizedBox(height: 6),
          Text(
            '$count / $aboutMaxChars',
            key: const ValueKey('about-me-count'),
            style: Fonts.sans(size: 12.5, color: s.valid ? p.dim : p.accent),
          ),
          const SizedBox(height: 4),
          Text(
            t(
              'Ce texte est envoyé au service d’IA que tu utilises, à chaque discussion. Avec le modèle local, rien ne sort de ton ordinateur.',
            ),
            key: const ValueKey('about-me-note'),
            style: Fonts.sans(size: 13.5, color: p.dim),
          ),
          const SizedBox(height: 10),
          Wrap(
            spacing: 12,
            runSpacing: 8,
            crossAxisAlignment: WrapCrossAlignment.center,
            children: [
              Btn(
                key: const ValueKey('about-me-save'),
                label: t('Enregistrer'),
                go: true,
                onTap: s.dirty && s.valid && !s.saving ? s.save : null,
              ),
              _status(p),
            ],
          ),
        ],
      ),
    );
  }
}

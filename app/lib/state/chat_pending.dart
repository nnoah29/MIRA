//! Question en attente d'un accord de l'utilisateur (consentement ou type de contenu) et codes d'erreur qui l'annoncent.

import '../engine/ai_models.dart';
import 'chat_context.dart';

/// Code d'erreur du moteur : le service n'est pas autorisé à recevoir ce type de contenu.
const contentNotAllowedCode = 'incompatible_state.ai_content_not_allowed';

/// Question en attente d'un accord ; [stage] compte les renvois (0 : premier envoi, 1 : après le consentement, 2 : après l'autorisation du contenu) pour ne jamais boucler.
class ChatPending {
  final AiConnectionInfo connection;
  final String question;
  final ChatContextInput input;
  final String providerName;
  final int stage;
  const ChatPending(
    this.connection,
    this.question,
    this.input,
    this.providerName, [
    this.stage = 0,
  ]);

  List<String> get kinds => input.hasMail ? const [aiKindMail] : const [];
}

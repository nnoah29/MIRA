final _automated = RegExp(
  r'^(no[-_.]?reply|do[-_.]?not[-_.]?reply|notifications?|mailer-daemon|postmaster|bounces?)([-_.+].*)?$',
  caseSensitive: false,
);

/// L'adresse est-elle celle d'un expéditeur automatique (`noreply@`, `notifications@`…) ?
bool isAutomated(String email) {
  final at = email.indexOf('@');
  return _automated.hasMatch(at < 0 ? email : email.substring(0, at));
}

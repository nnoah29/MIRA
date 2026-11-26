/// Ce que la vue web fait d'une demande de navigation.
enum NavigationVerdict { load, openExternal, block }

const _openable = {'http', 'https', 'mailto'};
const _ownDocument = {'about', 'data'};

/// Décide du sort d'une navigation ; `about:` et `data:` ne sont admis que le temps du chargement du document, jamais depuis un lien du courriel.
NavigationVerdict navigationVerdict(String url, {required bool loading}) {
  final scheme = Uri.tryParse(url)?.scheme.toLowerCase() ?? '';
  if (_openable.contains(scheme)) return NavigationVerdict.openExternal;
  if (loading && _ownDocument.contains(scheme)) return NavigationVerdict.load;
  return NavigationVerdict.block;
}

/// Décide du sort d'une navigation demandée par la page elle-même : jamais d'ouverture externe, un vrai clic passe par `clickedLink`.
NavigationVerdict automaticNavigationVerdict(
  String url, {
  required bool loading,
}) {
  final verdict = navigationVerdict(url, loading: loading);
  return verdict == NavigationVerdict.openExternal
      ? NavigationVerdict.block
      : verdict;
}

/// Adresse d'un clic relayé par le script de page, si elle peut s'ouvrir dans le système.
String? clickedLink(Object? message) {
  if (message is! Map) return null;
  final link = message['link'];
  if (link is! String) return null;
  final verdict = navigationVerdict(link, loading: false);
  return verdict == NavigationVerdict.openExternal ? link : null;
}

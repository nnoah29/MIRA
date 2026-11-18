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

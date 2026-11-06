/// Version du protocole de trames que ce client parle.
const Map<String, int> clientProtocolVersion = {'major': 1, 'minor': 0};

/// Version du contrat d'API pour laquelle l'app est écrite.
const Map<String, int> appContractVersion = {'major': 1, 'minor': 6};

/// Trame `hello` : le champ `auth` porte le jeton local.
Map<String, dynamic> helloFrame({
  required String clientId,
  required String clientName,
  required String token,
  Map<String, int> protocolVersion = clientProtocolVersion,
  Map<String, int> contractVersion = appContractVersion,
}) => {
  'type': 'hello',
  'protocol_version': protocolVersion,
  'contract_version': contractVersion,
  'client_id': clientId,
  'client_name': clientName,
  'auth': {'type': 'local_token', 'token': token},
};

/// Trame `call` : une requête du contrat.
Map<String, dynamic> callFrame(int id, Map<String, dynamic> request) => {
  'type': 'call',
  'id': id,
  'request': request,
};

/// Trame `open_events` : `after` nul pour ne recevoir que les événements futurs.
Map<String, dynamic> openEventsFrame(
  int id, {
  int? after,
  required int credit,
}) => {'type': 'open_events', 'id': id, 'after': after, 'credit': credit};

/// Trame `open_chat` : le `ChatStart` du contrat.
Map<String, dynamic> openChatFrame(
  int id,
  Map<String, dynamic> request, {
  required int credit,
}) => {'type': 'open_chat', 'id': id, 'request': request, 'credit': credit};

/// Trame `get_original_html` pour un message.
Map<String, dynamic> getOriginalHtmlFrame(int id, String message) => {
  'type': 'get_original_html',
  'id': id,
  'message': message,
};

/// Trame `host_op` d'arrêt du démon.
Map<String, dynamic> hostOpShutdownFrame(int id) => {
  'type': 'host_op',
  'id': id,
  'op': 'shutdown',
};

/// Trame `credit` : `n` trames de plus pour le flux `id`.
Map<String, dynamic> creditFrame(int id, int n) => {
  'type': 'credit',
  'id': id,
  'n': n,
};

/// Trame `cancel` d'un échange.
Map<String, dynamic> cancelFrame(int id) => {'type': 'cancel', 'id': id};

/// Trame `ping`.
Map<String, dynamic> pingFrame(int nonce) => {'type': 'ping', 'nonce': nonce};

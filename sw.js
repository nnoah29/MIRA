// Service worker de la démo. Le jeton de build ci-dessous est remplacé à chaque déploiement (packaging/web/build-demo.sh) :
// un nouveau build ouvre un nouveau cache et supprime l'ancien, sans jamais mélanger deux versions.
const CACHE = 'mira-demo-4d6f858-202610301651';
const ALWAYS_FRESH = new Set(['index.html', 'sw.js', 'version.json', 'manifest.json']);

self.addEventListener('install', () => self.skipWaiting());

self.addEventListener('activate', (event) => {
  event.waitUntil(
    caches
      .keys()
      .then((keys) => Promise.all(keys.filter((key) => key !== CACHE).map((key) => caches.delete(key))))
      .then(() => self.clients.claim()),
  );
});

// La page envoie, après son premier affichage, la liste de ce qu'elle a chargé : le cache contient ainsi exactement
// ce qui sert à démarrer (bonne variante de CanvasKit, polices), et la visite suivante ne télécharge plus rien.
self.addEventListener('message', (event) => {
  if (event.data?.type !== 'cache-loaded') return;
  event.waitUntil(
    caches.open(CACHE).then((cache) =>
      Promise.all(
        event.data.urls.map((url) =>
          cache.match(url).then((hit) => hit || cache.add(url).catch(() => undefined)),
        ),
      ),
    ),
  );
});

self.addEventListener('fetch', (event) => {
  const request = event.request;
  const url = new URL(request.url);
  if (request.method !== 'GET' || url.origin !== self.location.origin) return;
  const name = url.pathname.split('/').pop() || 'index.html';
  const fresh = request.mode === 'navigate' || ALWAYS_FRESH.has(name);
  event.respondWith(fresh ? networkFirst(request) : cacheFirst(request));
});

function store(request, response) {
  if (response.ok) {
    const copy = response.clone();
    caches.open(CACHE).then((cache) => cache.put(request, copy));
  }
  return response;
}

function networkFirst(request) {
  return fetch(request)
    .then((response) => store(request, response))
    .catch(() => caches.match(request).then((hit) => hit || caches.match('index.html')));
}

function cacheFirst(request) {
  return caches.match(request).then((hit) => hit || fetch(request).then((response) => store(request, response)));
}

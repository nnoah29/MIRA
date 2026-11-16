#include <malloc.h>

#include "my_application.h"

int main(int argc, char** argv) {
  // Deux arènes glibc suffisent : au-delà, la mémoire libérée reste éparpillée et gonfle le tas.
  mallopt(M_ARENA_MAX, 2);
  // Rend vite la mémoire libre au système : les gros blocs (images, JSON du moteur) passent par mmap.
  mallopt(M_TRIM_THRESHOLD, 128 * 1024);
  mallopt(M_MMAP_THRESHOLD, 128 * 1024);
  g_autoptr(MyApplication) app = my_application_new();
  return g_application_run(G_APPLICATION(app), argc, argv);
}

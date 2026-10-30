package n.noah.mira

import java.security.SecureRandom

/** Où la clé du trousseau, déjà enveloppée, est rangée d'un lancement à l'autre. */
interface WrappedKeyStorage {
    fun read(): ByteArray?
    fun write(sealed: ByteArray)
    fun clear()
}

/** Enveloppe et rouvre la clé du trousseau avec une clé qui ne quitte pas le matériel sécurisé. */
interface KeyWrapper {
    fun wrap(plain: ByteArray): ByteArray
    fun unwrap(sealed: ByteArray): ByteArray
    fun discard()
}

/**
 * Clé (32 octets) du trousseau chiffré du moteur. Tirée une fois au hasard, rangée enveloppée.
 * Si l'enveloppe n'est plus lisible (clé du Keystore perdue ou différente), elle est remplacée
 * par une neuve : mieux vaut un trousseau à refaire qu'un moteur qui ne s'ouvre plus.
 */
class SecretsKey(
    private val storage: WrappedKeyStorage,
    private val wrapper: KeyWrapper,
    private val random: () -> ByteArray = ::randomKey,
    private val onReset: (Exception) -> Unit = {},
) {
    fun get(): ByteArray = storage.read()?.let(::reopen) ?: create()

    /** Rouvre la clé rangée ; `null` si elle est illisible, après avoir jeté l'enveloppe et la clé du Keystore. */
    private fun reopen(sealed: ByteArray): ByteArray? =
        try {
            wrapper.unwrap(sealed).also { check(it.size == KEY_SIZE) { "clé de taille inattendue" } }
        } catch (e: Exception) {
            onReset(e)
            storage.clear()
            wrapper.discard()
            null
        }

    /** Tire une clé neuve et range son enveloppe. */
    private fun create(): ByteArray {
        val key = random()
        storage.write(wrapper.wrap(key))
        return key
    }

    companion object {
        const val KEY_SIZE = 32

        private fun randomKey(): ByteArray = ByteArray(KEY_SIZE).also { SecureRandom().nextBytes(it) }
    }
}

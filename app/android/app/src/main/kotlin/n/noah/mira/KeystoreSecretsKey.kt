package n.noah.mira

import android.content.Context
import android.security.keystore.KeyGenParameterSpec
import android.security.keystore.KeyProperties
import android.util.Base64
import android.util.Log
import java.io.File
import java.security.KeyStore
import javax.crypto.Cipher
import javax.crypto.KeyGenerator
import javax.crypto.SecretKey
import javax.crypto.spec.GCMParameterSpec

private const val ALIAS = "mira-secrets-wrap"
private const val PREFS = "mira-keystore"
private const val ENTRY = "wrapped"
private const val SECRETS_FILE = "secrets.bin"

/** Clé du trousseau d'une app Android : enveloppe gardée dans les préférences, clé d'enveloppe dans le Keystore. */
fun secretsKeyFor(context: Context): SecretsKey =
    SecretsKey(PrefsWrappedKey(context), KeystoreWrapper()) { cause ->
        Log.w("MIRA", "clé du trousseau illisible, remplacée par une neuve", cause)
        File(context.filesDir, SECRETS_FILE).delete()
    }

/** Enveloppe de la clé du trousseau, rangée en Base64 dans les préférences privées de l'app. */
private class PrefsWrappedKey(context: Context) : WrappedKeyStorage {
    private val prefs = context.getSharedPreferences(PREFS, Context.MODE_PRIVATE)

    override fun read(): ByteArray? = prefs.getString(ENTRY, null)?.let { Base64.decode(it, Base64.NO_WRAP) }

    override fun write(sealed: ByteArray) {
        prefs.edit().putString(ENTRY, Base64.encodeToString(sealed, Base64.NO_WRAP)).commit()
    }

    override fun clear() {
        prefs.edit().remove(ENTRY).commit()
    }
}

/** Enveloppe AES-256-GCM par une clé du Keystore Android qui ne quitte jamais le matériel sécurisé. */
private class KeystoreWrapper : KeyWrapper {
    private fun store(): KeyStore = KeyStore.getInstance("AndroidKeyStore").apply { load(null) }

    private fun wrappingKey(): SecretKey {
        (store().getKey(ALIAS, null) as? SecretKey)?.let { return it }
        val generator = KeyGenerator.getInstance(KeyProperties.KEY_ALGORITHM_AES, "AndroidKeyStore")
        generator.init(
            KeyGenParameterSpec.Builder(ALIAS, KeyProperties.PURPOSE_ENCRYPT or KeyProperties.PURPOSE_DECRYPT)
                .setBlockModes(KeyProperties.BLOCK_MODE_GCM)
                .setEncryptionPaddings(KeyProperties.ENCRYPTION_PADDING_NONE)
                .setKeySize(256)
                .build()
        )
        return generator.generateKey()
    }

    override fun wrap(plain: ByteArray): ByteArray {
        val cipher = Cipher.getInstance("AES/GCM/NoPadding")
        cipher.init(Cipher.ENCRYPT_MODE, wrappingKey())
        return cipher.iv + cipher.doFinal(plain)
    }

    override fun unwrap(sealed: ByteArray): ByteArray {
        val cipher = Cipher.getInstance("AES/GCM/NoPadding")
        cipher.init(Cipher.DECRYPT_MODE, wrappingKey(), GCMParameterSpec(128, sealed, 0, 12))
        return cipher.doFinal(sealed, 12, sealed.size - 12)
    }

    override fun discard() {
        store().deleteEntry(ALIAS)
    }
}

package n.noah.mira

import android.content.Context
import android.content.pm.ApplicationInfo
import android.security.keystore.KeyGenParameterSpec
import android.security.keystore.KeyProperties
import android.os.Bundle
import android.util.Base64
import android.view.WindowManager
import io.flutter.embedding.android.FlutterActivity
import io.flutter.embedding.engine.FlutterEngine
import io.flutter.plugin.common.MethodChannel
import java.security.KeyStore
import java.security.SecureRandom
import javax.crypto.Cipher
import javax.crypto.KeyGenerator
import javax.crypto.SecretKey
import javax.crypto.spec.GCMParameterSpec

class MainActivity : FlutterActivity() {
    /** En développement seulement : l'écran reste allumé tant que l'app est au premier plan. */
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        if (applicationInfo.flags and ApplicationInfo.FLAG_DEBUGGABLE != 0) {
            window.addFlags(WindowManager.LayoutParams.FLAG_KEEP_SCREEN_ON)
        }
    }

    override fun configureFlutterEngine(flutterEngine: FlutterEngine) {
        super.configureFlutterEngine(flutterEngine)
        MethodChannel(flutterEngine.dartExecutor.binaryMessenger, "mira/keystore")
            .setMethodCallHandler { call, result ->
                if (call.method == "secretsKey") {
                    try {
                        result.success(SecretsKey.get(applicationContext))
                    } catch (e: Exception) {
                        result.error("keystore", "clé du trousseau indisponible", null)
                    }
                } else {
                    result.notImplemented()
                }
            }
    }
}

/**
 * Clé (32 octets) du trousseau chiffré du moteur. Tirée une fois au hasard, elle est rangée
 * chiffrée par une clé du Keystore Android qui ne quitte jamais le matériel sécurisé.
 */
object SecretsKey {
    private const val ALIAS = "mira-secrets-wrap"
    private const val PREFS = "mira-keystore"
    private const val ENTRY = "wrapped"

    fun get(context: Context): ByteArray {
        val prefs = context.getSharedPreferences(PREFS, Context.MODE_PRIVATE)
        prefs.getString(ENTRY, null)?.let { return unwrap(Base64.decode(it, Base64.NO_WRAP)) }
        val key = ByteArray(32).also { SecureRandom().nextBytes(it) }
        prefs.edit().putString(ENTRY, Base64.encodeToString(wrap(key), Base64.NO_WRAP)).apply()
        return key
    }

    private fun wrappingKey(): SecretKey {
        val store = KeyStore.getInstance("AndroidKeyStore").apply { load(null) }
        (store.getKey(ALIAS, null) as? SecretKey)?.let { return it }
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

    private fun wrap(plain: ByteArray): ByteArray {
        val cipher = Cipher.getInstance("AES/GCM/NoPadding")
        cipher.init(Cipher.ENCRYPT_MODE, wrappingKey())
        return cipher.iv + cipher.doFinal(plain)
    }

    private fun unwrap(sealed: ByteArray): ByteArray {
        val cipher = Cipher.getInstance("AES/GCM/NoPadding")
        cipher.init(Cipher.DECRYPT_MODE, wrappingKey(), GCMParameterSpec(128, sealed, 0, 12))
        return cipher.doFinal(sealed, 12, sealed.size - 12)
    }
}

package n.noah.mira

import android.content.pm.ApplicationInfo
import android.os.Bundle
import android.view.WindowManager
import io.flutter.embedding.android.FlutterActivity
import io.flutter.embedding.engine.FlutterEngine
import io.flutter.plugin.common.MethodChannel

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
                        result.success(secretsKeyFor(applicationContext).get())
                    } catch (e: Exception) {
                        result.error("keystore", "clé du trousseau indisponible", e.javaClass.simpleName)
                    }
                } else {
                    result.notImplemented()
                }
            }
    }
}

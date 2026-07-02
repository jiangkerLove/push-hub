package com.jiangker.push.sample

import android.content.Intent
import android.os.Bundle
import android.util.Log
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.Card
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import com.jiangker.push.sample.ui.theme.SampleTheme

/**
 * 小米等厂商「打开应用内任意 Activity」调试页。
 *
 * Manifest: `android:name=".DemoTargetActivity"` + `launchMode="singleTop"`
 * 服务端：`click_action.type=open_page`，`activity=com.jiangker.push.sample.DemoTargetActivity`（全类名），
 * params 会写入 intent extras（冷启动 onCreate / 热启动 onNewIntent）。
 */
class DemoTargetActivity : ComponentActivity() {
    private var launchMode by mutableStateOf("—")
    private var paramsText by mutableStateOf("（无）")

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        enableEdgeToEdge()
        applyIntent(intent, coldStart = true)
        setContent {
            SampleTheme {
                Scaffold(modifier = Modifier.fillMaxSize()) { innerPadding ->
                    DemoTargetScreen(
                        launchMode = launchMode,
                        paramsText = paramsText,
                        modifier = Modifier.padding(innerPadding),
                    )
                }
            }
        }
    }

    override fun onNewIntent(intent: Intent) {
        super.onNewIntent(intent)
        setIntent(intent)
        applyIntent(intent, coldStart = false)
    }

    private fun applyIntent(intent: Intent?, coldStart: Boolean) {
        val params = PushIntentExtras.parse(intent)
        launchMode = if (coldStart) "冷启动 · onCreate" else "热启动 · onNewIntent"
        paramsText = if (params.isEmpty()) {
            "（没有解析到业务参数）"
        } else {
            params.entries.joinToString("\n") { (key, value) -> "$key = $value" }
        }
        Log.i(TAG, PushIntentExtras.describeLaunch(coldStart, params))
    }

    private companion object {
        const val TAG = "DemoTargetActivity"
    }
}

@Composable
private fun DemoTargetScreen(
    launchMode: String,
    paramsText: String,
    modifier: Modifier = Modifier,
) {
    Column(
        modifier = modifier
            .fillMaxSize()
            .verticalScroll(rememberScrollState())
            .padding(20.dp),
        verticalArrangement = Arrangement.spacedBy(16.dp),
    ) {
        Text(
            text = "推送目标页",
            style = MaterialTheme.typography.headlineSmall,
            fontWeight = FontWeight.Bold,
        )
        Text(
            text = "对应 Manifest：.DemoTargetActivity\n服务端 Activity 填：com.jiangker.push.sample.DemoTargetActivity",
            style = MaterialTheme.typography.bodyMedium,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
        )

        InfoCard(title = "启动方式", body = launchMode)
        InfoCard(title = "Intent 参数", body = paramsText)

        Spacer(modifier = Modifier.height(8.dp))
        Text(
            text = "冷启动：进程不在时点通知会复现 onCreate。\n"
                + "热启动：进程在前台/后台时点通知，singleTop + CLEAR_TOP 走 onNewIntent。",
            style = MaterialTheme.typography.bodySmall,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
        )
    }
}

@Composable
private fun InfoCard(title: String, body: String) {
    Card(
        modifier = Modifier.fillMaxWidth(),
        colors = CardDefaults.cardColors(
            containerColor = MaterialTheme.colorScheme.surfaceVariant.copy(alpha = 0.55f),
        ),
    ) {
        Column(modifier = Modifier.padding(16.dp)) {
            Text(
                text = title,
                style = MaterialTheme.typography.labelLarge,
                color = MaterialTheme.colorScheme.primary,
            )
            Spacer(modifier = Modifier.height(8.dp))
            Text(
                text = body,
                style = MaterialTheme.typography.bodyLarge,
                fontFamily = FontFamily.Monospace,
            )
        }
    }
}

package com.jiangker.push.core

import android.app.ActivityManager
import android.content.Context
import android.os.Process

object ProcessUtils {
    fun isMainProcess(context: Context): Boolean {
        val processName = currentProcessName(context) ?: return true
        return processName == context.applicationInfo.processName
    }

    private fun currentProcessName(context: Context): String? {
        val pid = Process.myPid()
        val activityManager = context.getSystemService(Context.ACTIVITY_SERVICE) as ActivityManager
        return activityManager.runningAppProcesses
            ?.firstOrNull { it.pid == pid }
            ?.processName
    }
}

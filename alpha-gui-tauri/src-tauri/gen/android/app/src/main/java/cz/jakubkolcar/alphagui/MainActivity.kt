package cz.jakubkolcar.alphagui

import android.app.Activity
import android.app.PendingIntent
import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent
import android.content.IntentFilter
import android.net.Uri
import android.os.Build
import android.os.Bundle
import android.hardware.usb.UsbDevice
import android.hardware.usb.UsbManager
import androidx.activity.result.ActivityResultLauncher
import androidx.activity.result.contract.ActivityResultContracts
import androidx.activity.enableEdgeToEdge
import androidx.documentfile.provider.DocumentFile
import java.util.concurrent.atomic.AtomicBoolean
import java.util.concurrent.ArrayBlockingQueue
import java.util.concurrent.TimeUnit

class MainActivity : TauriActivity() {
  private lateinit var backupDirectoryLauncher: ActivityResultLauncher<Intent>

  override fun onCreate(savedInstanceState: Bundle?) {
    enableEdgeToEdge()
    currentActivity = this
    backupDirectoryLauncher = registerForActivityResult(
      ActivityResultContracts.StartActivityForResult()
    ) { result ->
      finishBackupDirectoryPick(result.resultCode, result.data)
    }
    super.onCreate(savedInstanceState)
  }

  override fun onDestroy() {
    pendingDirectoryPick?.offer(errorResult("activity closed while backup folder picker was open"))
    pendingDirectoryPick = null
    if (currentActivity === this) {
      currentActivity = null
    }
    super.onDestroy()
  }

  private fun finishBackupDirectoryPick(resultCode: Int, data: Intent?) {
    val selected = if (resultCode == Activity.RESULT_OK) data?.data else null
    val selectedText = try {
      if (selected != null && data != null) {
        val flags = data.flags and
          (Intent.FLAG_GRANT_READ_URI_PERMISSION or Intent.FLAG_GRANT_WRITE_URI_PERMISSION)
        contentResolver.takePersistableUriPermission(selected, flags)
      }
      selected?.toString()
    } catch (error: SecurityException) {
      errorResult("backup folder permission failed: ${error.message ?: error.javaClass.name}")
    }
    pendingDirectoryPick?.offer(selectedText ?: "")
    pendingDirectoryPick = null
  }

  private fun launchBackupDirectoryPicker(queue: ArrayBlockingQueue<String>) {
    val intent = Intent(Intent.ACTION_OPEN_DOCUMENT_TREE).apply {
      addFlags(Intent.FLAG_GRANT_READ_URI_PERMISSION)
      addFlags(Intent.FLAG_GRANT_WRITE_URI_PERMISSION)
      addFlags(Intent.FLAG_GRANT_PERSISTABLE_URI_PERMISSION)
      addFlags(Intent.FLAG_GRANT_PREFIX_URI_PERMISSION)
    }
    pendingDirectoryPick = queue
    backupDirectoryLauncher.launch(intent)
  }

  companion object {
    private const val PICKER_TIMEOUT_SECONDS = 120L
    private const val USB_PERMISSION_TIMEOUT_SECONDS = 30L
    private const val PICKER_ERROR_PREFIX = "__ALPHAGUI_ERROR__:"
    private const val ACTION_USB_PERMISSION = "cz.jakubkolcar.alphagui.USB_PERMISSION"

    @Volatile private var currentActivity: MainActivity? = null
    @Volatile private var pendingDirectoryPick: ArrayBlockingQueue<String>? = null

    @JvmStatic
    @Synchronized
    fun pickBackupDirectoryBlocking(): String? {
      val activity = currentActivity ?: throw IllegalStateException("AlphaGUI activity is not available")
      if (pendingDirectoryPick != null) {
        throw IllegalStateException("backup directory picker is already open")
      }
      val queue = ArrayBlockingQueue<String>(1)
      activity.runOnUiThread {
        try {
          activity.launchBackupDirectoryPicker(queue)
        } catch (error: Throwable) {
          pendingDirectoryPick = null
          queue.offer(errorResult("backup folder picker failed: ${error.message ?: error.javaClass.name}"))
        }
      }
      val selected = queue.poll(PICKER_TIMEOUT_SECONDS, TimeUnit.SECONDS)
        ?: throw IllegalStateException("backup folder picker timed out")
      return selected.ifEmpty { null }
    }

    @JvmStatic
    fun requestUsbPermissionBlocking(device: UsbDevice): Boolean {
      val activity = currentActivity ?: throw IllegalStateException("AlphaGUI activity is not available")
      val manager = activity.getSystemService(Context.USB_SERVICE) as UsbManager
      if (manager.hasPermission(device)) {
        return true
      }

      val started = ArrayBlockingQueue<Boolean>(1)
      val queue = ArrayBlockingQueue<Boolean>(1)
      val cancelled = AtomicBoolean(false)
      val receiver = object : BroadcastReceiver() {
        override fun onReceive(context: Context, intent: Intent) {
          if (intent.action != ACTION_USB_PERMISSION) return
          val granted = intent.getBooleanExtra(UsbManager.EXTRA_PERMISSION_GRANTED, false)
          queue.offer(granted)
        }
      }
      activity.runOnUiThread {
        if (cancelled.get()) {
          started.offer(false)
          queue.offer(false)
          return@runOnUiThread
        }
        val filter = IntentFilter(ACTION_USB_PERMISSION)
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
          activity.registerReceiver(receiver, filter, Context.RECEIVER_NOT_EXPORTED)
        } else {
          @Suppress("DEPRECATION")
          activity.registerReceiver(receiver, filter)
        }

        try {
          val permissionIntent = PendingIntent.getBroadcast(
            activity,
            0,
            Intent(ACTION_USB_PERMISSION).setPackage(activity.packageName),
            PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_MUTABLE
          )
          manager.requestPermission(device, permissionIntent)
          started.offer(true)
        } catch (error: Throwable) {
          try {
            activity.unregisterReceiver(receiver)
          } catch (_: IllegalArgumentException) {
          }
          started.offer(false)
          queue.offer(false)
        }
      }
      if (started.poll(USB_PERMISSION_TIMEOUT_SECONDS, TimeUnit.SECONDS) != true) {
        cancelled.set(true)
        return false
      }
      val granted = queue.poll(USB_PERMISSION_TIMEOUT_SECONDS, TimeUnit.SECONDS) == true
      try {
        activity.unregisterReceiver(receiver)
      } catch (_: IllegalArgumentException) {
      }
      return granted
    }

    @JvmStatic
    fun readUriBytes(uri: String): ByteArray {
      val activity = currentActivity ?: throw IllegalStateException("AlphaGUI activity is not available")
      return activity.contentResolver.openInputStream(Uri.parse(uri))?.use { stream ->
        stream.readBytes()
      } ?: throw IllegalStateException("cannot open Android content URI for reading")
    }

    @JvmStatic
    fun writeBackupFile(rootUri: String, relativePath: String, bytes: ByteArray) {
      val activity = currentActivity ?: throw IllegalStateException("AlphaGUI activity is not available")
      val root = DocumentFile.fromTreeUri(activity, Uri.parse(rootUri))
        ?: throw IllegalArgumentException("backup folder URI is invalid")
      val parts = relativePath.split('/').filter { it.isNotBlank() }
      require(parts.isNotEmpty()) { "backup relative path is empty" }

      var directory = root
      for (name in parts.dropLast(1)) {
        directory = directory.findFile(name)?.takeIf { it.isDirectory }
          ?: directory.createDirectory(name)
          ?: throw IllegalStateException("cannot create backup directory $name")
      }

      val fileName = parts.last()
      directory.findFile(fileName)?.let { existing ->
        check(existing.isFile && existing.delete()) { "cannot replace backup file $fileName" }
      }
      val file = directory.createFile(mimeTypeFor(fileName), fileName)
        ?: throw IllegalStateException("cannot create backup file $fileName")
      activity.contentResolver.openOutputStream(file.uri, "wt")?.use { stream ->
        stream.write(bytes)
      } ?: throw IllegalStateException("cannot open backup file $fileName for writing")
    }

    private fun mimeTypeFor(fileName: String): String {
      return when {
        fileName.endsWith(".txt", ignoreCase = true) -> "text/plain"
        else -> "application/octet-stream"
      }
    }

    private fun errorResult(message: String): String {
      return "$PICKER_ERROR_PREFIX$message"
    }
  }
}

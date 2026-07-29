import 'dart:async';
import 'dart:convert';
import 'dart:io';
import 'package:path_provider/path_provider.dart';

class CodeExecutorService {
  /// Mendapatkan daftar versi runtime yang terpasang berdasarkan bahasa (contoh: 'node' atau 'php')
  static Future<List<String>> getInstalledRuntimePaths(String language) async {
    try {
      final supportDir = await getApplicationSupportDirectory();
      final runtimesDir = Directory('${supportDir.path}/runtimes');
      if (!await runtimesDir.exists()) {
        return [];
      }

      final List<String> paths = [];
      final entries = await runtimesDir.list().toList();
      for (final entry in entries) {
        if (entry is Directory) {
          final dirName = entry.uri.pathSegments.where((s) => s.isNotEmpty).last;
          if (dirName.startsWith('$language-')) {
            final exePath = '${entry.path}/$language${Platform.isWindows ? '.exe' : ''}';
            if (await File(exePath).exists()) {
              paths.add(exePath);
            } else {
              // Cadangan: jika ekstensi tidak ada (khususnya untuk dummy di windows tanpa .exe)
              final noExtPath = '${entry.path}/$language';
              if (await File(noExtPath).exists()) {
                paths.add(noExtPath);
              }
            }
          }
        }
      }
      
      return paths;
    } catch (e) {
      return [];
    }
  }

  /// Menjalankan kode pada file sementara menggunakan binary runtime yang dipilih
  static Future<String> executeWithRuntime(String exePath, String content, String language) async {
    final tempDir = await getTemporaryDirectory();
    final ext = language == 'node' ? 'js' : language;
    final tempFile = File('${tempDir.path}/rpl_temp_${DateTime.now().millisecondsSinceEpoch}.$ext');
    
    try {
      await tempFile.writeAsString(content);

      String targetExe = exePath;
      List<String> processArgs = [tempFile.path];
      Map<String, String>? environment;
      
      // Khusus untuk Node.js di Android:
      // OS Android sering memblokir execve() langsung pada binary dinamis di folder data aplikasi (SELinux).
      // Triknya adalah mengeksekusi sistem linker secara eksplisit dan melempar binary sebagai argumen pertama!
      if (!Platform.isWindows) {
        final binFile = File('${exePath}.bin');
        if (await binFile.exists()) {
          targetExe = '/system/bin/linker64'; // Trik bypass SELinux
          processArgs = [binFile.path, tempFile.path];
          
          final libDir = Directory('${File(exePath).parent.path}/lib');
          if (await libDir.exists()) {
            final currentLdPath = Platform.environment['LD_LIBRARY_PATH'] ?? '';
            environment = {
              'LD_LIBRARY_PATH': '${libDir.path}:$currentLdPath'
            };
          }
        }
      }

      final process = await Process.start(targetExe, processArgs, environment: environment);

      final stdoutCompleter = Completer<String>();
      final stderrCompleter = Completer<String>();

      process.stdout.transform(utf8.decoder).join().then(stdoutCompleter.complete);
      process.stderr.transform(utf8.decoder).join().then(stderrCompleter.complete);

      // Timeout setelah 10 detik
      final exitCode = await process.exitCode.timeout(
        const Duration(seconds: 10),
        onTimeout: () {
          process.kill(ProcessSignal.sigkill);
          throw TimeoutException('Eksekusi dibatalkan karena melebihi batas waktu (10 detik).');
        },
      );

      final out = await stdoutCompleter.future;
      final err = await stderrCompleter.future;

      if (exitCode != 0) {
        return out.isEmpty ? err : '$out\n$err';
      }
      return out.isEmpty && err.isNotEmpty ? err : out;
    } catch (e) {
      if (e is TimeoutException) {
        return e.message ?? 'Timeout';
      }
      return 'Gagal mengeksekusi kode: ${e.toString()}';
    } finally {
      if (await tempFile.exists()) {
        await tempFile.delete();
      }
    }
  }
}

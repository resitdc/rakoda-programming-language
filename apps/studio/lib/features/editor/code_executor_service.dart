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
            // Kita perlu memeriksa beberapa lokasi yang mungkin untuk binary
            final possiblePaths = [
              if (Platform.isAndroid) '${entry.path}/bin/${language}_wrapper',
              if (Platform.isAndroid) '${entry.path}/$language/bin/${language}_wrapper',
              '${entry.path}/$language${Platform.isWindows ? '.exe' : ''}',
              '${entry.path}/bin/$language${Platform.isWindows ? '.exe' : ''}',
              '${entry.path}/$language/bin/$language${Platform.isWindows ? '.exe' : ''}',
              '${entry.path}/$language/bin/${language}3${Platform.isWindows ? '.exe' : ''}', // khusus python3
              '${entry.path}/${language}3${Platform.isWindows ? '.exe' : ''}',
            ];

            bool found = false;
            for (final p in possiblePaths) {
              final file = File(p);
              if (await file.exists()) {
                final stat = await file.stat();
                if (stat.type != FileSystemEntityType.directory) {
                  paths.add(p);
                  found = true;
                  break;
                }
              }
            }

            if (!found) {
              // Cadangan: tanpa ekstensi untuk dummy/alias di OS tertentu
              final possiblePathsNoExt = [
                '${entry.path}/$language',
                '${entry.path}/bin/$language',
                '${entry.path}/$language/bin/$language',
                '${entry.path}/$language/bin/${language}3',
                '${entry.path}/${language}3',
              ];
              for (final p in possiblePathsNoExt) {
                final file = File(p);
                if (await file.exists()) {
                  final stat = await file.stat();
                  if (stat.type != FileSystemEntityType.directory) {
                    paths.add(p);
                    break;
                  }
                }
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
  static Future<String> executeWithRuntime(String exePath, String content, String language, {String? workingDirectory}) async {
    final tempDir = await getTemporaryDirectory();
    final ext = language == 'node' ? 'js' : language;
    final tempFile = File('${tempDir.path}/rpl_temp_${DateTime.now().millisecondsSinceEpoch}.$ext');
    
    try {
      await tempFile.writeAsString(content);

      String targetExe = exePath;
      List<String> processArgs = [tempFile.path];
      
      if (language == 'go') {
        processArgs = ['run', tempFile.path];
      } else if (language == 'kt') {
        processArgs = ['-script', tempFile.path];
      }
      
      Map<String, String>? environment;
      
      if (language == 'php') {
        processArgs = ['-d', 'opcache.enable=0', '-d', 'opcache.enable_cli=0', tempFile.path];
        environment = {
          'TMPDIR': Directory.systemTemp.path,
        };
      }
      // Khusus untuk di Android:
      // OS Android sering memblokir execve() langsung pada binary dinamis di folder data aplikasi (SELinux).
      // Triknya adalah mengeksekusi sistem linker secara eksplisit, ATAU mengeksekusi shell script dengan 'sh'.
      if (Platform.isAndroid) {
        bool isShellScript = false;
        try {
          final firstLine = await File(exePath).openRead().transform(utf8.decoder).transform(const LineSplitter()).first;
          if (firstLine.startsWith('#!')) {
            isShellScript = true;
          }
        } catch (_) {}

        if (isShellScript) {
          targetExe = '/system/bin/sh';
          processArgs = [exePath, ...processArgs];
        } else {
          // Fallback to linker64 for ELF binaries
          targetExe = '/system/bin/linker64';
          processArgs = [exePath, ...processArgs];
          
          final libDir1 = Directory('${File(exePath).parent.path}/lib');
          final libDir2 = Directory('${File(exePath).parent.parent.path}/lib');
          
          String ldLibPath = '';
          if (await libDir1.exists()) ldLibPath += '${libDir1.path}:';
          if (await libDir2.exists()) ldLibPath += '${libDir2.path}:';
          
          if (ldLibPath.isNotEmpty) {
            final currentLdPath = Platform.environment['LD_LIBRARY_PATH'] ?? '';
            environment = {
              ...?environment,
              'LD_LIBRARY_PATH': '$ldLibPath$currentLdPath'
            };
          }
        }
      }

      final process = await Process.start(targetExe, processArgs, environment: environment, workingDirectory: workingDirectory);

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

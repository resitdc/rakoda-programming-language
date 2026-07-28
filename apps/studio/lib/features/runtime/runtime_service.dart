import 'dart:convert';
import 'dart:io';
import 'package:http/http.dart' as http;
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'runtime_model.dart';

final runtimeManifestProvider = FutureProvider<RuntimeManifest>((ref) async {
  const manifestUrl = 'https://raw.githubusercontent.com/resitdc/rakoda-runtime/main/manifest/runtime.json';
  
  final response = await http.get(Uri.parse(manifestUrl)).timeout(const Duration(seconds: 10));
  
  if (response.statusCode == 200) {
    final Map<String, dynamic> jsonMap = json.decode(response.body);
    return RuntimeManifest.fromJson(jsonMap);
  } else {
    throw Exception('Gagal memuat manifest dari server (Status: ${response.statusCode})');
  }
});

class SystemInfo {
  final String os;
  final String arch;

  SystemInfo({required this.os, required this.arch});

  static SystemInfo getCurrent() {
    String currentOs = 'linux';
    if (Platform.isWindows) currentOs = 'windows';
    if (Platform.isMacOS) currentOs = 'macos';
    if (Platform.isAndroid) currentOs = 'android';

    String currentArch = 'x64';
    if (Platform.isAndroid) {
      currentArch = 'arm64';
    } else if (Platform.isMacOS) {
      // Dart Platform.version contains info about the architecture
      if (Platform.version.contains('arm64') || Platform.version.contains('aarch64')) {
        currentArch = 'arm64';
      }
    } else if (Platform.isLinux) {
      if (Platform.version.contains('aarch64')) {
        currentArch = 'arm64';
      }
    }

    return SystemInfo(os: currentOs, arch: currentArch);
  }
}

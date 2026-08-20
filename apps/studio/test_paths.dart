import 'dart:io';

void main() async {
  final supportDir = "/Users/resitdc/Library/Application Support/com.rakoda.rplStudio";
  final runtimesDir = Directory('$supportDir/runtimes');
  if (!runtimesDir.existsSync()) {
    print("No runtimes");
    return;
  }
  final entries = runtimesDir.listSync();
  for (final entry in entries) {
    if (entry is Directory) {
      final dirName = entry.uri.pathSegments.where((s) => s.isNotEmpty).last;
      if (dirName.startsWith('php-')) {
        final possiblePaths = [
          '${entry.path}/php',
          '${entry.path}/bin/php',
        ];
        for (final p in possiblePaths) {
          if (File(p).existsSync()) {
            print("Found: $p");
          }
        }
      }
    }
  }
}

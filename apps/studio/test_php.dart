import 'dart:io';

void main() {
  final supportDir = '/Users/resitdc/Library/Application Support/com.resitdc.rplStudio';
  final runtimesDir = Directory('${supportDir}/runtimes');
  if (runtimesDir.existsSync()) {
    final entries = runtimesDir.listSync();
    print("Found runtimesDir. Entries:");
    for (final entry in entries) {
      print(entry.path);
    }
  } else {
    print("No runtimesDir at $supportDir");
  }
}

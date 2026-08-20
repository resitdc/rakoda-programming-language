import 'dart:io';

void main() async {
  final supportDir = "${Platform.environment['HOME']}/Library/Application Support/com.rakoda.rpl-studio";
  final runtimesDir = Directory('$supportDir/runtimes');
  if (await runtimesDir.exists()) {
    print("Runtimes dir exists!");
    for (var entity in runtimesDir.listSync()) {
      print("Found: ${entity.path}");
    }
  } else {
    print("Runtimes dir not found at $supportDir");
  }
}

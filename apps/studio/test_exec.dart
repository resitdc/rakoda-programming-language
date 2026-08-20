import 'dart:io';

void main() async {
  File('test_bin').writeAsStringSync('#!/bin/bash\necho "Hello"');
  // No execute bit
  try {
    final p = await Process.start('./test_bin', []);
    p.stdout.listen((d) => stdout.add(d));
    print("Success!");
  } catch(e) {
    print("Failed: $e");
  }
}

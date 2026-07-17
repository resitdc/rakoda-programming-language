import 'package:sqflite_common_ffi/sqflite_ffi.dart';
void main() {
  print(OpenDatabaseOptions(singleInstance: false).singleInstance);
}
